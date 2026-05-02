// Project brief — the north-star markdown that scopes every generation.
// Hand-edited by the user OR AI-edited via voce_brief_set (S65 Day 3).

import { existsSync, readFileSync, statSync } from "node:fs";
import { atomicWriteFile } from "./atomic.js";
import { PATHS } from "./store.js";

export interface Brief {
  /** Markdown body. */
  content: string;
  /** ISO-8601 last-modified time of the file on disk. */
  last_modified: string;
}

/** Returns the brief, or null if the project has no brief.md yet. */
export function readBrief(): Brief | null {
  const path = PATHS.brief();
  if (!existsSync(path)) return null;
  const content = readFileSync(path, "utf8");
  const last_modified = statSync(path).mtime.toISOString();
  return { content, last_modified };
}

/**
 * Replace the brief with `content`. Atomic — either the new brief is fully
 * present or the old one is untouched. Callers (notably voce_brief_set) are
 * expected to confirm with the human before invoking this; the storage
 * layer does not prompt.
 */
export function writeBrief(content: string): void {
  atomicWriteFile(PATHS.brief(), content);
}
