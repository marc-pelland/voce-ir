#!/usr/bin/env node
/**
 * Voce IR MCP Server — exposes Voce IR tools for Claude Code and MCP clients.
 *
 * Pipeline tools (S65 Day 1 description rewrite):
 *   voce_validate, voce_compile, voce_inspect, voce_schema, voce_examples,
 *   voce_generate.
 *
 * Memory tools (S65 Day 3):
 *   voce_brief_get, voce_brief_set,
 *   voce_decisions_list, voce_decisions_log,
 *   voce_session_resume, voce_check_drift.
 *
 * Resources (S65 Day 2 wired through .voce/ store):
 *   voce://brief, voce://decisions, voce://drift-warnings, voce://status.
 *
 * Start: npx voce-mcp (stdio transport)
 */

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  ListResourcesRequestSchema,
  ReadResourceRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { execSync } from "node:child_process";
import { readFileSync, existsSync, writeFileSync, unlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import {
  detectDrift,
  latestIrSnapshot,
  listDecisions,
  listDrift,
  listSessions,
  logDecision,
  readBrief,
  readSession,
  writeBrief,
} from "./memory/index.js";
import {
  gateFinalize,
  getWorkflowState,
  recordAnswer,
  recordFinalization,
  recordProposal,
  recordRefinement,
  startGeneration,
} from "./workflow/index.js";

// Find the voce binary — check local workspace build first, then PATH
const __dirname = dirname(fileURLToPath(import.meta.url));
function findVoceBinary(): string {
  const candidates = [
    resolve(__dirname, "../../../target/release/voce"),
    resolve(__dirname, "../../../target/debug/voce"),
  ];
  for (const c of candidates) {
    if (existsSync(c)) return c;
  }
  return "voce"; // fall back to PATH
}
const VOCE_BIN = findVoceBinary();

const server = new Server(
  { name: "voce-ir", version: "0.3.0" },
  { capabilities: { tools: {}, resources: {} } }
);

// ── Tools ────────────────────────────────────────────────────────

// Tool descriptions encode Voce's conversational pillars — every MCP client
// inherits the right behavior without client-side prompting. Total budget
// across all `description` fields stays under ~1 KB to keep token cost low
// on every model turn that lists tools.
server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [
    {
      name: "voce_validate",
      description:
        "Validate a Voce IR document. Returns per-pass diagnostics (severity, code, path, hint). Run before compile — a11y, security, SEO are errors in Voce, not warnings. Fix every error before declaring IR done.",
      inputSchema: {
        type: "object" as const,
        properties: {
          ir_json: { type: "string", description: "Voce IR JSON to validate" },
        },
        required: ["ir_json"],
      },
    },
    {
      name: "voce_compile",
      description:
        "Compile validated Voce IR to HTML. Run only after voce_validate passes — never present output from invalid IR as final. Result has zero runtime JS.",
      inputSchema: {
        type: "object" as const,
        properties: {
          ir_json: { type: "string", description: "Voce IR JSON to compile" },
        },
        required: ["ir_json"],
      },
    },
    {
      name: "voce_inspect",
      description:
        "Structured summary of an IR document — node counts, semantic tree, features. Run before compile to confirm intent and spot missing pillars (semantics, error/loading/empty states).",
      inputSchema: {
        type: "object" as const,
        properties: {
          ir_json: { type: "string", description: "Voce IR JSON to inspect" },
        },
        required: ["ir_json"],
      },
    },
    {
      name: "voce_schema",
      description:
        "Schema docs for a Voce IR node type, or all types if omitted. Covers layout, state, motion, navigation, a11y, theming, data, forms, SEO, i18n.",
      inputSchema: {
        type: "object" as const,
        properties: {
          node_type: {
            type: "string",
            description: "Node type, e.g. Container, FormNode. Omit for the overview.",
          },
        },
      },
    },
    {
      name: "voce_examples",
      description:
        "List or retrieve reference IR that compiles and validates cleanly. Start from an example and modify, rather than authoring from scratch.",
      inputSchema: {
        type: "object" as const,
        properties: {
          name: {
            type: "string",
            description: "Example name, e.g. landing-page. Omit to list all.",
          },
        },
      },
    },
    {
      name: "voce_generate",
      description:
        "One-shot legacy entry. Prefer the workflow: voce_generate_start → answer (×N) → propose → refine → finalize. Skipping discovery violates the conversational pillars.",
      inputSchema: {
        type: "object" as const,
        properties: {
          prompt: {
            type: "string",
            description: "Concrete brief built from prior discovery turns",
          },
        },
        required: ["prompt"],
      },
    },
    // ── Memory tools (S65 Day 3) ──────────────────────────────────
    {
      name: "voce_brief_get",
      description:
        "Read the project brief — the north star every generation is checked against. Returns null content when no brief exists yet.",
      inputSchema: { type: "object" as const, properties: {} },
    },
    {
      name: "voce_brief_set",
      description:
        "Replace the project brief with new markdown. Atomic write. Confirm with the user BEFORE invoking — overwriting the brief is consequential and the storage layer does not prompt.",
      inputSchema: {
        type: "object" as const,
        properties: {
          brief_md: { type: "string", description: "New brief content (markdown)" },
        },
        required: ["brief_md"],
      },
    },
    {
      name: "voce_decisions_list",
      description:
        "List decisions, oldest first. Read this before proposing IR — drift checks reference these entries. Optional ISO-8601 since filter.",
      inputSchema: {
        type: "object" as const,
        properties: {
          since: { type: "string", description: "ISO-8601 cutoff (inclusive)" },
        },
      },
    },
    {
      name: "voce_decisions_log",
      description:
        "Append a decision to the log. Use when the conversation produces a durable choice (architecture, scope, anti-pattern). Include rationale; future drift checks reference it.",
      inputSchema: {
        type: "object" as const,
        properties: {
          summary: { type: "string", description: "One-line decision summary" },
          rationale: { type: "string", description: "Why this decision was made" },
          supersedes: { type: "string", description: "id of a prior decision this replaces" },
          conflicts_with: { type: "string", description: "id of a decision this knowingly overrides" },
        },
        required: ["summary", "rationale"],
      },
    },
    {
      name: "voce_session_resume",
      description:
        "Resume a prior session — returns conversation entries, the most recent ir_snapshot as current_ir, and the last decision id. Pass session_id to target a specific session, or omit for the most recent.",
      inputSchema: {
        type: "object" as const,
        properties: {
          session_id: { type: "string", description: "Specific session id; omit for most recent" },
        },
      },
    },
    {
      name: "voce_check_drift",
      description:
        "Detect potential conflicts between a proposed IR and prior decisions. v1 is a keyword heuristic — false positives expected, judge each report. Run before declaring an IR final.",
      inputSchema: {
        type: "object" as const,
        properties: {
          proposed_ir: { type: "string", description: "Voce IR JSON to check" },
        },
        required: ["proposed_ir"],
      },
    },
    // ── Generation workflow (S65 Day 4) ───────────────────────────
    {
      name: "voce_generate_start",
      description:
        "Open a generation session. Records user_intent, returns session_id. Use this BEFORE asking any discovery questions — every subsequent phase tool needs the session_id.",
      inputSchema: {
        type: "object" as const,
        properties: {
          user_intent: { type: "string", description: "The user's initial brief, verbatim" },
        },
        required: ["user_intent"],
      },
    },
    {
      name: "voce_generate_answer",
      description:
        "Record one (question, answer) discovery turn. Pass ready: true when you have enough context to propose an IR. Server does not auto-flip ready — that's your call.",
      inputSchema: {
        type: "object" as const,
        properties: {
          session_id: { type: "string", description: "Session id from voce_generate_start" },
          question: { type: "string", description: "The question you asked the user" },
          answer: { type: "string", description: "The user's answer, verbatim" },
          ready: { type: "boolean", description: "True when discovery is complete" },
        },
        required: ["session_id", "question", "answer", "ready"],
      },
    },
    {
      name: "voce_generate_propose",
      description:
        "Submit your generated IR for review. BLOCKS if readiness < 70 — keep doing discovery first. Returns readiness, completeness pillar check, and records the IR as the session's current snapshot.",
      inputSchema: {
        type: "object" as const,
        properties: {
          session_id: { type: "string", description: "Session id" },
          ir_json: { type: "string", description: "The IR JSON you generated" },
        },
        required: ["session_id", "ir_json"],
      },
    },
    {
      name: "voce_generate_refine",
      description:
        "Apply user feedback to a proposed IR. Records the feedback turn and updates the snapshot. Run completeness checks after — the new IR has to clear the same gates as the original proposal.",
      inputSchema: {
        type: "object" as const,
        properties: {
          session_id: { type: "string", description: "Session id" },
          feedback: { type: "string", description: "The user's feedback" },
          ir_json: { type: "string", description: "Updated IR JSON" },
        },
        required: ["session_id", "feedback", "ir_json"],
      },
    },
    {
      name: "voce_generate_finalize",
      description:
        "Validate, compile, and seal the session. BLOCKS on missing pillars or any validation error — full-stack-completeness gate. Returns html, validation summary, and deployment hints.",
      inputSchema: {
        type: "object" as const,
        properties: {
          session_id: { type: "string", description: "Session id" },
        },
        required: ["session_id"],
      },
    },
  ],
}));

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case "voce_validate":
        return runVoceCommand("validate", args?.ir_json as string);
      case "voce_compile":
        return runVoceCommand("compile", args?.ir_json as string);
      case "voce_inspect":
        return runVoceCommand("inspect", args?.ir_json as string);
      case "voce_schema":
        return getSchema(args?.node_type as string | undefined);
      case "voce_examples":
        return getExamples(args?.name as string | undefined);
      case "voce_generate":
        return generateIr(args?.prompt as string);
      case "voce_brief_get":
        return briefGet();
      case "voce_brief_set":
        return briefSet(args?.brief_md as string);
      case "voce_decisions_list":
        return decisionsList(args?.since as string | undefined);
      case "voce_decisions_log":
        return decisionsLog({
          summary: args?.summary as string,
          rationale: args?.rationale as string,
          supersedes: args?.supersedes as string | undefined,
          conflicts_with: args?.conflicts_with as string | undefined,
        });
      case "voce_session_resume":
        return sessionResume(args?.session_id as string | undefined);
      case "voce_check_drift":
        return checkDrift(args?.proposed_ir as string);
      case "voce_generate_start":
        return generateStart(args?.user_intent as string);
      case "voce_generate_answer":
        return generateAnswer({
          session_id: args?.session_id as string,
          question: args?.question as string,
          answer: args?.answer as string,
          ready: args?.ready as boolean,
        });
      case "voce_generate_propose":
        return generatePropose(args?.session_id as string, args?.ir_json as string);
      case "voce_generate_refine":
        return generateRefine(
          args?.session_id as string,
          args?.feedback as string,
          args?.ir_json as string,
        );
      case "voce_generate_finalize":
        return generateFinalize(args?.session_id as string);
      default:
        return { content: [{ type: "text" as const, text: `Unknown tool: ${name}` }], isError: true };
    }
  } catch (error) {
    return {
      content: [{ type: "text" as const, text: `Error: ${(error as Error).message}` }],
      isError: true,
    };
  }
});

