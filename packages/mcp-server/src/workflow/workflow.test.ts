// Workflow tests — phase derivation, readiness gate, completeness check, and
// the orchestrator's refusal contract.

import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { newSessionId } from "../memory/session.js";
import {
  gateFinalize,
  recordAnswer,
  recordFinalization,
  recordProposal,
  recordRefinement,
  startGeneration,
} from "./orchestrator.js";
import { getWorkflowState } from "./state.js";
import { scoreCompleteness, scoreReadiness } from "./scoring.js";

let workspace: string;

beforeEach(() => {
  workspace = mkdtempSync(join(tmpdir(), "voce-workflow-test-"));
  process.env.VOCE_PROJECT_ROOT = workspace;
});

afterEach(() => {
  rmSync(workspace, { recursive: true, force: true });
  delete process.env.VOCE_PROJECT_ROOT;
});

describe("startGeneration", () => {
  it("opens a session in the discovering phase", () => {
    const r = startGeneration("Build a coffee wholesale platform");
    expect(r.state.phase).toBe("discovering");
    expect(r.state.user_intent).toBe("Build a coffee wholesale platform");
    expect(r.state.discovery_turns).toBe(0);
    expect(r.readiness.ready).toBe(false);
  });

  it("rejects empty user_intent", () => {
    expect(() => startGeneration("")).toThrow(/user_intent/);
    expect(() => startGeneration("   ")).toThrow(/user_intent/);
  });
});

describe("recordAnswer", () => {
  it("increments discovery_turns and stays in discovering", () => {
    const { session_id } = startGeneration("x");
    const r = recordAnswer(session_id, "What does the product do?", "Coffee wholesale", false);
    expect(r.state.discovery_turns).toBe(1);
    expect(r.state.phase).toBe("discovering");
    expect(r.state.ready).toBe(false);
  });

  it("flips ready when the agent declares it", () => {
    const { session_id } = startGeneration("x");
    recordAnswer(session_id, "q1", "a1", false);
    recordAnswer(session_id, "q2", "a2", false);
    const r = recordAnswer(session_id, "q3", "a3", true);
    expect(r.state.ready).toBe(true);
    expect(r.state.phase).toBe("ready");
  });

  it("refuses when the session has not been started", () => {
    expect(() => recordAnswer(newSessionId(), "q", "a", false)).toThrow(/no session/);
  });

  it("refuses when the session is finalized", () => {
    const { session_id } = startGeneration("x");
    for (let i = 0; i < 5; i++) recordAnswer(session_id, `q${i}`, `a${i}`, i === 4);
    recordProposal(session_id, completeIr());
    recordFinalization(session_id);
    expect(() => recordAnswer(session_id, "q", "a", false)).toThrow(/finalized/);
  });
});

describe("recordProposal", () => {
  it("blocks when readiness < 70", () => {
    const { session_id } = startGeneration("x");
    // Only one turn — readiness will be 20 (intent) + 20 (1 turn) = 40.
    recordAnswer(session_id, "q", "a", false);
    const r = recordProposal(session_id, completeIr());
    expect(r.ok).toBe(false);
    expect(r.message).toMatch(/Readiness score/);
  });

  it("accepts when readiness >= 70", () => {
    const { session_id } = startGeneration("x");
    recordAnswer(session_id, "q1", "a1", false);
    recordAnswer(session_id, "q2", "a2", false);
    recordAnswer(session_id, "q3", "a3", false);
    recordAnswer(session_id, "q4", "a4", true); // sets ready: 20+20+20+20+20=100
    const r = recordProposal(session_id, completeIr());
    expect(r.ok).toBe(true);
    expect(r.state.phase).toBe("proposed");
    expect(r.state.has_proposal).toBe(true);
  });

  it("flags missing pillars in completeness", () => {
    const { session_id } = startGeneration("x");
    for (let i = 0; i < 4; i++) recordAnswer(session_id, `q${i}`, `a${i}`, i === 3);
    const r = recordProposal(session_id, JSON.stringify({
      value_type: "ViewRoot",
      children: [{ value_type: "FormNode", node_id: "form" }],
    }));
    expect(r.ok).toBe(true);
    expect(r.completeness.complete).toBe(false);
    expect(r.completeness.missing_pillars.some((p) => /a11y/.test(p))).toBe(true);
    expect(r.completeness.missing_pillars.some((p) => /validation/.test(p))).toBe(true);
  });
});

