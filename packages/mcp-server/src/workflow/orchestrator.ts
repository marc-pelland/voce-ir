// Orchestrator — the five workflow phases as state-mutating functions.
// All persistence is through the .voce/ memory module; this file owns the
// state transitions and gating logic, not I/O details.

import { appendSession, newSessionId } from "../memory/session.js";
import { encodeEvent, getWorkflowState } from "./state.js";
import { scoreCompleteness, scoreReadiness } from "./scoring.js";
import type { CompletenessReport, ReadinessReport, WorkflowState } from "./types.js";

export interface StartResult {
  session_id: string;
  state: WorkflowState;
  readiness: ReadinessReport;
}

export interface AnswerResult {
  state: WorkflowState;
  readiness: ReadinessReport;
}

export interface ProposeResult {
  ok: boolean;
  state: WorkflowState;
  readiness: ReadinessReport;
  completeness: CompletenessReport;
  /** Set when ok === false — explains what to do next. */
  message?: string;
}

export interface RefineResult {
  state: WorkflowState;
  completeness: CompletenessReport;
}

export interface FinalizeGate {
  ok: boolean;
  state: WorkflowState;
  completeness: CompletenessReport;
  /** Set when ok === false — explains what to do next. */
  message?: string;
}

/**
 * Begin a new generation session. `user_intent` is the user's first brief —
 * stored verbatim so subsequent phases can reference it.
 */
export function startGeneration(userIntent: string): StartResult {
  if (typeof userIntent !== "string" || userIntent.trim().length === 0) {
    throw new Error("startGeneration: user_intent must be a non-empty string");
  }
  const sessionId = newSessionId();
  appendSession(sessionId, {
    role: "user",
    content: userIntent,
  });
  appendSession(sessionId, {
    role: "system",
    content: encodeEvent({ workflow: "started", user_intent: userIntent }),
  });
  const state = getWorkflowState(sessionId);
  return { session_id: sessionId, state, readiness: scoreReadiness(state) };
}

/**
 * Record one (question, answer) pair from the discovery phase. The agent
 * passes `ready: true` when it has enough context to propose. Server does
 * not auto-flip ready — that's the agent's call.
 */
export function recordAnswer(
  sessionId: string,
  question: string,
  answer: string,
  ready: boolean,
): AnswerResult {
  const before = getWorkflowState(sessionId);
  if (before.phase === "not_started") {
    throw new Error("recordAnswer: no session — call voce_generate_start first");
  }
  if (before.phase === "finalized") {
    throw new Error("recordAnswer: session is finalized — start a new one");
  }
  appendSession(sessionId, { role: "assistant", content: question });
  appendSession(sessionId, { role: "user", content: answer });
  appendSession(sessionId, {
    role: "system",
    content: encodeEvent({ workflow: "answered", question, answer, ready }),
  });
  const state = getWorkflowState(sessionId);
  return { state, readiness: scoreReadiness(state) };
}

/**
 * Propose an IR. Refuses if readiness score < 70. Records the IR as an
 * ir_snapshot on the session so resume / finalize can pick it up.
 */
export function recordProposal(
  sessionId: string,
  irJson: string,
  opts: { briefPresent: boolean } = { briefPresent: false },
): ProposeResult {
  const before = getWorkflowState(sessionId);
  if (before.phase === "not_started") {
    throw new Error("recordProposal: no session — call voce_generate_start first");
  }
  const readiness = scoreReadiness(before, opts);
  if (!readiness.ready) {
    return {
      ok: false,
      state: before,
      readiness,
      completeness: { complete: false, missing_pillars: [] },
      message:
        `Readiness score ${readiness.score} < 70. Continue discovery — ` +
        `use voce_generate_answer until the agent declares ready: true.`,
    };
  }
  if (typeof irJson !== "string" || irJson.length === 0) {
    throw new Error("recordProposal: ir_json must be a non-empty string");
  }
  appendSession(sessionId, {
    role: "assistant",
    content: "Proposed IR",
    ir_snapshot: irJson,
  });
  appendSession(sessionId, {
    role: "system",
    content: encodeEvent({ workflow: "proposed" }),
  });
  const state = getWorkflowState(sessionId);
  return {
    ok: true,
    state,
    readiness: scoreReadiness(state, opts),
    completeness: scoreCompleteness(irJson),
  };
}

export function recordRefinement(
  sessionId: string,
  feedback: string,
  irJson: string,
): RefineResult {
  const before = getWorkflowState(sessionId);
  if (!before.has_proposal) {
    throw new Error("recordRefinement: no proposal yet — call voce_generate_propose first");
  }
  if (typeof irJson !== "string" || irJson.length === 0) {
    throw new Error("recordRefinement: ir_json must be a non-empty string");
  }
  appendSession(sessionId, { role: "user", content: feedback });
  appendSession(sessionId, {
    role: "assistant",
    content: "Refined IR",
    ir_snapshot: irJson,
  });
  appendSession(sessionId, {
    role: "system",
    content: encodeEvent({ workflow: "refined", feedback }),
  });
  const state = getWorkflowState(sessionId);
  return { state, completeness: scoreCompleteness(irJson) };
}

/**
 * Gate finalize: refuse if no proposal exists or completeness check fails.
 * The MCP tool layer is responsible for the actual validate + compile calls
 * — this function returns whether the gate is open.
 */
export function gateFinalize(sessionId: string, irJson: string): FinalizeGate {
  const state = getWorkflowState(sessionId);
  if (!state.has_proposal) {
    return {
      ok: false,
      state,
      completeness: { complete: false, missing_pillars: [] },
      message: "No proposal — call voce_generate_propose first.",
    };
  }
  const completeness = scoreCompleteness(irJson);
  if (!completeness.complete) {
    return {
      ok: false,
      state,
      completeness,
      message:
        "Completeness check failed — address each missing pillar via " +
        "voce_generate_refine before finalizing.",
    };
  }
  return { ok: true, state, completeness };
}

/** Mark the session finalized. Called only after the validate + compile path succeeds. */
export function recordFinalization(sessionId: string): WorkflowState {
  appendSession(sessionId, {
    role: "system",
    content: encodeEvent({ workflow: "finalized" }),
  });
  return getWorkflowState(sessionId);
}
