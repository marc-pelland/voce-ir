// Workflow types — the 5-phase generation flow per S65 §3.

export type Phase =
  | "not_started"
  | "discovering"
  | "ready"
  | "proposed"
  | "finalized";

/**
 * A workflow event encoded into a SessionEntry. The session JSONL is the
 * single source of truth — workflow phase is derived by scanning it. Events
 * are written as session entries with role "system" and content set to
 * JSON.stringify(WorkflowEvent).
 */
export type WorkflowEvent =
  | { workflow: "started"; user_intent: string }
  | { workflow: "answered"; question: string; answer: string; ready: boolean }
  | { workflow: "proposed" }
  | { workflow: "refined"; feedback: string }
  | { workflow: "finalized" };

/** Snapshot of where a session is in the workflow. Pure derivation from the ledger. */
export interface WorkflowState {
  session_id: string;
  phase: Phase;
  user_intent: string | null;
  /** Number of (question, answer) pairs recorded. */
  discovery_turns: number;
  /** Set once any answered event flips ready=true, OR phase has advanced past discovery. */
  ready: boolean;
  has_proposal: boolean;
  finalized: boolean;
}

/** Readiness gating output — used by voce_generate_propose to refuse early calls. */
export interface ReadinessReport {
  /** 0–100. Built from turn count + presence of brief + ready flag. */
  score: number;
  ready: boolean;
  missing: string[];
  blocking: string[];
}

/** Output of the lightweight completeness pillar check. */
export interface CompletenessReport {
  complete: boolean;
  missing_pillars: string[];
}