// ── Resources ────────────────────────────────────────────────────

server.setRequestHandler(ListResourcesRequestSchema, async () => ({
  resources: [
    {
      uri: "voce://brief",
      name: "Project Brief",
      description: "The project's north-star brief — every generation is checked against it.",
      mimeType: "text/markdown",
    },
    {
      uri: "voce://decisions",
      name: "Decision Log",
      description: "Append-only log of design / architecture decisions, oldest first.",
      mimeType: "application/jsonl",
    },
    {
      uri: "voce://drift-warnings",
      name: "Drift Warnings",
      description: "Detected conflicts between proposed IR and prior decisions.",
      mimeType: "application/jsonl",
    },
    {
      uri: "voce://status",
      name: "Project Status",
      description: "Snapshot of brief presence, decision count, and pending drift.",
      mimeType: "text/plain",
    },
  ],
}));

server.setRequestHandler(ReadResourceRequestSchema, async (request) => {
  const { uri } = request.params;

  switch (uri) {
    case "voce://brief": {
      const brief = readBrief();
      const text = brief?.content ?? "No brief yet. Use voce_brief_set to author one (S65 Day 3).";
      return { contents: [{ uri, mimeType: "text/markdown", text }] };
    }
    case "voce://decisions": {
      const lines = listDecisions().map((d) => JSON.stringify(d));
      const text = lines.length > 0 ? lines.join("\n") : "";
      return { contents: [{ uri, mimeType: "application/jsonl", text }] };
    }
    case "voce://drift-warnings": {
      const lines = listDrift().map((d) => JSON.stringify(d));
      const text = lines.length > 0 ? lines.join("\n") : "";
      return { contents: [{ uri, mimeType: "application/jsonl", text }] };
    }
    case "voce://status": {
      return { contents: [{ uri, mimeType: "text/plain", text: getProjectStatus() }] };
    }
    default:
      return { contents: [{ uri, mimeType: "text/plain", text: `Unknown resource: ${uri}` }] };
  }
});

