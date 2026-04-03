/**
 * Session persistence — saves/loads conversation state to .voce/sessions/.
 */

import { existsSync, mkdirSync, writeFileSync, readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const SESSIONS_DIR = ".voce/sessions";

export interface SessionData {
  id: string;
  timestamp: string;
  state: object;
}

/** Save conversation state to a session file. */
export function saveSession(state: object): string {
  const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
  const id = `session-${timestamp}`;
  const path = join(SESSIONS_DIR, `${id}.json`);

  if (!existsSync(SESSIONS_DIR)) {
    mkdirSync(SESSIONS_DIR, { recursive: true });
  }

  const data: SessionData = { id, timestamp: new Date().toISOString(), state };
  writeFileSync(path, JSON.stringify(data, null, 2));
  return id;
}

/** Load the most recent session. */
export function loadLatestSession(): SessionData | null {
  if (!existsSync(SESSIONS_DIR)) return null;

  const files = readdirSync(SESSIONS_DIR)
    .filter((f) => f.endsWith(".json"))
    .sort()
    .reverse();

  if (files.length === 0) return null;

  const data = readFileSync(join(SESSIONS_DIR, files[0]), "utf-8");
  return JSON.parse(data);
}

/** Load a specific session by ID. */
export function loadSession(id: string): SessionData | null {
  const path = join(SESSIONS_DIR, `${id}.json`);
  if (!existsSync(path)) return null;
  const data = readFileSync(path, "utf-8");
  return JSON.parse(data);
}
