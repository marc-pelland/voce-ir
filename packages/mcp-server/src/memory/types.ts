// Memory contract for the .voce/ store. See packages/mcp-server/.voce.SCHEMA.md
// (mirrored to .voce/SCHEMA.md at the repo root) for the durable spec.
//
// All log entries are timestamped ISO-8601 strings. JSONL files are append-only;
// the storage layer enforces that invariant — callers cannot rewrite history.

/** A recorded design / architecture / product decision. */
export interface Decision {
  id: string;
  timestamp: string;
  summary: string;
  rationale: string;
  supersedes?: string;
  conflicts_with?: string;
}

/** A drift warning — a proposed IR conflicts with a prior decision. */
export interface DriftWarning {
  timestamp: string;
  decision_id: string;
  drift_description: string;
  resolution: "pending" | "accepted" | "overridden";
}

/** A single conversation turn captured in a session log. */
export interface SessionEntry {
  timestamp: string;
  role: "user" | "assistant" | "system" | "tool";
  /** Free-form payload — text, tool name + args, etc. */
  content: string;
  /** Optional: tool name when role === "tool". */
  tool?: string;
  /**
   * Optional: a snapshot of the proposed IR at this turn. Set by the agent
   * (typically on assistant turns that emit IR). voce_session_resume returns
   * the most recent ir_snapshot as `current_ir` so a fresh session can pick
   * up exactly where the prior one left off.
   */
  ir_snapshot?: string;
}

// ─── Runtime validators ───────────────────────────────────────────
// Hand-rolled predicates rather than zod — keeps the dep footprint
// minimal. Each validator returns null on success or a string error.

export function validateDecision(value: unknown): string | null {
  if (!isPlainObject(value)) return "not an object";
  const v = value as Record<string, unknown>;
  if (typeof v.id !== "string" || v.id.length === 0) return "id missing or not a non-empty string";
  if (typeof v.timestamp !== "string" || !isIsoTimestamp(v.timestamp)) return "timestamp not an ISO-8601 string";
  if (typeof v.summary !== "string" || v.summary.length === 0) return "summary missing or empty";
  if (typeof v.rationale !== "string" || v.rationale.length === 0) return "rationale missing or empty";
  if (v.supersedes !== undefined && typeof v.supersedes !== "string") return "supersedes must be a string";
  if (v.conflicts_with !== undefined && typeof v.conflicts_with !== "string") return "conflicts_with must be a string";
  return null;
}

export function validateDriftWarning(value: unknown): string | null {
  if (!isPlainObject(value)) return "not an object";
  const v = value as Record<string, unknown>;
  if (typeof v.timestamp !== "string" || !isIsoTimestamp(v.timestamp)) return "timestamp not an ISO-8601 string";
  if (typeof v.decision_id !== "string" || v.decision_id.length === 0) return "decision_id missing or empty";
  if (typeof v.drift_description !== "string" || v.drift_description.length === 0) return "drift_description missing or empty";
  if (v.resolution !== "pending" && v.resolution !== "accepted" && v.resolution !== "overridden") {
    return "resolution must be one of: pending | accepted | overridden";
  }
  return null;
}

export function validateSessionEntry(value: unknown): string | null {
  if (!isPlainObject(value)) return "not an object";
  const v = value as Record<string, unknown>;
  if (typeof v.timestamp !== "string" || !isIsoTimestamp(v.timestamp)) return "timestamp not an ISO-8601 string";
  if (v.role !== "user" && v.role !== "assistant" && v.role !== "system" && v.role !== "tool") {
    return "role must be one of: user | assistant | system | tool";
  }
  if (typeof v.content !== "string") return "content must be a string";
  if (v.tool !== undefined && typeof v.tool !== "string") return "tool must be a string";
  if (v.ir_snapshot !== undefined && typeof v.ir_snapshot !== "string") return "ir_snapshot must be a string";
  return null;
}

function isPlainObject(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

function isIsoTimestamp(v: string): boolean {
  // Cheap check — full RFC 3339 is overkill. We only emit ISO via toISOString().
  const d = new Date(v);
  return !Number.isNaN(d.getTime()) && d.toISOString() === v;
}