// ── Helpers ──────────────────────────────────────────────────────

function runVoceCommand(
  command: string,
  irJson: string
): { content: Array<{ type: "text"; text: string }>; isError?: boolean } {
  const tmpFile = join(tmpdir(), `voce-mcp-${Date.now()}.voce.json`);
  writeFileSync(tmpFile, irJson);

  try {
    if (command === "compile") {
      const outFile = join(tmpdir(), `voce-mcp-${Date.now()}.html`);
      execSync(`"${VOCE_BIN}" compile "${tmpFile}" -o "${outFile}" --skip-fonts`, {
        encoding: "utf-8",
        stdio: ["pipe", "pipe", "pipe"],
      });
      const html = readFileSync(outFile, "utf-8");
      try { unlinkSync(outFile); } catch { /* ignore */ }
      return { content: [{ type: "text", text: html }] };
    }

    const formatFlag = command === "validate" ? " --format json" : "";
    const output = execSync(`"${VOCE_BIN}" ${command}${formatFlag} "${tmpFile}"`, {
      encoding: "utf-8",
      stdio: ["pipe", "pipe", "pipe"],
    });
    return { content: [{ type: "text", text: output }] };
  } catch (error: unknown) {
    const err = error as { stdout?: string; stderr?: string };
    return {
      content: [{ type: "text", text: err.stdout || err.stderr || "Command failed" }],
      isError: true,
    };
  } finally {
    try { unlinkSync(tmpFile); } catch { /* ignore */ }
  }
}

