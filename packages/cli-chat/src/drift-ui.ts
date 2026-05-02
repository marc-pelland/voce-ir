// Drift push-back UI. After voce_check_drift returns reports, this module
// formats each potential conflict and resolves [r/s/c]:
//   r  revise the IR — the model gets told to fix it and try again
//   s  supersede the prior decision — log a new decision overriding it
//   c  continue anyway — note the override but don't change anything
//
// Both `s` and `c` write a record to .voce/decisions.jsonl so the choice
// is auditable.

import chalk from "chalk";
import type { DriftReport } from "@voce-ir/mcp-server/memory";

export function formatDriftReport(report: DriftReport): string {
  const date = report.decision_timestamp.slice(0, 10);
  const idShort = report.decision_id.slice(0, 8);
  const lines = [
    "",
    chalk.yellow(`Hold on — decision [${idShort}] from ${date} may conflict with this IR.`),
    chalk.dim(`  "${report.decision_summary}"`),
    chalk.dim(`  matched terms: ${report.matched_terms.join(", ")}`),
    "",
    chalk.dim(`  ${report.note}`),
  ];
  return lines.join("\n");
}

export type DriftChoice = "revise" | "supersede" | "continue";

export function parseDriftChoice(answer: string): DriftChoice | null {
  const a = answer.trim().toLowerCase();
  if (a === "r" || a === "revise") return "revise";
  if (a === "s" || a === "supersede") return "supersede";
  if (a === "c" || a === "continue") return "continue";
  return null;
}

export const DRIFT_PROMPT =
  "[r] revise IR  [s] supersede decision  [c] continue anyway: ";

/** Compose a decision-log entry recording how the user resolved the drift. */
export function driftResolutionAsDecision(
  report: DriftReport,
  choice: DriftChoice,
): { summary: string; rationale: string; supersedes?: string; conflicts_with?: string } {
  if (choice === "supersede") {
    return {
      summary: `Supersedes decision ${report.decision_id.slice(0, 8)} (drift resolution)`,
      rationale: `User chose to supersede the prior decision ("${report.decision_summary}") via /generate flow.`,
      supersedes: report.decision_id,
    };
  }
  // continue: knowingly override but don't supersede
  return {
    summary: `Knowingly continued past drift on decision ${report.decision_id.slice(0, 8)}`,
    rationale: `User chose [c] — continue anyway despite term overlap with prior decision ("${report.decision_summary}"). Original decision stands.`,
    conflicts_with: report.decision_id,
  };
}