describe("recordRefinement", () => {
  it("requires a prior proposal", () => {
    const { session_id } = startGeneration("x");
    expect(() => recordRefinement(session_id, "feedback", completeIr())).toThrow(/no proposal/);
  });

  it("updates the IR snapshot without leaving the proposed phase", () => {
    const { session_id } = startGeneration("x");
    for (let i = 0; i < 4; i++) recordAnswer(session_id, `q${i}`, `a${i}`, i === 3);
    recordProposal(session_id, completeIr());
    const r = recordRefinement(session_id, "make the headline shorter", completeIr({ headline: "Short" }));
    expect(r.state.phase).toBe("proposed");
  });
});

describe("gateFinalize", () => {
  it("blocks when no proposal exists", () => {
    const { session_id } = startGeneration("x");
    const gate = gateFinalize(session_id, completeIr());
    expect(gate.ok).toBe(false);
    expect(gate.message).toMatch(/No proposal/);
  });

  it("blocks when completeness fails", () => {
    const { session_id } = startGeneration("x");
    for (let i = 0; i < 4; i++) recordAnswer(session_id, `q${i}`, `a${i}`, i === 3);
    const incomplete = JSON.stringify({
      value_type: "ViewRoot",
      children: [{ value_type: "FormNode", node_id: "form" }],
    });
    recordProposal(session_id, incomplete);
    const gate = gateFinalize(session_id, incomplete);
    expect(gate.ok).toBe(false);
    expect(gate.message).toMatch(/Completeness check failed/);
  });

  it("opens when proposal + completeness are both clean", () => {
    const { session_id } = startGeneration("x");
    for (let i = 0; i < 4; i++) recordAnswer(session_id, `q${i}`, `a${i}`, i === 3);
    const ir = completeIr();
    recordProposal(session_id, ir);
    const gate = gateFinalize(session_id, ir);
    expect(gate.ok).toBe(true);
  });
});

describe("getWorkflowState (replay)", () => {
  it("derives phase 'finalized' after the full happy path", () => {
    const { session_id } = startGeneration("x");
    for (let i = 0; i < 4; i++) recordAnswer(session_id, `q${i}`, `a${i}`, i === 3);
    recordProposal(session_id, completeIr());
    recordFinalization(session_id);
    expect(getWorkflowState(session_id).phase).toBe("finalized");
  });
});

describe("scoreReadiness", () => {
  it("starts at 20 with just user_intent", () => {
    const r = scoreReadiness({
      session_id: "s",
      phase: "discovering",
      user_intent: "x",
      discovery_turns: 0,
      ready: false,
      has_proposal: false,
      finalized: false,
    });
    expect(r.score).toBe(20);
    expect(r.ready).toBe(false);
  });

  it("hits 100 with full discovery + ready flag", () => {
    const r = scoreReadiness({
      session_id: "s",
      phase: "ready",
      user_intent: "x",
      discovery_turns: 5,
      ready: true,
      has_proposal: false,
      finalized: false,
    });
    expect(r.score).toBe(100);
    expect(r.ready).toBe(true);
  });
});

describe("scoreCompleteness", () => {
  it("returns complete: true on a richly-structured IR", () => {
    expect(scoreCompleteness(completeIr()).complete).toBe(true);
  });

  it("flags interactive IR without semantic_node_id", () => {
    const ir = JSON.stringify({ value_type: "FormNode", node_id: "form", validation_rules: [{}] });
    expect(scoreCompleteness(ir).missing_pillars.some((p) => /a11y/.test(p))).toBe(true);
  });

  it("flags ActionNode without error_state", () => {
    const ir = JSON.stringify({
      value_type: "ActionNode",
      node_id: "submit",
      loading_state: "x",
      semantic_node_id: "s1",
    });
    expect(scoreCompleteness(ir).missing_pillars.some((p) => /error states/.test(p))).toBe(true);
  });
});

// ─── helpers ────────────────────────────────────────────────────

interface IrOpts { headline?: string }

function completeIr(opts: IrOpts = {}): string {
  return JSON.stringify({
    value_type: "ViewRoot",
    metadata: { value_type: "PageMetadata", title: opts.headline ?? "Voce" },
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