function getSchema(nodeType?: string): { content: Array<{ type: "text"; text: string }> } {
  // Read schema docs from the docs site
  const schemaDir = resolve(__dirname, "../../../docs/site/src/schema");
  if (!existsSync(schemaDir)) {
    return { content: [{ type: "text", text: "Schema docs not found. Run 'cd docs/site && mdbook build' first." }] };
  }

  if (nodeType) {
    // Try to find a matching schema doc
    const files = ["layout.md", "state.md", "motion.md", "navigation.md", "accessibility.md",
      "theming.md", "data.md", "forms.md", "seo.md", "i18n.md", "overview.md"];
    for (const f of files) {
      const path = join(schemaDir, f);
      if (existsSync(path)) {
        const content = readFileSync(path, "utf-8");
        if (content.includes(nodeType)) {
          return { content: [{ type: "text", text: content }] };
        }
      }
    }
    return { content: [{ type: "text", text: `Node type ${nodeType} not found in schema docs.` }] };
  }

  // Return the overview
  const overviewPath = join(schemaDir, "overview.md");
  const content = existsSync(overviewPath)
    ? readFileSync(overviewPath, "utf-8")
    : "Schema overview not found.";
  return { content: [{ type: "text", text: content }] };
}

function getExamples(name?: string): { content: Array<{ type: "text"; text: string }> } {
  if (name) {
    const paths = [
      `examples/landing-page/${name}.voce.json`,
      `examples/intents/${name}/ir.voce.json`,
      `examples/${name}.voce.json`,
    ];
    for (const p of paths) {
      if (existsSync(p)) {
        return { content: [{ type: "text", text: readFileSync(p, "utf-8") }] };
      }
    }
    return { content: [{ type: "text", text: `Example '${name}' not found.` }] };
  }

  // List available examples
  const examples = [
    "landing-page — Reference landing page (37 nodes, 11 types)",
    "01-hero-section — Hero with headline and CTA",
    "02-contact-form — Contact form with validation",
  ];
  return { content: [{ type: "text", text: "Available examples:\n" + examples.join("\n") }] };
}

