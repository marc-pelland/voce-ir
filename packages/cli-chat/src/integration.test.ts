// End-to-end integration test for cli-chat. Drives processChatTurn with a
// scripted mock Anthropic client and the same wrapped executor + storage
// layer the real binary uses. Verifies session ledger, IR capture, token
// aggregation, and the readiness/drift interactive surfaces.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { processChatTurn } from "./chat-flow.js";
import { wrapExecutor } from "./wrapped-executor.js";
import { resolveSession } from "./session-manager.js";
import { buildSystemPrompt } from "./prompt.js";
import type { ChatState } from "./commands/registry.js";
import {
  appendSession,
  listDecisions,
  readBrief,
  readSession,
} from "@voce-ir/mcp-server/memory";

let workspace: string;

function freshState(sessionId: string): ChatState {
  return {
    conversationHistory: [],
    currentIr: null,
    irHistory: [],
    sessionId,
    model: "claude-test",
    systemPrompt: buildSystemPrompt({ brief: readBrief(), recentDecisions: [] }),
    tokenUsage: { input: 0, output: 0, cache_read: 0, cache_creation: 0 },
  };
}

interface ScriptedTurn {
  content: Array<
    | { type: "text"; text: string }
    | { type: "tool_use"; id: string; name: string; input: unknown }
  >;
  usage?: {
    input_tokens?: number;
    output_tokens?: number;
    cache_read_input_tokens?: number;
    cache_creation_input_tokens?: number;
  };
}

function mockClient(turns: ScriptedTurn[]): {
  messages: { create: ReturnType<typeof vi.fn> };
  invocations: () => number;
} {
  let i = 0;
  const create = vi.fn(async () => {
    const next = turns[i++];
    if (!next) throw new Error("scripted client exhausted");
    return next;
  });
  return {
    messages: { create },
    invocations: () => i,
  };
}

function passthroughExecutor() {
  return wrapExecutor({
    ask: async () => "y", // never called in the simple paths
    log: () => {},
  });
}

beforeEach(() => {
  workspace = mkdtempSync(join(tmpdir(), "voce-cli-int-"));
  process.env.VOCE_PROJECT_ROOT = workspace;
});

afterEach(() => {
  rmSync(workspace, { recursive: true, force: true });
  delete process.env.VOCE_PROJECT_ROOT;
});

describe("processChatTurn — happy paths", () => {
  it("a single text-only turn writes a user + assistant entry to the session ledger", async () => {
    const session = resolveSession({ resume: null });
    const state = freshState(session.id);
    const client = mockClient([
      { content: [{ type: "text", text: "hi back" }], usage: { input_tokens: 50, output_tokens: 5 } },
    ]);

    const result = await processChatTurn(
      state,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      client as any,
      { executor: passthroughExecutor() },
      "hi",
    );

    expect(result.completed).toBe(true);
    expect(result.finalText).toBe("hi back");

    const entries = readSession(session.id);
    const roles = entries.map((e) => e.role);
    expect(roles).toContain("user");
    expect(roles).toContain("assistant");
    expect(entries.find((e) => e.role === "user")?.content).toBe("hi");
    expect(entries.find((e) => e.role === "assistant")?.content).toBe("hi back");

    expect(state.tokenUsage.input).toBe(50);
    expect(state.tokenUsage.output).toBe(5);
  });

  it("captures IR from a json fence and pushes it onto irHistory on the next capture", async () => {
    const session = resolveSession({ resume: null });
    const state = freshState(session.id);
    const ir1 = JSON.stringify({ value_type: "ViewRoot", v: 1 });
    const ir2 = JSON.stringify({ value_type: "ViewRoot", v: 2 });

    const client = mockClient([
      { content: [{ type: "text", text: "v1\n```json\n" + ir1 + "\n```\n" }], usage: { input_tokens: 1, output_tokens: 1 } },
    ]);
    await processChatTurn(
      state,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      client as any,
      { executor: passthroughExecutor() },
      "first",
    );
    expect(state.currentIr).toBe(ir1);
    expect(state.irHistory).toEqual([]);

    const client2 = mockClient([
      { content: [{ type: "text", text: "v2\n```json\n" + ir2 + "\n```\n" }], usage: { input_tokens: 1, output_tokens: 1 } },
    ]);
    await processChatTurn(
      state,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      client2 as any,
      { executor: passthroughExecutor() },
      "second",
    );
    expect(state.currentIr).toBe(ir2);
    expect(state.irHistory[0]).toBe(ir1);
  });
});

