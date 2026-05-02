// Readiness + completeness scoring. Pure functions — no I/O. Both are
// "advisory" per S65 §5: the workflow uses readiness to gate propose
// (score < 70 blocks) and completeness to gate finalize (any missing pillar
// blocks).

import type { WorkflowState, ReadinessReport, CompletenessReport } from "./types.js";

/**
 * Score how ready a session is to move from discovery → propose. Conservative:
 * we want the calling agent to actually do discovery, not skip to generation.
 *
 * Components (weights add to 100):
 *   user_intent recorded:    20
 *   discovery turns ≥ 1:     20
 *   discovery turns ≥ 3:     20
 *   discovery turns ≥ 5:     20
 *   agent declared ready:    20
 *
 * The 70-pt floor matches the spec's "voce_generate_propose blocks on score < 70".
 */
export function scoreReadiness(
  state: WorkflowState,
  opts: { briefPresent: boolean } = { briefPresent: false },
): ReadinessReport {
  const missing: string[] = [];
  const blocking: string[] = [];

  let score = 0;

  if (state.user_intent !== null && state.user_intent.length > 0) {
    score += 20;
  } else {
    missing.push("user_intent — call voce_generate_start with the user's brief");
    blocking.push("user_intent");
  }

  if (state.discovery_turns >= 1) score += 20;
  else missing.push("at least one discovery turn (voce_generate_answer)");

  if (state.discovery_turns >= 3) score += 20;
  else if (state.discovery_turns < 3) missing.push("3+ discovery turns recommended");

  if (state.discovery_turns >= 5) score += 20;
  else if (state.discovery_turns < 5) missing.push("5+ discovery turns ideal");

  if (state.ready) score += 20;
  else if (!state.ready) missing.push("agent has not declared ready (pass ready: true on voce_generate_answer)");

  // Brief presence is an optional bonus — doesn't move the score, but the
  // missing list mentions it so the agent knows to check.
  if (!opts.briefPresent) {
    missing.push("project brief absent — voce_brief_set when the conversation produces a north star");
  }

  return { score, ready: score >= 70, missing, blocking };
}

/**
 * Lightweight pillar coverage check — looks at the IR JSON as a string and
 * flags missing structural patterns. Cheap heuristics; the validator catches
 * the strict cases. This is a *generation-time* check, not a *correctness*
 * check — it asks "did the agent remember to include the things users expect"
 * not "is this IR valid".
 */
export function scoreCompleteness(irJson: string): CompletenessReport {
  const missing_pillars: string[] = [];

  // Accessibility — interactive IR should declare semantic nodes.
  const hasInteractive = /"value_type"\s*:\s*"(?:FormNode|ActionNode|Surface)"/i.test(irJson);
  const hasSemantic = /"semantic_node_id"|"value_type"\s*:\s*"SemanticNode"/i.test(irJson);
  if (hasInteractive && !hasSemantic) {
    missing_pillars.push("a11y — interactive nodes present but no semantic_node_id references");
  }

  // Forms — a FormNode without validation rules is a half-implementation.
  if (/"value_type"\s*:\s*"FormNode"/i.test(irJson) && !/"validation_rules"|"ValidationRule"/i.test(irJson)) {
    missing_pillars.push("validation — FormNode present without ValidationRule");
  }

  // Actions — server actions need error and loading states.
  if (/"value_type"\s*:\s*"ActionNode"/i.test(irJson)) {
    if (!/"error_state"|"on_error"/i.test(irJson)) missing_pillars.push("error states — ActionNode without an error path");
    if (!/"loading_state"|"pending_state"/i.test(irJson)) missing_pillars.push("loading states — ActionNode without a pending UI");
  }

  // Empty states — lists/data subscriptions usually need them.
  if (/"value_type"\s*:\s*"SubscriptionNode"/i.test(irJson) && !/"empty_state"|"on_empty"/i.test(irJson)) {
    missing_pillars.push("empty states — SubscriptionNode without an empty UI");
  }

  // SEO — pages should declare metadata.
  if (/"value_type"\s*:\s*"ViewRoot"|"PageMetadata"/i.test(irJson) && !/"PageMetadata"|"metadata"\s*:/i.test(irJson)) {
    missing_pillars.push("seo — ViewRoot without PageMetadata");
  }

  return { complete: missing_pillars.length === 0, missing_pillars };
}
