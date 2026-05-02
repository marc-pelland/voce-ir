// Workflow state derivation — read the session ledger, compute the current phase.
// No mutation in this file; writes go through orchestrator.ts.

import { readSession } from "../memory/session.js";
import type { SessionEntry } from "../memory/types.js";
import type { Phase, WorkflowEvent, WorkflowState } from "./types.js";

/** Decode a session entry as a WorkflowEvent if possible. */
export function decodeEvent(entry: SessionEntry): WorkflowEvent | null {
  if (entry.role !== "system") return null;
  try {
    const parsed = JSON.parse(entry.content);
    if (
      parsed !== null &&
      typeof parsed === "object" &&
      typeof (parsed as { workflow?: unknown }).workflow === "string"
    ) {
      return parsed as WorkflowEvent;
    }
  } catch {
    // Not a workflow event — treat as plain system message.
  }
  return null;
}

export function encodeEvent(event: WorkflowEvent): string {
  return JSON.stringify(event);
}

/**
 * Derive the current state of `sessionId` by replaying its workflow events.
 * If the session has zero workflow events, phase is "not_started".
 */
export function getWorkflowState(sessionId: string): WorkflowState {
  const entries = readSession(sessionId);
  let phase: Phase = "not_started";
  let user_intent: string | null = null;
  let discovery_turns = 0;
  let ready = false;
  let has_proposal = false;
  let finalized = false;

  for (const entry of entries) {
    const ev = decodeEvent(entry);
    if (ev === null) continue;

    switch (ev.workflow) {
      case "started":
        phase = "discovering";
        user_intent = ev.user_intent;
        break;
      case "answered":
        discovery_turns += 1;
        if (ev.ready) {
          ready = true;
          phase = "ready";
        }
        break;
      case "proposed":
        has_proposal = true;
        phase = "proposed";
        ready = true; // propose implies the agent decided we're past discovery
        break;
      case "refined":
        // Stay in "proposed" — refine produces a new IR but doesn't advance phase.
        has_proposal = true;
        phase = "proposed";
        break;
      case "finalized":
        finalized = true;
        phase = "finalized";
        break;
    }
  }

  return {
    session_id: sessionId,
    phase,
    user_intent,
    discovery_turns,
    ready,
    has_proposal,
    finalized,
  };
}
