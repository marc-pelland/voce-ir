/**
 * Session context builder — summarizes the current project state
 * for agent consumption.
 *
 * Keeps under 2K tokens so it fits alongside the schema context
 * and user prompt.
 */

import type { DiscoveryBrief } from "../agents/types.js";
import type { Decision } from "../memory/decisions.js";

export interface SessionContext {
  /** Compact text summary for agent system prompts. */
  summary: string;
  /** Token estimate. */
  estimatedTokens: number;
}

/**
 * Build a session context from project state.
 */
export function buildSessionContext(params: {
  brief: DiscoveryBrief | null;
  stylePack: string | null;
  decisions: Decision[];
  irSections: string[];
  turnCount: number;
}): SessionContext {
  const lines: string[] = [];

  lines.push("## Current Project Context\n");

  if (params.brief) {
    lines.push(`Purpose: ${params.brief.purpose}`);
    lines.push(`Audience: ${params.brief.audience}`);
    lines.push(`Tone: ${params.brief.tone}`);
    lines.push(`Planned sections: ${params.brief.sections.join(", ")}`);
  }

  if (params.stylePack) {
    lines.push(`Style pack: ${params.stylePack}`);
  }

  if (params.irSections.length > 0) {
    lines.push(`\nCurrent IR sections: ${params.irSections.join(", ")}`);
  }

  if (params.decisions.length > 0) {
    lines.push("\nRecent decisions:");
    // Only last 5 decisions to keep context compact
    const recent = params.decisions.slice(-5);
    for (const d of recent) {
      if (d.status === "active") {
        lines.push(`  - ${d.id}: ${d.decision}`);
      }
    }
  }

  lines.push(`\nConversation turns so far: ${params.turnCount}`);

  const summary = lines.join("\n");
  const estimatedTokens = Math.ceil(summary.length / 4);

  return { summary, estimatedTokens };
}
