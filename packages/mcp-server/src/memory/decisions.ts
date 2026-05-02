// Decision log — append-only JSONL at .voce/decisions.jsonl.
// Each line is a Decision (see types.ts). Callers cannot rewrite history;
// supersession is expressed via the `supersedes` field, not by mutating the
// superseded entry.

import { randomUUID } from "node:crypto";
import { appendJsonlLine, readJsonlFile } from "./atomic.js";
import { PATHS } from "./store.js";
import type { Decision } from "./types.js";
import { validateDecision } from "./types.js";

export interface LogDecisionInput {
  summary: string;
  rationale: string;
  supersedes?: string;
  conflicts_with?: string;
  /** Optional override — defaults to a fresh UUID. */
  id?: string;
  /** Optional override — defaults to now. */
  timestamp?: string;
}

/** List decisions, oldest first. `since` is an ISO-8601 timestamp filter. */
export function listDecisions(opts: { since?: string } = {}): Decision[] {
  const { entries } = readJsonlFile<Decision>(PATHS.decisions(), validateDecision);
  if (opts.since === undefined) return entries;
  const cutoff = new Date(opts.since).getTime();
  if (Number.isNaN(cutoff)) return entries;
  return entries.filter((d) => new Date(d.timestamp).getTime() >= cutoff);
}

/** Returns the decision with the given id, or null if not found. */
export function getDecision(id: string): Decision | null {
  const { entries } = readJsonlFile<Decision>(PATHS.decisions(), validateDecision);
  return entries.find((d) => d.id === id) ?? null;
}

/** Append a new decision. Returns the persisted entry (with id + timestamp). */
export function logDecision(input: LogDecisionInput): Decision {
  const decision: Decision = {
    id: input.id ?? randomUUID(),
    timestamp: input.timestamp ?? new Date().toISOString(),
    summary: input.summary,
    rationale: input.rationale,
    ...(input.supersedes !== undefined ? { supersedes: input.supersedes } : {}),
    ...(input.conflicts_with !== undefined ? { conflicts_with: input.conflicts_with } : {}),
  };
  const reason = validateDecision(decision);
  if (reason !== null) {
    throw new Error(`logDecision: invalid input — ${reason}`);
  }
  appendJsonlLine(PATHS.decisions(), decision);
  return decision;
}
