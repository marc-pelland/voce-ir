// Session log — one JSONL file per session at .voce/sessions/<session-id>.jsonl.
// Append-only. Used for crash recovery and conversation replay (voce_session_resume
// in S65 Day 3). The sessions/ directory is gitignored — these are working state.

import { existsSync, mkdirSync, readdirSync, statSync } from "node:fs";
import { randomUUID } from "node:crypto";
import { appendJsonlLine, readJsonlFile } from "./atomic.js";
import { PATHS } from "./store.js";
import type { SessionEntry } from "./types.js";
import { validateSessionEntry } from "./types.js";

export interface AppendSessionInput {
  role: SessionEntry["role"];
  content: string;
  tool?: string;
  ir_snapshot?: string;
  timestamp?: string;
}

/** Generates a fresh session id. */
export function newSessionId(): string {
  return randomUUID();
}

/** Append a single entry. Returns the persisted entry. */
export function appendSession(sessionId: string, input: AppendSessionInput): SessionEntry {
  ensureSessionsDir();
  const entry: SessionEntry = {
    timestamp: input.timestamp ?? new Date().toISOString(),
    role: input.role,
    content: input.content,
    ...(input.tool !== undefined ? { tool: input.tool } : {}),
    ...(input.ir_snapshot !== undefined ? { ir_snapshot: input.ir_snapshot } : {}),
  };
  const reason = validateSessionEntry(entry);
  if (reason !== null) {
    throw new Error(`appendSession: invalid input — ${reason}`);
  }
  appendJsonlLine(PATHS.session(sessionId), entry);
  return entry;
}

export function readSession(sessionId: string): SessionEntry[] {
  const { entries } = readJsonlFile<SessionEntry>(PATHS.session(sessionId), validateSessionEntry);
  return entries;
}

/**
 * Returns the most recent ir_snapshot stored in this session, or null if
 * none of the entries carry one. Used by voce_session_resume to surface
 * `current_ir` so a resumed session can pick up where the prior one left off.
 */
export function latestIrSnapshot(sessionId: string): string | null {
  const entries = readSession(sessionId);
  for (let i = entries.length - 1; i >= 0; i--) {
    const snap = entries[i]?.ir_snapshot;
    if (typeof snap === "string") return snap;
  }
  return null;
}

export interface SessionSummary {
  id: string;
  /** ISO-8601 of the most recent entry. */
  last_modified: string;
  entry_count: number;
}

/** List all sessions with metadata, most-recent-first. */
export function listSessions(): SessionSummary[] {
  const dir = PATHS.sessionsDir();
  if (!existsSync(dir)) return [];
  const summaries: SessionSummary[] = [];
  for (const file of readdirSync(dir)) {
    if (!file.endsWith(".jsonl")) continue;
    const id = file.slice(0, -".jsonl".length);
    const fullPath = PATHS.session(id);
    const stat = statSync(fullPath);
    const entries = readSession(id);
    summaries.push({
      id,
      last_modified: stat.mtime.toISOString(),
      entry_count: entries.length,
    });
  }
  summaries.sort((a, b) => b.last_modified.localeCompare(a.last_modified));
  return summaries;
}

function ensureSessionsDir(): void {
  const dir = PATHS.sessionsDir();
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
}
