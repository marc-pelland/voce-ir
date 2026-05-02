#!/usr/bin/env node
/**
 * Voce IR MCP Server — exposes Voce IR tools for Claude Code and MCP clients.
 *
 * Tools:
 * - voce_validate: Validate an IR file
 * - voce_compile: Compile IR to HTML
 * - voce_inspect: Get IR structure summary
 * - voce_schema: Get the Voce IR JSON schema
 * - voce_examples: List/retrieve example IR files
 * - voce_generate: Generate IR from natural language
 *
 * Resources:
 * - voce://brief — current project brief
 * - voce://status — project health status
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
import { readFileSync, existsSync, readdirSync, writeFileSync, unlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

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
        "Generate IR from natural language. Voce rejects vibe-coding — do NOT call with a vague brief. Discovery first: ask one question at a time, building context. Result must pass voce_validate before final.",
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
      description: "The current project brief (north star)",
      mimeType: "text/yaml",
    },
    {
      uri: "voce://status",
      name: "Project Status",
      description: "Current project health: brief, decisions, drift score",
      mimeType: "text/plain",
    },
  ],
}));

server.setRequestHandler(ReadResourceRequestSchema, async (request) => {
  const { uri } = request.params;

  switch (uri) {
    case "voce://brief": {
      const briefPath = ".voce/brief.yaml";
      const content = existsSync(briefPath)
        ? readFileSync(briefPath, "utf-8")
        : "No brief found. Run 'voce design' to create one.";
      return { contents: [{ uri, mimeType: "text/yaml", text: content }] };
    }
    case "voce://status": {
      const status = getProjectStatus();
      return { contents: [{ uri, mimeType: "text/plain", text: status }] };
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

function getProjectStatus(): string {
  const lines: string[] = ["Voce IR Project Status\n"];

  if (existsSync(".voce/brief.yaml")) {
    lines.push("Brief: found (.voce/brief.yaml)");
  } else {
    lines.push("Brief: not created (run 'voce design')");
  }

  const decDir = ".voce/decisions";
  if (existsSync(decDir)) {
    const count = readdirSync(decDir).filter((f: string) => f.endsWith(".yaml")).length;
    lines.push(`Decisions: ${count} recorded`);
  }

  return lines.join("\n");
}

// ── Start ────────────────────────────────────────────────────────

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch(console.error);
