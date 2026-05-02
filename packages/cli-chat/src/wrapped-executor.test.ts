// Wrapped executor tests — the cli-chat-only logic that gates propose on
// readiness UI and walks drift reports with [r/s/c]. Both depend on
// .voce/ state, so each test runs against a fresh temp workspace.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { wrapExecutor } from "./wrapped-executor.js";
import { listDecisions, logDecision } from "@voce-ir/mcp-server/memory";

let workspace: string;

beforeEach(() => {
  workspace = mkdtempSync(join(tmpdir(), "voce-wrapped-test-"));
  process.env.VOCE_PROJECT_ROOT = workspace;
});

afterEach(() => {
  rmSync(workspace, { recursive: true, force: true });
  delete process.env.VOCE_PROJECT_ROOT;
});

describe("wrapExecutor", () => {
  it("passes through unrelated tool calls without prompting", async () => {
    const ask = vi.fn();
    const log = vi.fn();
    const exec = wrapExecutor({ ask, log });
    const result = await exec("voce_brief_get", {});
    expect(result.content[0]?.text).toMatch(/brief_md/);
    expect(ask).not.toHaveBeenCalled();
  });

  it("on voce_generate_propose: shows readiness, blocks on 'n'", async () => {
    const ask = vi.fn().mockResolvedValueOnce("n");
    const log = vi.fn();
    const exec = wrapExecutor({ ask, log });
    const result = await exec("voce_generate_propose", { session_id: "fake" });
    const parsed = JSON.parse(result.content[0]?.text ?? "{}");
    expect(parsed.intercepted).toBe("user_paused");
    expect(parsed.ok).toBe(false);
    // The readiness header should have been logged before the prompt.
    expect(log).toHaveBeenCalled();
  });

  it("on voce_generate_propose: 'q' captures the follow-up question", async () => {
    const ask = vi
      .fn()
      .mockResolvedValueOnce("q")
      .mockResolvedValueOnce("What's the primary user persona?");
    const log = vi.fn();
    const exec = wrapExecutor({ ask, log });
    const result = await exec("voce_generate_propose", { session_id: "fake" });
    const parsed = JSON.parse(result.content[0]?.text ?? "{}");
    expect(parsed.intercepted).toBe("user_wants_question_first");
    expect(parsed.user_followup).toBe("What's the primary user persona?");
  });

  it("on voce_check_drift: walks each report and persists the choice", async () => {
    const target = logDecision({ summary: "no modal nodes", rationale: "Modals trap focus" });
    const ask = vi.fn().mockResolvedValueOnce("s");
    const log = vi.fn();
    const exec = wrapExecutor({ ask, log });
    const result = await exec("voce_check_drift", {
      proposed_ir: JSON.stringify({ value_type: "Modal" }),
    });
    const parsed = JSON.parse(result.content[0]?.text ?? "{}");
    expect(parsed.drift).toHaveLength(1);
    // Supersede recorded as a fresh decision referencing target.id.
    const decisions = listDecisions();
    expect(decisions.length).toBeGreaterThan(1);
    const supersession = decisions[decisions.length - 1];
    expect(supersession?.supersedes).toBe(target.id);
  });

  it("on voce_check_drift: 'c' (continue) records a conflicts_with decision", async () => {
    const target = logDecision({ summary: "no modal nodes", rationale: "x" });
    const ask = vi.fn().mockResolvedValueOnce("c");
    const log = vi.fn();
    const exec = wrapExecutor({ ask, log });
    await exec("voce_check_drift", {
      proposed_ir: JSON.stringify({ value_type: "Modal" }),
    });
    const decisions = listDecisions();
    const last = decisions[decisions.length - 1];
    expect(last?.conflicts_with).toBe(target.id);
    expect(last?.supersedes).toBeUndefined();
  });

  it("on voce_check_drift: 'r' (revise) doesn't write a decision", async () => {
    logDecision({ summary: "no modal nodes", rationale: "x" });
    const ask = vi.fn().mockResolvedValueOnce("r");
    const log = vi.fn();
    const exec = wrapExecutor({ ask, log });
    const beforeCount = listDecisions().length;
    await exec("voce_check_drift", {
      proposed_ir: JSON.stringify({ value_type: "Modal" }),
    });
    const afterCount = listDecisions().length;
    expect(afterCount).toBe(beforeCount);
  });

  it("re-prompts on invalid drift choice", async () => {
    logDecision({ summary: "no modal nodes", rationale: "x" });
    const ask = vi
      .fn()
      .mockResolvedValueOnce("z")  // invalid
      .mockResolvedValueOnce("r"); // valid
    const log = vi.fn();
    const exec = wrapExecutor({ ask, log });
    await exec("voce_check_drift", {
      proposed_ir: JSON.stringify({ value_type: "Modal" }),
    });
    expect(ask).toHaveBeenCalledTimes(2);
    expect(log.mock.calls.some((c) => /Type r, s, or c/.test(c[0]))).toBe(true);
  });
});
