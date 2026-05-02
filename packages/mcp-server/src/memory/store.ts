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

export const PATHS = {
  brief: () => join(voceDir(), "brief.md"),
  decisions: () => join(voceDir(), "decisions.jsonl"),
  driftWarnings: () => join(voceDir(), "drift-warnings.jsonl"),
  userProfile: () => join(voceDir(), "user-profile.md"),
  sessionsDir: () => join(voceDir(), "sessions"),
  session: (id: string) => join(voceDir(), "sessions", `${id}.jsonl`),
};