function generateIr(prompt: string): { content: Array<{ type: "text"; text: string }> } {
  try {
    const output = execSync(`"${VOCE_BIN}" generate "${prompt.replace(/"/g, '\\"')}"`, {
      encoding: "utf-8",
      timeout: 60000,
      stdio: ["pipe", "pipe", "pipe"],
    });
    return { content: [{ type: "text", text: output }] };
  } catch (error: unknown) {
    const err = error as { stderr?: string };
    return { content: [{ type: "text", text: err.stderr || "Generation failed" }] };
  }
}

// ── Memory tools (S65 Day 3) ─────────────────────────────────────

type ToolResult = {
  content: Array<{ type: "text"; text: string }>;
  isError?: boolean;
};

function jsonResult(value: unknown): ToolResult {
  return { content: [{ type: "text", text: JSON.stringify(value, null, 2) }] };
}

function briefGet(): ToolResult {
  const brief = readBrief();
  return jsonResult({
    brief_md: brief?.content ?? null,
    last_modified: brief?.last_modified ?? null,
  });
}

function briefSet(briefMd: string): ToolResult {
  if (typeof briefMd !== "string" || briefMd.length === 0) {
    return {
      content: [{ type: "text", text: "voce_brief_set: brief_md must be a non-empty string" }],
      isError: true,
    };
  }
  writeBrief(briefMd);
  const after = readBrief();
  return jsonResult({
    written: true,
    last_modified: after?.last_modified ?? null,
    bytes: Buffer.byteLength(briefMd, "utf8"),
  });
}

function decisionsList(since?: string): ToolResult {
  return jsonResult({ decisions: listDecisions(since !== undefined ? { since } : {}) });
}

function decisionsLog(input: {
  summary: string;
  rationale: string;
  supersedes?: string;
  conflicts_with?: string;
}): ToolResult {
  if (typeof input.summary !== "string" || input.summary.length === 0) {
    return {
      content: [{ type: "text", text: "voce_decisions_log: summary is required" }],
      isError: true,
    };
  }
  if (typeof input.rationale !== "string" || input.rationale.length === 0) {
    return {
      content: [{ type: "text", text: "voce_decisions_log: rationale is required" }],
      isError: true,
    };
  }
  const decision = logDecision(input);
  return jsonResult({ id: decision.id, decision });
}

function sessionResume(sessionId?: string): ToolResult {
  const sessions = listSessions();
  const id = sessionId ?? sessions[0]?.id ?? null;
  if (id === null) {
    return jsonResult({
      session_id: null,
      messages: [],
      current_ir: null,
      last_decision_id: null,
      note: "No sessions on disk yet.",
    });
  }
  const messages = readSession(id);
  const allDecisions = listDecisions();
  return jsonResult({
    session_id: id,
    messages,
    current_ir: latestIrSnapshot(id),
    last_decision_id: allDecisions[allDecisions.length - 1]?.id ?? null,
  });
}

function checkDrift(proposedIr: string): ToolResult {
  if (typeof proposedIr !== "string" || proposedIr.length === 0) {
    return {
      content: [{ type: "text", text: "voce_check_drift: proposed_ir is required" }],
      isError: true,
    };
  }
  return jsonResult(detectDrift(proposedIr, listDecisions()));
}

// ── Generation workflow tools (S65 Day 4) ────────────────────────

function generateStart(userIntent: string): ToolResult {
  if (typeof userIntent !== "string" || userIntent.trim().length === 0) {
    return {
      content: [{ type: "text", text: "voce_generate_start: user_intent is required" }],
      isError: true,
    };
  }
  const result = startGeneration(userIntent);
  return jsonResult(result);
}

