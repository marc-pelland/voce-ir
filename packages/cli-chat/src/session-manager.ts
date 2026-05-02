// Session lifecycle for voce-chat. Resolves whether this invocation is
// resuming an existing session (and which one) vs. opening a fresh one.

import {
  appendSession,
  ensureVoceDir,
  listSessions,
  newSessionId,
  readSession,
} from "@voce-ir/mcp-server/memory";
import type { SessionEntry } from "@voce-ir/mcp-server/memory";

export interface ResolvedSession {
  id: string;
  /** True when an existing session was loaded, false when a fresh one was opened. */
  resumed: boolean;
  /** Pre-existing entries when resumed; empty array when fresh. */
  history: SessionEntry[];
}

export interface ResumeOpts {
  /** Pass the explicit session id; "auto" = pick the most recent; null = fresh session. */
  resume: string | "auto" | null;
}

/**
 * Resolve the session this voce-chat invocation should attach to. Always
 * touches `.voce/` on disk so the directory exists for the writes that follow.
 */
export function resolveSession(opts: ResumeOpts): ResolvedSession {
  ensureVoceDir();

  if (opts.resume === null) {
    return { id: newSessionId(), resumed: false, history: [] };
  }

  if (opts.resume === "auto") {
    const sessions = listSessions();
    const mostRecent = sessions[0]?.id;
    if (!mostRecent) {
      // --resume with nothing on disk just opens a fresh session.
      return { id: newSessionId(), resumed: false, history: [] };
    }
    return {
      id: mostRecent,
      resumed: true,
      history: readSession(mostRecent),
    };
  }

  // Explicit id — load it. If absent, fall back to a fresh session rather
  // than crashing so the user can carry on after a typo.
  const history = readSession(opts.resume);
  if (history.length === 0) {
    return { id: newSessionId(), resumed: false, history: [] };
  }
  return { id: opts.resume, resumed: true, history };
}

export function logUserTurn(sessionId: string, content: string): void {
  appendSession(sessionId, { role: "user", content });
}

export function logAssistantTurn(sessionId: string, content: string, irSnapshot?: string): void {
  appendSession(sessionId, {
    role: "assistant",
    content,
    ...(irSnapshot !== undefined ? { ir_snapshot: irSnapshot } : {}),
  });
}

export function logSystemEvent(sessionId: string, content: string): void {
  appendSession(sessionId, { role: "system", content });
}
