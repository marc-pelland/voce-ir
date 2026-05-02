// Per-command tests. Each handler receives a stub context; assertions
// inspect either the side-effect logs or the state mutation.

import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import type { ChatState, CommandContext } from "./registry.js";
import { clearCommand, costCommand, modelCommand, undoCommand } from "./state.js";
import { diffCommand, irCommand, loadCommand, pushIrHistory, saveCommand, showCommand, summarizeDiff } from "./ir.js";
import { briefCommand, decisionCommand, decisionsCommand, explainCommand } from "./memory.js";
import { logDecision, writeBrief } from "@voce-ir/mcp-server/memory";

let workspace: string;

function freshState(): ChatState {
  return {
    conversationHistory: [],
    currentIr: null,
    irHistory: [],
    sessionId: "test-session",
    model: "claude-test",
    systemPrompt: "",
    tokenUsage: { input: 0, output: 0, cache_read: 0, cache_creation: 0 },
  };
}

function ctxWith(state: ChatState): CommandContext & { logs: string[] } {
  const logs: string[] = [];
  const ctx = {
    state,
    client: null,
    voceBin: "voce",
    log: (m: string) => logs.push(m),
    exit: () => {},
    logs,
  };
  return ctx;
}

beforeEach(() => {
  workspace = mkdtempSync(join(tmpdir(), "voce-cmd-test-"));
  process.env.VOCE_PROJECT_ROOT = workspace;
});

afterEach(() => {
  rmSync(workspace, { recursive: true, force: true });
  delete process.env.VOCE_PROJECT_ROOT;
});

describe("state commands", () => {
  it("/clear empties conversation history and IR state", async () => {
    const state = freshState();
    state.conversationHistory.push({ role: "user", content: [{ type: "text", text: "x" }] });
    state.currentIr = "{}";
    state.irHistory.push("{}");
    const ctx = ctxWith(state);
    await clearCommand.handler("", ctx);
    expect(state.conversationHistory).toEqual([]);
    expect(state.currentIr).toBeNull();
    expect(state.irHistory).toEqual([]);
  });

  it("/model with no arg prints current model", async () => {
    const ctx = ctxWith(freshState());
    await modelCommand.handler("", ctx);
    expect(ctx.logs[0]).toMatch(/claude-test/);
  });

  it("/model <name> mutates state", async () => {
    const state = freshState();
    const ctx = ctxWith(state);
    await modelCommand.handler("claude-opus-4-7", ctx);
    expect(state.model).toBe("claude-opus-4-7");
    expect(ctx.logs[0]).toMatch(/claude-test → claude-opus-4-7/);
  });

  it("/cost prints usage breakdown", async () => {
    const state = freshState();
    state.tokenUsage = { input: 100, output: 50, cache_read: 10, cache_creation: 5 };
    const ctx = ctxWith(state);
    await costCommand.handler("", ctx);
    const out = ctx.logs.join("\n");
    expect(out).toMatch(/input:\s+100/);
    expect(out).toMatch(/output:\s+50/);
  });

  it("/undo pops the most recent prior IR", async () => {
    const state = freshState();
    state.currentIr = "v3";
    state.irHistory = ["v2", "v1"];
    const ctx = ctxWith(state);
    await undoCommand.handler("", ctx);
    expect(state.currentIr).toBe("v2");
    expect(state.irHistory).toEqual(["v1"]);
  });

  it("/undo with empty history is a no-op with a clear message", async () => {
    const ctx = ctxWith(freshState());
    await undoCommand.handler("", ctx);
    expect(ctx.logs[0]).toMatch(/Nothing to undo/);
  });
});