function generateAnswer(input: {
  session_id: string;
  question: string;
  answer: string;
  ready: boolean;
}): ToolResult {
  if (!input.session_id) {
    return { content: [{ type: "text", text: "voce_generate_answer: session_id is required" }], isError: true };
  }
  if (!input.question || !input.answer) {
    return { content: [{ type: "text", text: "voce_generate_answer: question and answer are required" }], isError: true };
  }
  if (typeof input.ready !== "boolean") {
    return { content: [{ type: "text", text: "voce_generate_answer: ready (boolean) is required" }], isError: true };
  }
  return jsonResult(recordAnswer(input.session_id, input.question, input.answer, input.ready));
}

function generatePropose(sessionId: string, irJson: string): ToolResult {
  if (!sessionId || !irJson) {
    return {
      content: [{ type: "text", text: "voce_generate_propose: session_id and ir_json are required" }],
      isError: true,
    };
  }
  const briefPresent = readBrief() !== null;
  const result = recordProposal(sessionId, irJson, { briefPresent });
  if (!result.ok) {
    // Soft failure — the agent gets a clear next-step message but the call
    // doesn't bubble up as an MCP error (the workflow gate is the contract,
    // not an exception).
    return jsonResult(result);
  }
  return jsonResult(result);
}

function generateRefine(sessionId: string, feedback: string, irJson: string): ToolResult {
  if (!sessionId || !feedback || !irJson) {
    return {
      content: [{ type: "text", text: "voce_generate_refine: session_id, feedback, and ir_json are required" }],
      isError: true,
    };
  }
  return jsonResult(recordRefinement(sessionId, feedback, irJson));
}

function generateFinalize(sessionId: string): ToolResult {
  if (!sessionId) {
    return {
      content: [{ type: "text", text: "voce_generate_finalize: session_id is required" }],
      isError: true,
    };
  }
  const ir = latestIrSnapshot(sessionId);
  if (ir === null) {
    return {
      content: [{ type: "text", text: "voce_generate_finalize: no proposal found — call voce_generate_propose first" }],
      isError: true,
    };
  }
  const gate = gateFinalize(sessionId, ir);
  if (!gate.ok) {
    return jsonResult(gate);
  }

  // Validate via the voce CLI. We re-use runVoceCommand which already has
  // the temp-file shuffle, but in JSON mode for structured output.
  const validation = runVoceCommand("validate", ir);
  const validationText = validation.content[0]?.text ?? "";
  let validationParsed: unknown = validationText;
  try { validationParsed = JSON.parse(validationText); } catch { /* keep raw */ }
  if (validation.isError) {
    return jsonResult({
      ok: false,
      message: "Validation failed — call voce_generate_refine with the corrected IR.",
      validation: validationParsed,
      gate,
    });
  }

  // Compile to HTML.
  const compile = runVoceCommand("compile", ir);
  if (compile.isError) {
    return jsonResult({
      ok: false,
      message: "Compile failed.",
      compile_output: compile.content[0]?.text ?? "",
      gate,
    });
  }

  recordFinalization(sessionId);

  return jsonResult({
    ok: true,
    state: getWorkflowState(sessionId),
    ir_json: ir,
    validation: validationParsed,
    html: compile.content[0]?.text ?? "",
    deployment_hints: [
      "voce-adapter-vercel — git-push deploy with Edge runtime",
      "voce-adapter-netlify — Netlify Functions + CDN",
      "voce-adapter-cloudflare — Workers + Pages",
      "voce-adapter-static — single-file HTML for any static host",
    ],
  });
}

function getProjectStatus(): string {
  const lines: string[] = ["Voce IR Project Status\n"];

  const brief = readBrief();
  if (brief) {
    lines.push(`Brief: present (${brief.content.length} chars, last modified ${brief.last_modified})`);
  } else {
    lines.push("Brief: not authored (use voce_brief_set)");
  }

  const decisions = listDecisions();
  lines.push(`Decisions: ${decisions.length} recorded`);

  const drift = listDrift();
  const pending = drift.filter((d) => d.resolution === "pending").length;
  lines.push(`Drift warnings: ${drift.length} total, ${pending} pending`);

  return lines.join("\n");
}

// ── Start ────────────────────────────────────────────────────────

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch(console.error);
