// Public surface of the .voce/ memory module. Day 3 tools (voce_brief_*,
// voce_decisions_*, voce_session_*, voce_check_drift) consume these helpers.
// Direct fs access from outside this module is forbidden — go through here.

export type { Brief } from "./brief.js";
export { readBrief, writeBrief } from "./brief.js";

export type { Decision, DriftWarning, SessionEntry } from "./types.js";
export {
  validateDecision,
  validateDriftWarning,
  validateSessionEntry,
} from "./types.js";

export type { LogDecisionInput } from "./decisions.js";
export { getDecision, listDecisions, logDecision } from "./decisions.js";

export type { LogDriftInput } from "./drift.js";
export { listDrift, logDrift } from "./drift.js";

export type { AppendSessionInput, SessionSummary } from "./session.js";
export {
  appendSession,
  latestIrSnapshot,
  listSessions,
  newSessionId,
  readSession,
} from "./session.js";

export { PATHS, ensureVoceDir, voceDir } from "./store.js";

export type { DriftCheckResult, DriftReport } from "./drift-check.js";
export { detectDrift } from "./drift-check.js";
