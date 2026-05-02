#!/usr/bin/env node
/**
 * Voce IR MCP Server — exposes Voce IR tools for any MCP client.
 *
 * Tool definitions and executors live in ./tools — same module the
 * @voce-ir/cli-chat tool-use loop consumes. This file is a thin
 * transport shim: stdio in, MCP framing, tool dispatch + resources.
 *
 * Pipeline / memory / workflow / quality gates: see ./tools/definitions.ts.
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

import { listDecisions, listDrift, readBrief } from "./memory/index.js";
import { TOOL_DEFINITIONS, executeTool } from "./tools/index.js";

const server = new Server(
  { name: "voce-ir", version: "0.3.0" },
  { capabilities: { tools: {}, resources: {} } },
);

// ── Tools ────────────────────────────────────────────────────────

server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: TOOL_DEFINITIONS.map((t) => ({
    name: t.name,
    description: t.description,
    inputSchema: t.inputSchema,
  })),
}));

server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;
  // The SDK's CallTool result type is a tagged union of ContentResult and
  // TaskStartedResult — our ToolResult matches the ContentResult branch but
  // TS can't pick the branch from a named interface. The cast is safe and
  // narrow.
  return executeTool(name, args) as unknown as Awaited<
    ReturnType<Parameters<typeof server.setRequestHandler>[1]>
  >;
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
      return { contents: [{ uri, mimeType: "application/jsonl", text: lines.join("\n") }] };
    }
    case "voce://drift-warnings": {
      const lines = listDrift().map((d) => JSON.stringify(d));
      return { contents: [{ uri, mimeType: "application/jsonl", text: lines.join("\n") }] };
    }
    case "voce://status": {
      return { contents: [{ uri, mimeType: "text/plain", text: getProjectStatus() }] };
    }
    default:
      return { contents: [{ uri, mimeType: "text/plain", text: `Unknown resource: ${uri}` }] };
  }
});

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

// ── Exports + entry ──────────────────────────────────────────────

/** Exposed so tests (and any embedder) can attach a non-stdio transport. */
export { server };

async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
}

// Run main() only when this file is the process entry point. Importing the
// module — which the integration tests do — must not steal stdio.
const isMain = import.meta.url === `file://${process.argv[1]}`;
if (isMain) {
  main().catch((err) => {
    // eslint-disable-next-line no-console -- entry point, no logger available
    console.error(err);
    process.exit(1);
  });
}
