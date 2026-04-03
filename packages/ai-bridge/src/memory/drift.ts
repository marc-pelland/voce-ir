/**
 * Drift detection — measures how well the current state matches the brief.
 *
 * Scores 0-100: how aligned is the generated output with the original plan?
 * Checks: planned sections present, style consistency, unplanned additions.
 */

import type { DiscoveryBrief } from "../agents/types.js";

export interface DriftReport {
  /** Overall drift score (0 = perfect alignment, 100 = completely drifted). */
  score: number;
  /** Planned sections that are missing from the IR. */
  missingSections: string[];
  /** Sections in the IR that weren't in the plan. */
  unplannedSections: string[];
  /** Human-readable summary. */
  summary: string;
}

/**
 * Detect drift between the brief and the current IR.
 *
 * @param brief The saved project brief.
 * @param irSections Section names found in the current IR (extracted during ingestion).
 */
export function detectDrift(
  brief: DiscoveryBrief,
  irSections: string[]
): DriftReport {
  const plannedLower = new Set(
    brief.sections.map((s) => s.toLowerCase().trim())
  );
  const actualLower = new Set(
    irSections.map((s) => s.toLowerCase().trim())
  );

  // Missing: in plan but not in IR
  const missing = brief.sections.filter(
    (s) => !actualLower.has(s.toLowerCase().trim())
  );

  // Unplanned: in IR but not in plan
  const unplanned = irSections.filter(
    (s) => !plannedLower.has(s.toLowerCase().trim())
  );

  // Score: penalize missing more than unplanned
  const totalPlanned = brief.sections.length || 1;
  const missingPenalty = (missing.length / totalPlanned) * 60;
  const unplannedPenalty = (unplanned.length / Math.max(totalPlanned, 1)) * 20;
  const score = Math.min(Math.round(missingPenalty + unplannedPenalty), 100);

  // Summary
  let summary: string;
  if (score === 0) {
    summary = "Perfectly aligned with the brief.";
  } else if (score < 20) {
    summary = "Minor drift — mostly aligned with the brief.";
  } else if (score < 50) {
    summary = "Moderate drift — some planned sections are missing or unplanned elements were added.";
  } else {
    summary = "Significant drift — the output has diverged from the original brief. Consider reviewing.";
  }

  if (missing.length > 0) {
    summary += ` Missing: ${missing.join(", ")}.`;
  }
  if (unplanned.length > 0) {
    summary += ` Unplanned: ${unplanned.join(", ")}.`;
  }

  return { score, missingSections: missing, unplannedSections: unplanned, summary };
}

/** Format drift score as a colored status string. */
export function driftStatus(score: number): string {
  if (score === 0) return "on track";
  if (score < 20) return "minor drift";
  if (score < 50) return "moderate drift";
  return "significant drift";
}