describe("IR commands", () => {
  it("/show with no IR prints a hint", async () => {
    const ctx = ctxWith(freshState());
    await showCommand.handler("", ctx);
    expect(ctx.logs[0]).toMatch(/No IR/);
  });

  it("/show prints the current IR verbatim", async () => {
    const state = freshState();
    state.currentIr = "{ \"a\": 1 }";
    const ctx = ctxWith(state);
    await showCommand.handler("", ctx);
    expect(ctx.logs[0]).toBe("{ \"a\": 1 }");
  });

  it("/save writes the IR to a file", async () => {
    const state = freshState();
    state.currentIr = "{}";
    const ctx = ctxWith(state);
    const target = join(workspace, "out.voce.json");
    await saveCommand.handler(target, ctx);
    const fs = await import("node:fs");
    expect(fs.readFileSync(target, "utf-8")).toBe("{}");
  });

  it("/load reads a JSON file into state", async () => {
    const state = freshState();
    const ctx = ctxWith(state);
    const target = join(workspace, "in.voce.json");
    writeFileSync(target, "{\"x\":1}");
    await loadCommand.handler(target, ctx);
    expect(state.currentIr).toBe("{\"x\":1}");
  });

  it("/load rejects non-JSON content", async () => {
    const state = freshState();
    const ctx = ctxWith(state);
    const target = join(workspace, "bad.voce.json");
    writeFileSync(target, "not json");
    await loadCommand.handler(target, ctx);
    expect(state.currentIr).toBeNull();
    expect(ctx.logs.join("\n")).toMatch(/Failed to load/);
  });

  it("/ir parses inline JSON and pushes prior IR onto the undo stack", async () => {
    const state = freshState();
    state.currentIr = "{\"old\":true}";
    const ctx = ctxWith(state);
    await irCommand.handler("{\"new\":true}", ctx);
    expect(state.currentIr).toBe("{\"new\":true}");
    expect(state.irHistory[0]).toBe("{\"old\":true}");
  });

  it("/diff produces a meaningful summary", async () => {
    const state = freshState();
    state.irHistory = ["{\"a\":1}"];
    state.currentIr = "{\"a\":2}";
    const ctx = ctxWith(state);
    await diffCommand.handler("", ctx);
    const out = ctx.logs.join("\n");
    expect(out).toMatch(/before:/);
    expect(out).toMatch(/after:/);
  });

  it("summarizeDiff highlights differing lines", () => {
    const a = "{\n  \"x\": 1\n}";
    const b = "{\n  \"x\": 2\n}";
    const out = summarizeDiff(a, b);
    expect(out).toMatch(/--\s+"x": 1/);
    expect(out).toMatch(/\+\+\s+"x": 2/);
  });

  it("pushIrHistory caps history at 32", () => {
    const state = freshState();
    for (let i = 0; i < 40; i++) {
      pushIrHistory({ state }, `v${i}`);
    }
    expect(state.irHistory.length).toBeLessThanOrEqual(32);
    expect(state.currentIr).toBe("v39");
  });
});

describe("memory commands", () => {
  it("/brief shows nothing when no brief exists", async () => {
    const ctx = ctxWith(freshState());
    await briefCommand.handler("", ctx);
    expect(ctx.logs.join("\n")).toMatch(/No brief yet/);
  });

  it("/brief prints existing brief content", async () => {
    writeBrief("# Test brief\n\nbody");
    const ctx = ctxWith(freshState());
    await briefCommand.handler("", ctx);
    expect(ctx.logs.join("\n")).toMatch(/Test brief/);
  });

  it("/brief <md> writes the brief atomically", async () => {
    const ctx = ctxWith(freshState());
    await briefCommand.handler("# new", ctx);
    expect(ctx.logs.join("\n")).toMatch(/Brief written/);
  });

  it("/decisions prints recent log entries", async () => {
    logDecision({ summary: "Use TS", rationale: "Static types" });
    logDecision({ summary: "Use Postmark", rationale: "Lower bounce" });
    const ctx = ctxWith(freshState());
    await decisionsCommand.handler("5", ctx);
    expect(ctx.logs.join("\n")).toMatch(/Use TS/);
    expect(ctx.logs.join("\n")).toMatch(/Use Postmark/);
  });

  it("/decision logs a new entry from 'summary | rationale'", async () => {
    const ctx = ctxWith(freshState());
    await decisionCommand.handler("Use Postmark | Lower bounce vs Sendgrid", ctx);
    expect(ctx.logs.join("\n")).toMatch(/Logged decision/);
  });

  it("/decision rejects malformed input", async () => {
    const ctx = ctxWith(freshState());
    await decisionCommand.handler("only summary", ctx);
    expect(ctx.logs.join("\n")).toMatch(/summary and rationale are required/);
  });

  it("/explain returns submitText when an IR is loaded", async () => {
    const state = freshState();
    state.currentIr = "{}";
    const ctx = ctxWith(state);
    const result = await explainCommand.handler("", ctx);
    expect(result).toMatchObject({ handled: true });
    if (result.handled && "submitText" in result) {
      expect(result.submitText).toMatch(/Walk me through/);
    } else {
      throw new Error("expected submitText");
    }
  });

  it("/explain refuses gracefully when no IR exists", async () => {
    const ctx = ctxWith(freshState());
    const result = await explainCommand.handler("", ctx);
    if (result.handled && "submitText" in result) {
      throw new Error("should not have produced submitText");
    }
    expect(ctx.logs.join("\n")).toMatch(/No IR yet/);
  });
});