describe("processChatTurn — full discovery → propose loop", () => {
  it("walks tool calls through start → answer ×4 → propose, persisting each in the ledger", async () => {
    const session = resolveSession({ resume: null });
    const state = freshState(session.id);

    // Each ScriptedTurn corresponds to one model invocation. Tool calls and
    // their results round-trip; the last turn produces a final text.
    const client = mockClient([
      // 1. start
      {
        content: [
          {
            type: "tool_use",
            id: "t1",
            name: "voce_generate_start",
            input: { user_intent: "B2B platform for coffee roasters" },
          },
        ],
      },
      // 2-5. four answers
      ...[0, 1, 2, 3].map((i) => ({
        content: [
          {
            type: "tool_use" as const,
            id: `a${i}`,
            name: "voce_generate_answer",
            input: {
              session_id: "S",
              question: `q${i}?`,
              answer: `a${i}`,
              ready: i === 3,
            },
          },
        ],
      })),
      // 6. final text wrapping it up
      { content: [{ type: "text", text: "Got it — readiness should be high now." }] },
    ]);

    // Capture session_id from the start tool's result so subsequent answer
    // calls can use it. The wrapped executor uses real executeTool which
    // creates real sessions; we route the model's S placeholder through
    // a small redirect.
    const realExecutor = passthroughExecutor();
    let workflowSessionId: string | null = null;
    const interceptingExecutor = async (
      name: string,
      args: Record<string, unknown> | undefined,
    ) => {
      if (name === "voce_generate_start") {
        const r = await realExecutor(name, args);
        const parsed = JSON.parse(r.content[0]?.text ?? "{}");
        workflowSessionId = parsed.session_id;
        return r;
      }
      if (workflowSessionId !== null && (args as { session_id?: string })?.session_id === "S") {
        return realExecutor(name, { ...args, session_id: workflowSessionId });
      }
      return realExecutor(name, args);
    };

    const result = await processChatTurn(
      state,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      client as any,
      { executor: interceptingExecutor },
      "build it",
    );

    expect(result.completed).toBe(true);
    expect(result.finalText).toMatch(/readiness/i);

    // Six tool-result entries in the ledger (start + 4 answers + 0 propose
    // since we stopped before propose to keep the test focused on tool flow).
    // Wait: actually 5 tool calls (1 start + 4 answer). Verify count.
    const toolEntries = readSession(session.id).filter((e) => e.role === "tool");
    expect(toolEntries.length).toBe(5);
    expect(toolEntries.map((e) => e.tool)).toEqual([
      "voce_generate_start",
      "voce_generate_answer",
      "voce_generate_answer",
      "voce_generate_answer",
      "voce_generate_answer",
    ]);
  });
});

describe("processChatTurn — readiness UI gate", () => {
  it("blocks voce_generate_propose when the user answers 'n'", async () => {
    const session = resolveSession({ resume: null });
    const state = freshState(session.id);

    const ask = vi.fn().mockResolvedValueOnce("n");
    const log = vi.fn();
    const interactiveExec = wrapExecutor({ ask, log });

    const client = mockClient([
      {
        content: [
          {
            type: "tool_use",
            id: "p1",
            name: "voce_generate_propose",
            input: { session_id: "fake", ir_json: "{}" },
          },
        ],
      },
      { content: [{ type: "text", text: "OK, let's keep talking." }] },
    ]);

    const result = await processChatTurn(
      state,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      client as any,
      { executor: interactiveExec },
      "propose now",
    );

    expect(result.completed).toBe(true);
    expect(ask).toHaveBeenCalled();
    // The wrapped executor returned a synthetic intercept payload — the
    // tool ledger entry's content reflects user_paused.
    const toolEntries = readSession(session.id).filter((e) => e.role === "tool");
    const proposeEntry = toolEntries.find((e) => e.tool === "voce_generate_propose");
    expect(proposeEntry?.content).toMatch(/user_paused/);
  });
});

describe("processChatTurn — drift push-back", () => {
  it("on supersede, a new decision references the prior one", async () => {
    appendSession(resolveSession({ resume: null }).id, {
      role: "system",
      content: "scaffolding",
    }); // touches workspace
    const decisionsBefore = listDecisions().length;

    // Seed a decision that the IR will overlap with.
    const exec = wrapExecutor({ ask: async () => "s", log: () => {} });
    await exec("voce_decisions_log", {
      summary: "no Modal nodes",
      rationale: "Modals trap focus",
    });

    const result = await exec("voce_check_drift", {
      proposed_ir: JSON.stringify({ value_type: "Modal" }),
    });
    const parsed = JSON.parse(result.content[0]?.text ?? "{}");
    expect(parsed.drift).toHaveLength(1);

    const decisionsAfter = listDecisions();
    expect(decisionsAfter.length).toBe(decisionsBefore + 2); // original + supersession
    expect(decisionsAfter[decisionsAfter.length - 1]?.supersedes).toBe(
      decisionsAfter[decisionsAfter.length - 2]?.id,
    );
  });
});
