// Atomic-write primitives for the .voce/ store. Every public write goes through
// these helpers so a partial flush can never leave a half-written file or a
// truncated JSONL line behind.

import {
  closeSync,
  existsSync,
  fsyncSync,
  mkdirSync,
  openSync,
  readFileSync,
  renameSync,
  unlinkSync,
  writeSync,
} from "node:fs";
import { dirname, join } from "node:path";

/**
 * Write `content` to `targetPath` atomically.
 * Strategy: write to a sibling temp file, fsync, rename over target.
 * The rename on POSIX is atomic — readers see either the old or new file,
 * never a partial write.
 */
export function atomicWriteFile(targetPath: string, content: string): void {
  const dir = dirname(targetPath);
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });

  const tmp = join(dir, `.${Date.now()}-${process.pid}.tmp`);
  const fd = openSync(tmp, "w", 0o644);
  try {
    writeSync(fd, content);
    fsyncSync(fd);
  } finally {
    closeSync(fd);
  }
  try {
    renameSync(tmp, targetPath);
  } catch (err) {
    try { unlinkSync(tmp); } catch { /* best effort */ }
    throw err;
  }
}

/**
 * Append a single JSONL line to `targetPath`. The line is the JSON encoding
 * of `entry` followed by `\n`. Existing lines are never read or rewritten —
 * append-only is the contract. If the file does not exist, it is created.
 *
 * The whole append is fsynced so a crash mid-append cannot leave a torn
 * line. Multi-line writes are NOT supported through this helper — one call,
 * one line, on purpose.
 */
export function appendJsonlLine(targetPath: string, entry: unknown): void {
  const dir = dirname(targetPath);
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });

  const line = JSON.stringify(entry) + "\n";
  if (line.includes("\n", 0) && line.lastIndexOf("\n") !== line.length - 1) {
    // JSON.stringify never emits unescaped newlines, so this should be unreachable —
    // but if it ever happened it would corrupt the JSONL invariant, so guard.
    throw new Error("appendJsonlLine: serialized entry contains an embedded newline");
  }

  const fd = openSync(targetPath, "a", 0o644);
  try {
    writeSync(fd, line);
    fsyncSync(fd);
  } finally {
    closeSync(fd);
  }
}

/**
 * Read a JSONL file and parse each line. Lines that fail validation are
 * collected into the `errors` array — a corrupt entry never crashes the
 * read path; the storage layer is conservative.
 */
export function readJsonlFile<T>(
  targetPath: string,
  validate: (value: unknown) => string | null,
): { entries: T[]; errors: Array<{ line: number; reason: string }> } {
  if (!existsSync(targetPath)) return { entries: [], errors: [] };

  const raw = readFileSync(targetPath, "utf8");
  const lines = raw.split("\n");
  const entries: T[] = [];
  const errors: Array<{ line: number; reason: string }> = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (line.length === 0) continue;
    let parsed: unknown;
    try {
      parsed = JSON.parse(line);
    } catch (err) {
      errors.push({ line: i + 1, reason: `parse error: ${(err as Error).message}` });
      continue;
    }
    const reason = validate(parsed);
    if (reason !== null) {
      errors.push({ line: i + 1, reason });
      continue;
    }
    entries.push(parsed as T);
  }

  return { entries, errors };
}
