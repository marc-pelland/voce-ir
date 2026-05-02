// Drift log — append-only JSONL at .voce/drift-warnings.jsonl.
// Each entry references a decision_id from decisions.jsonl. Drift detection
// itself (computing the warnings) lives in voce_check_drift (S65 Day 3) —
// this file just owns persistence.

import { appendJsonlLine, readJsonlFile } from "./atomic.js";
import { PATHS } from "./store.js";
import type { DriftWarning } from "./types.js";
import { validateDriftWarning } from "./types.js";

export interface LogDriftInput {
  decision_id: string;
  drift_description: string;
  resolution?: DriftWarning["resolution"];
  timestamp?: string;
}

export function listDrift(opts: { since?: string } = {}): DriftWarning[] {
  const { entries } = readJsonlFile<DriftWarning>(PATHS.driftWarnings(), validateDriftWarning);
  if (opts.since === undefined) return entries;
  const cutoff = new Date(opts.since).getTime();
  if (Number.isNaN(cutoff)) return entries;
  return entries.filter((d) => new Date(d.timestamp).getTime() >= cutoff);
}

export function logDrift(input: LogDriftInput): DriftWarning {
  const entry: DriftWarning = {
    timestamp: input.timestamp ?? new Date().toISOString(),
    decision_id: input.decision_id,
    drift_description: input.drift_description,
    resolution: input.resolution ?? "pending",
  };
  const reason = validateDriftWarning(entry);
  if (reason !== null) {
    throw new Error(`logDrift: invalid input — ${reason}`);
  }
  appendJsonlLine(PATHS.driftWarnings(), entry);
  return entry;
}
