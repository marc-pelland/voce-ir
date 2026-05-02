// Generation Readiness Score — formatted for the user before any
// voce_generate_propose call. The numbers and missing list come from the
// shared workflow scoring; this file only owns presentation + the
// [Y/n/q] interactive resolution.

import chalk from "chalk";
import type { ReadinessReport } from "@voce-ir/mcp-server/workflow";

export function formatReadinessReport(r: ReadinessReport): string {
  const bar = `${r.score}/100`;
  const tone = r.score >= 70 ? chalk.green : r.score >= 40 ? chalk.yellow : chalk.red;
  const lines = [
    "",
    chalk.bold(`Generation Readiness: ${tone(bar)}`),
    "",
  ];
  if (r.missing.length === 0) {
    lines.push(chalk.green("  ✓ All discovery checkpoints satisfied."));
  } else {
    for (const item of r.missing) {
      lines.push(`  ${chalk.dim("✗")} ${item}`);
    }
  }
  if (r.blocking.length > 0) {
    lines.push("");
    lines.push(chalk.red(`  Blocking: ${r.blocking.join(", ")}`));
  }
  return lines.join("\n");
}

export type ReadinessChoice = "proceed" | "abort" | "ask-question";

export function parseReadinessChoice(answer: string): ReadinessChoice {
  const a = answer.trim().toLowerCase();
  if (a === "" || a === "y" || a === "yes") return "proceed";
  if (a === "q" || a === "question" || a === "ask") return "ask-question";
  return "abort"; // "n", "no", anything else
}

export const READINESS_PROMPT = "Proceed with proposal? [Y/n/q for question] ";
