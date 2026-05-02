// MCP integration test — drives the server through an in-memory transport
// pair so the full request/response surface (tools + resources) is exercised
// the same way a real MCP client would. Sister to the unit tests; this is
// what catches handler-level regressions that pass each module in isolation.

import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { execSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";

import { server } from "./index.js";

let workspace: string;
let client: Client;
let voceAvailable: boolean;

function hasVoce(): boolean {
  try {
    execSync("voce --version", { stdio: "pipe" });
    return true;
  } catch {
    return false;
  }
}

beforeEach(async () => {
  workspace = mkdtempSync(join(tmpdir(), "voce-mcp-integration-"));
  process.env.VOCE_PROJECT_ROOT = workspace;
  voceAvailable = hasVoce();

  const [serverTransport, clientTransport] = InMemoryTransport.createLinkedPair();
  // Keep server connection alive across the suite by attaching transport
  // here. The server module is shared (singleton) — fine for integration
  // tests because each test resets the .voce/ workspace.
  await server.connect(serverTransport);

  client = new Client({ name: "voce-test-client", version: "0.0.0" }, { capabilities: {} });
  await client.connect(clientTransport);
});

afterEach(async () => {
  await client.close();
  await server.close();
  rmSync(workspace, { recursive: true, force: true });
  delete process.env.VOCE_PROJECT_ROOT;
});

// Helper — call a tool and parse its JSON text result.
async function callJson(name: string, args: Record<string, unknown>): Promise<unknown> {
  const result = await client.callTool({ name, arguments: args });
  const first = (result.content as Array<{ type: string; text: string }>)[0];
  if (!first || first.type !== "text") throw new Error(`${name}: no text content`);
  try {
    return JSON.parse(first.text);
  } catch {
    return first.text;
  }
}

describe("MCP server — discovery surface", () => {
  it("lists every tool we shipped", async () => {
    const { tools } = await client.listTools();
    const names = tools.map((t) => t.name).sort();
    expect(names).toEqual([
      "voce_brief_get",
      "voce_brief_set",
      "voce_check_drift",
      "voce_compile",
      "voce_decisions_list",
      "voce_decisions_log",
      "voce_examples",
      "voce_feature_completeness",
      "voce_generate",
      "voce_generate_answer",
      "voce_generate_finalize",
      "voce_generate_propose",
      "voce_generate_refine",
      "voce_generate_start",
      "voce_generation_readiness",
      "voce_inspect",
      "voce_schema",
      "voce_session_resume",
      "voce_validate",
    ]);
  });

  it("lists every resource we shipped", async () => {
    const { resources } = await client.listResources();
    const uris = resources.map((r) => r.uri).sort();
    expect(uris).toEqual([
      "voce://brief",
      "voce://decisions",
      "voce://drift-warnings",
      "voce://status",
    ]);
  });
});

describe("MCP server — memory tools", () => {
  it("brief_set then brief_get round-trips through the .voce/ store", async () => {
    const setRes = (await callJson("voce_brief_set", { brief_md: "# Test\n\nHello" })) as {
      written: boolean; bytes: number;
    };
    expect(setRes.written).toBe(true);
    expect(setRes.bytes).toBeGreaterThan(0);

    const getRes = (await callJson("voce_brief_get", {})) as { brief_md: string };
    expect(getRes.brief_md).toBe("# Test\n\nHello");
  });

  it("decisions_log then decisions_list returns the new entry", async () => {
    const log = (await callJson("voce_decisions_log", {
      summary: "Use dark mode by default",
      rationale: "Brand alignment + battery savings",
    })) as { id: string };
    expect(log.id).toMatch(/^[0-9a-f-]{36}$/);

    const list = (await callJson("voce_decisions_list", {})) as { decisions: Array<{ id: string }> };
    expect(list.decisions).toHaveLength(1);
    expect(list.decisions[0]?.id).toBe(log.id);
  });

  it("check_drift surfaces decisions whose terms appear in the IR", async () => {
    await callJson("voce_decisions_log", {
      summary: "no Modal nodes",
      rationale: "Modals trap focus and hurt accessibility",
    });
    const result = (await callJson("voce_check_drift", {
      proposed_ir: JSON.stringify({ value_type: "Modal" }),
    })) as { drift: unknown[] };
    expect(result.drift).toHaveLength(1);
  });
});

describe("MCP server — generation workflow", () => {
  it("blocks propose at low readiness then accepts after discovery + ready", async () => {
    const start = (await callJson("voce_generate_start", {
      user_intent: "B2B SaaS platform for coffee roasters",
    })) as { session_id: string };

    // Premature propose — only 0 turns, ready false.
    const blocked = (await callJson("voce_generate_propose", {
      session_id: start.session_id,
      ir_json: "{}",
    })) as { ok: boolean; message?: string };
    expect(blocked.ok).toBe(false);
    expect(blocked.message ?? "").toMatch(/Readiness score/);

    // Walk discovery to readiness 100 (5 turns, last ready: true).
    for (let i = 0; i < 5; i++) {
      await callJson("voce_generate_answer", {
        session_id: start.session_id,
        question: `q${i}`,
        answer: `a${i}`,
        ready: i === 4,
      });
    }

    const accepted = (await callJson("voce_generate_propose", {
      session_id: start.session_id,
      ir_json: completeIr(),
    })) as { ok: boolean; readiness: { score: number } };
    expect(accepted.ok).toBe(true);
    expect(accepted.readiness.score).toBeGreaterThanOrEqual(70);
  });

  it("generation_readiness mirrors propose's gate", async () => {
    const start = (await callJson("voce_generate_start", {
      user_intent: "x",
    })) as { session_id: string };
    const score = (await callJson("voce_generation_readiness", {
      session_id: start.session_id,
    })) as { score: number; ready: boolean };
    expect(score.score).toBe(20); // user_intent only
    expect(score.ready).toBe(false);
  });

  it("feature_completeness flags the same pillars as the workflow gate", async () => {
    const incomplete = JSON.stringify({
      value_type: "ViewRoot",
      children: [{ value_type: "FormNode", node_id: "form" }],
    });
    const result = (await callJson("voce_feature_completeness", {
      ir_json: incomplete,
    })) as { complete: boolean; missing_pillars: string[] };
    expect(result.complete).toBe(false);
    expect(result.missing_pillars.length).toBeGreaterThan(0);
  });

  it("session_resume returns the most recent ir_snapshot", async () => {
    const start = (await callJson("voce_generate_start", {
      user_intent: "x",
    })) as { session_id: string };
    for (let i = 0; i < 5; i++) {
      await callJson("voce_generate_answer", {
        session_id: start.session_id,
        question: `q${i}`,
        answer: `a${i}`,
        ready: i === 4,
      });
    }
    const ir = completeIr({ tag: "v1" });
    await callJson("voce_generate_propose", {
      session_id: start.session_id,
      ir_json: ir,
    });
    const resumed = (await callJson("voce_session_resume", {
      session_id: start.session_id,
    })) as { current_ir: string };
    expect(resumed.current_ir).toBe(ir);
  });

  it("finalize gates on validation when voce CLI is available", async () => {
    if (!voceAvailable) {
      // Skip the end-to-end leg locally if the CLI isn't installed; the gate
      // logic itself is covered by the workflow unit tests.
      return;
    }

    const start = (await callJson("voce_generate_start", {
      user_intent: "x",
    })) as { session_id: string };
    for (let i = 0; i < 5; i++) {
      await callJson("voce_generate_answer", {
        session_id: start.session_id,
        question: `q${i}`,
        answer: `a${i}`,
        ready: i === 4,
      });
    }
    // Use a deliberately incomplete IR — validate WILL fail on it.
    const badIr = JSON.stringify({
      value_type: "ViewRoot",
      children: [],
    });
    await callJson("voce_generate_propose", {
      session_id: start.session_id,
      ir_json: badIr,
    });
    const result = await callJson("voce_generate_finalize", {
      session_id: start.session_id,
    });
    // Either completeness blocks it before validate, or validate fails.
    // Both produce ok: false. The point is we never silently finalize bad IR.
    expect((result as { ok: boolean }).ok).toBe(false);
  });
});

interface IrOpts { tag?: string }

function completeIr(opts: IrOpts = {}): string {
  return JSON.stringify({
    value_type: "ViewRoot",
    metadata: { value_type: "PageMetadata", title: opts.tag ?? "Voce" },
    children: [
      {
        value_type: "FormNode",
        node_id: "form",
        semantic_node_id: "sem-form",
        validation_rules: [{ value_type: "ValidationRule", rule: "required" }],
        children: [
          {
            value_type: "ActionNode",
            node_id: "submit",
            semantic_node_id: "sem-submit",
            error_state: "Sorry, try again",
            loading_state: "Sending…",
          },
        ],
      },
    ],
  });
}
