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
import { readFileSync, existsSync } from "node:fs";

const server = new Server(
  { name: "voce-ir", version: "0.3.0" },
  { capabilities: { tools: {}, resources: {} } }
);

// ── Tools ────────────────────────────────────────────────────────

server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [
    {
      name: "voce_validate",
      description: "Validate a Voce IR JSON file against all quality rules (a11y, security, SEO, forms, etc.)",
      inputSchema: {
        type: "object" as const,
        properties: {
          ir_json: { type: "string", description: "Voce IR JSON content to validate" },
        },
        required: ["ir_json"],
      },
    },
    {
      name: "voce_compile",
      description: "Compile Voce IR JSON to HTML output. Returns the compiled HTML string.",
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
      description: "Get a structured summary of a Voce IR document (node counts, types, features)",
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
      description: "Get the Voce IR schema documentation for a specific node type or all types",
      inputSchema: {
        type: "object" as const,
        properties: {
          node_type: { type: "string", description: "Node type to get schema for (e.g., 'Container', 'FormNode'). Omit for full schema." },
        },
      },
    },
    {
      name: "voce_examples",
      description: "List available example IR files or retrieve a specific one",
      inputSchema: {
        type: "object" as const,
        properties: {
          name: { type: "string", description: "Example name to retrieve (e.g., 'landing-page'). Omit to list all." },
        },
      },
    },
    {
      name: "voce_generate",
      description: "Generate Voce IR from a natural language description. Requires ANTHROPIC_API_KEY.",
      inputSchema: {
        type: "object" as const,
        properties: {
          prompt: { type: "string", description: "Natural language description of what to build" },
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
  const { writeFileSync, unlinkSync } = require("node:fs");
  const { tmpdir } = require("node:os");
  const { join } = require("node:path");

  const tmpFile = join(tmpdir(), `voce-mcp-${Date.now()}.voce.json`);
  writeFileSync(tmpFile, irJson);

  try {
    let cmd: string;
    if (command === "compile") {
      const outFile = join(tmpdir(), `voce-mcp-${Date.now()}.html`);
      execSync(`voce compile "${tmpFile}" -o "${outFile}"`, { encoding: "utf-8", stdio: ["pipe", "pipe", "pipe"] });
      const html = readFileSync(outFile, "utf-8");
      unlinkSync(outFile);
      return { content: [{ type: "text", text: html }] };
    }

    cmd = `voce ${command} --format json "${tmpFile}"`;
    const output = execSync(cmd, { encoding: "utf-8", stdio: ["pipe", "pipe", "pipe"] });
    return { content: [{ type: "text", text: output }] };
  } catch (error: unknown) {
    const err = error as { stdout?: string; stderr?: string };
    return { content: [{ type: "text", text: err.stdout || err.stderr || "Command failed" }], isError: true };
  } finally {
    try { unlinkSync(tmpFile); } catch { /* ignore */ }
  }
}

function getSchema(nodeType?: string): { content: Array<{ type: "text"; text: string }> } {
  // Return the schema context that the AI bridge uses
  const { buildSchemaContext } = require("@voce-ir/ai-bridge");
  const context = buildSchemaContext();

  if (nodeType) {
    // Extract the relevant section for the requested type
    const lines = context.split("\n");
    const relevant = lines.filter((l: string) =>
      l.includes(nodeType) || l.includes(`**${nodeType}**`)
    );
    return { content: [{ type: "text", text: relevant.join("\n") || `Node type ${nodeType} not found in schema.` }] };
  }

  return { content: [{ type: "text", text: context }] };
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
    const output = execSync(`voce generate "${prompt.replace(/"/g, '\\"')}"`, {
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
    const count = require("node:fs").readdirSync(decDir).filter((f: string) => f.endsWith(".yaml")).length;
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
