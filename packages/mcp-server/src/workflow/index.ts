// Public surface of the generation workflow module. The MCP tool layer
// (../index.ts) consumes only what's exported here.

export type {
  CompletenessReport,
  Phase,
  ReadinessReport,
  WorkflowEvent,
  WorkflowState,
} from "./types.js";

export { decodeEvent, encodeEvent, getWorkflowState } from "./state.js";
export { scoreCompleteness, scoreReadiness } from "./scoring.js";

export type {
  AnswerResult,
  FinalizeGate,
  ProposeResult,
  RefineResult,
  StartResult,
} from "./orchestrator.js";
export {
  gateFinalize,
  recordAnswer,
  recordFinalization,
  recordProposal,
  recordRefinement,
  startGeneration,
} from "./orchestrator.js";
