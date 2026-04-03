/**
 * Brief enforcement — checks requests against the project brief.
 *
 * Before generation, compares the user's intent against the saved
 * brief and flags contradictions or scope drift.
 */

import type { DiscoveryBrief } from "../agents/types.js";

export interface EnforcementResult {
  /** Whether the request is compatible with the brief. */
  allowed: boolean;
  /** Warnings (tensions that don't block). */
  warnings: string[];
  /** Blockers (direct contradictions that should be resolved). */
  blockers: string[];
}

/**
 * Check a user request against the saved brief.
 * Returns warnings for tensions and blockers for contradictions.
 */
export function enforceBreif(
  request: string,
  brief: DiscoveryBrief
): EnforcementResult {
  const warnings: string[] = [];
  const blockers: string[] = [];
  const lower = request.toLowerCase();

  // Check for scope expansion
  const briefSections = new Set(
    brief.sections.map((s) => s.toLowerCase())
  );
  const expansionKeywords = [
    "add",
    "new section",
    "new page",
    "also include",
    "plus",
  ];
  const isExpansion = expansionKeywords.some((k) => lower.includes(k));

  if (isExpansion) {
    // Check if the requested addition is already in scope
    const isInScope = brief.sections.some((s) =>
      lower.includes(s.toLowerCase())
    );
    if (!isInScope) {
      warnings.push(
        `This adds scope beyond your brief. Your planned sections are: ${brief.sections.join(", ")}. Consider whether this is a must-have or nice-to-have.`
      );
    }
  }

  // Check for tone contradictions
  const toneLower = brief.tone.toLowerCase();
  const toneConflicts: Array<[string, string]> = [
    ["minimal", "heavy animation"],
    ["minimal", "particle effect"],
    ["clean", "cluttered"],
    ["dark", "light theme"],
    ["light", "dark theme"],
    ["professional", "playful"],
    ["simple", "complex"],
  ];

  for (const [briefWord, requestWord] of toneConflicts) {
    if (toneLower.includes(briefWord) && lower.includes(requestWord)) {
      warnings.push(
        `Your brief tone is "${brief.tone}" but this request includes "${requestWord}". These may conflict.`
      );
    }
  }

  // Check for constraint violations
  for (const constraint of brief.constraints) {
    const cLower = constraint.toLowerCase();
    if (cLower.includes("no auth") && lower.includes("login")) {
      blockers.push(
        `Your brief says "${constraint}" but this request involves login/authentication. Update the brief first.`
      );
    }
    if (cLower.includes("no form") && lower.includes("form")) {
      blockers.push(
        `Your brief says "${constraint}" but this request involves a form.`
      );
    }
  }

  return {
    allowed: blockers.length === 0,
    warnings,
    blockers,
  };
}
