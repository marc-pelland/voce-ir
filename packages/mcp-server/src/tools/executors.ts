// Tool executors — each Voce tool's implementation. Pure functions where
// possible; the few that shell out to the `voce` CLI (validate, compile,
// inspect, generate) do their fs shuffle here. The dispatch table at the
// bottom is the public entry — both the MCP server and the cli-chat
// tool-use loop call into it.

import { execSync } from "node:child_process";
import { existsSync, readFileSync, unlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import {
  detectDrift,
  latestIrSnapshot,
  listDecisions,
  listSessions,
  logDecision,
  readBrief,
  readSession,
  writeBrief,
} from "../memory/index.js";
import {
  gateFinalize,
  getWorkflowState,
  recordAnswer,
  recordFinalization,
  recordProposal,
  recordRefinement,
  scoreCompleteness,
  scoreReadiness,
  startGeneration,
} from "../workflow/index.js";

import type { ToolResult } from "./types.js";

// ── voce binary resolution ─────────────────────────────────────

const __dirname = dirname(fileURLToPath(import.meta.url));

function findVoceBinary(): string {
  // Resolve relative to the runtime location of this module. dist/ lives
  // inside packages/mcp-server, so the workspace root is three levels up.
  const candidates = [
    resolve(__dirname, "../../../target/release/voce"),
    resolve(__dirname, "../../../target/debug/voce"),
    resolve(__dirname, "../../../../target/release/voce"),
    resolve(__dirname, "../../../../target/debug/voce"),
  ];
  for (const c of candidates) {
    if (existsSync(c)) return c;
  }
  return "voce"; // fall back to PATH
}

const VOCE_BIN = findVoceBinary();

// ── Result helpers ─────────────────────────────────────────────

function jsonResult(value: unknown): ToolResult {
  return { content: [{ type: "text", text: JSON.stringify(value, null, 2) }] };
}

function errorResult(text: string): ToolResult {
  return { content: [{ type: "text", text }], isError: true };
}

// ── Pipeline tools (shell-out to voce CLI) ─────────────────────

function runVoceCommand(command: string, irJson: string): ToolResult {
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

function getSchema(nodeType?: string): ToolResult {
  const schemaDir = resolve(__dirname, "../../../docs/site/src/schema");
  if (!existsSync(schemaDir)) {
    return { content: [{ type: "text", text: "Schema docs not found. Run 'cd docs/site && mdbook build' first." }] };
  }

  if (nodeType) {
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

  const overviewPath = join(schemaDir, "overview.md");
  const content = existsSync(overviewPath)
    ? readFileSync(overviewPath, "utf-8")
    : "Schema overview not found.";
  return { content: [{ type: "text", text: content }] };
}

function getExamples(name?: string): ToolResult {
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

  const examples = [
    "landing-page — Reference landing page (37 nodes, 11 types)",
    "01-hero-section — Hero with headline and CTA",
    "02-contact-form — Contact form with validation",
  ];
  return { content: [{ type: "text", text: "Available examples:\n" + examples.join("\n") }] };
}

function generateIr(prompt: string): ToolResult {
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

// ── Memory tools ───────────────────────────────────────────────

function briefGet(): ToolResult {
  const brief = readBrief();
  return jsonResult({
    brief_md: brief?.content ?? null,
    last_modified: brief?.last_modified ?? null,
  });
}

function briefSet(briefMd: string): ToolResult {
  if (typeof briefMd !== "string" || briefMd.length === 0) {
    return errorResult("voce_brief_set: brief_md must be a non-empty string");
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
    return errorResult("voce_decisions_log: summary is required");
  }
  if (typeof input.rationale !== "string" || input.rationale.length === 0) {
    return errorResult("voce_decisions_log: rationale is required");
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
    return errorResult("voce_check_drift: proposed_ir is required");
  }
  return jsonResult(detectDrift(proposedIr, listDecisions()));
}

// ── Generation workflow ────────────────────────────────────────

function generateStart(userIntent: string): ToolResult {
  if (typeof userIntent !== "string" || userIntent.trim().length === 0) {
    return errorResult("voce_generate_start: user_intent is required");
  }
  return jsonResult(startGeneration(userIntent));
}

function generateAnswer(input: {
  session_id: string;
  question: string;
  answer: string;
  ready: boolean;
}): ToolResult {
  if (!input.session_id) return errorResult("voce_generate_answer: session_id is required");
  if (!input.question || !input.answer) {
    return errorResult("voce_generate_answer: question and answer are required");
  }
  if (typeof input.ready !== "boolean") {
    return errorResult("voce_generate_answer: ready (boolean) is required");
  }
  return jsonResult(recordAnswer(input.session_id, input.question, input.answer, input.ready));
}

function generatePropose(sessionId: string, irJson: string): ToolResult {
  if (!sessionId || !irJson) {
    return errorResult("voce_generate_propose: session_id and ir_json are required");
  }
  const briefPresent = readBrief() !== null;
  return jsonResult(recordProposal(sessionId, irJson, { briefPresent }));
}

function generateRefine(sessionId: string, feedback: string, irJson: string): ToolResult {
  if (!sessionId || !feedback || !irJson) {
    return errorResult("voce_generate_refine: session_id, feedback, and ir_json are required");
  }
  return jsonResult(recordRefinement(sessionId, feedback, irJson));
}

function generateFinalize(sessionId: string): ToolResult {
  if (!sessionId) return errorResult("voce_generate_finalize: session_id is required");
  const ir = latestIrSnapshot(sessionId);
  if (ir === null) {
    return errorResult("voce_generate_finalize: no proposal found — call voce_generate_propose first");
  }
  const gate = gateFinalize(sessionId, ir);
  if (!gate.ok) return jsonResult(gate);

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

// ── Quality gates ──────────────────────────────────────────────

function generationReadiness(sessionId: string): ToolResult {
  if (!sessionId) return errorResult("voce_generation_readiness: session_id is required");
  const state = getWorkflowState(sessionId);
  if (state.phase === "not_started") {
    return errorResult("voce_generation_readiness: session not found");
  }
  return jsonResult(scoreReadiness(state, { briefPresent: readBrief() !== null }));
}

function featureCompleteness(irJson: string): ToolResult {
  if (typeof irJson !== "string" || irJson.length === 0) {
    return errorResult("voce_feature_completeness: ir_json is required");
  }
  return jsonResult(scoreCompleteness(irJson));
}

// ── Dispatcher ─────────────────────────────────────────────────

/**
 * Execute a Voce tool by name. Errors are returned as ToolResult with
 * isError: true rather than thrown — callers (MCP transport, cli-chat
 * tool-use loop) hand the result straight to the model so it can react.
 */
export function executeTool(
  name: string,
  args: Record<string, unknown> | undefined,
): ToolResult {
  try {
    const a = args ?? {};
    switch (name) {
      case "voce_validate":
        return runVoceCommand("validate", a.ir_json as string);
      case "voce_compile":
        return runVoceCommand("compile", a.ir_json as string);
      case "voce_inspect":
        return runVoceCommand("inspect", a.ir_json as string);
      case "voce_schema":
        return getSchema(a.node_type as string | undefined);
      case "voce_examples":
        return getExamples(a.name as string | undefined);
      case "voce_generate":
        return generateIr(a.prompt as string);
      case "voce_brief_get":
        return briefGet();
      case "voce_brief_set":
        return briefSet(a.brief_md as string);
      case "voce_decisions_list":
        return decisionsList(a.since as string | undefined);
      case "voce_decisions_log":
        return decisionsLog({
          summary: a.summary as string,
          rationale: a.rationale as string,
          supersedes: a.supersedes as string | undefined,
          conflicts_with: a.conflicts_with as string | undefined,
        });
      case "voce_session_resume":
        return sessionResume(a.session_id as string | undefined);
      case "voce_check_drift":
        return checkDrift(a.proposed_ir as string);
      case "voce_generate_start":
        return generateStart(a.user_intent as string);
      case "voce_generate_answer":
        return generateAnswer({
          session_id: a.session_id as string,
          question: a.question as string,
          answer: a.answer as string,
          ready: a.ready as boolean,
        });
      case "voce_generate_propose":
        return generatePropose(a.session_id as string, a.ir_json as string);
      case "voce_generate_refine":
        return generateRefine(
          a.session_id as string,
          a.feedback as string,
          a.ir_json as string,
        );
      case "voce_generate_finalize":
        return generateFinalize(a.session_id as string);
      case "voce_generation_readiness":
        return generationReadiness(a.session_id as string);
      case "voce_feature_completeness":
        return featureCompleteness(a.ir_json as string);
      default:
        return errorResult(`Unknown tool: ${name}`);
    }
  } catch (error) {
    return errorResult(`Error: ${(error as Error).message}`);
  }
}
