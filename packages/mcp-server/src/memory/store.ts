// Resolves the path to the .voce/ store. The MCP server inherits cwd from its
// host (the project root the agent is operating against). VOCE_PROJECT_ROOT
// overrides — used by tests, also useful when the host's cwd is unrelated.

import { existsSync, mkdirSync } from "node:fs";
import { join } from "node:path";

export function voceDir(): string {
  const root = process.env.VOCE_PROJECT_ROOT ?? process.cwd();
  return join(root, ".voce");
}

export function ensureVoceDir(): string {
  const dir = voceDir();
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
  return dir;
}

// Session ids are UUIDs produced by newSessionId() (randomUUID). Session ids
// arrive from tool arguments and `--resume`, so they are untrusted: without
// this check a value like "../../../tmp/x" escapes the sessions directory and
// reads or writes arbitrary *.jsonl files. Validate at the single path
// chokepoint and fail closed.
const SESSION_ID_RE =
  /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

/** Throw if `id` is not a well-formed session id (UUID). */
export function assertSafeSessionId(id: string): void {
  if (typeof id !== "string" || !SESSION_ID_RE.test(id)) {
    throw new Error(`Invalid session id: ${JSON.stringify(id)}`);
  }
}

export const PATHS = {
  brief: () => join(voceDir(), "brief.md"),
  decisions: () => join(voceDir(), "decisions.jsonl"),
  driftWarnings: () => join(voceDir(), "drift-warnings.jsonl"),
  userProfile: () => join(voceDir(), "user-profile.md"),
  sessionsDir: () => join(voceDir(), "sessions"),
  session: (id: string) => {
    assertSafeSessionId(id);
    return join(voceDir(), "sessions", `${id}.jsonl`);
  },
};
