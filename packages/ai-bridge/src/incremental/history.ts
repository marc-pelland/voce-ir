/**
 * Patch history — stores patches for undo/redo and time-travel.
 *
 * Patches are stored in .voce/history/ as numbered JSON files.
 * `voce undo` reverts the last patch by applying its inverse.
 */

import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { vocePath, ensureVoceDir } from "../memory/directory.js";
import type { VocePatch } from "./diff.js";

const HISTORY_DIR = "history";
const MAX_PATCHES = 50;

/** Save a patch to history. Returns the patch number. */
export function savePatch(patch: VocePatch, root: string = "."): number {
  ensureVoceDir(root);
  const dir = vocePath(HISTORY_DIR, root);
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });

  const existing = listPatches(root);
  const num = existing.length + 1;

  const filename = `${String(num).padStart(4, "0")}.json`;
  writeFileSync(join(dir, filename), JSON.stringify(patch, null, 2));

  // Prune old patches
  if (existing.length >= MAX_PATCHES) {
    pruneOldest(dir, existing.length - MAX_PATCHES + 1);
  }

  return num;
}

/** Load a specific patch by number. */
export function loadPatch(num: number, root: string = "."): VocePatch | null {
  const dir = vocePath(HISTORY_DIR, root);
  const filename = `${String(num).padStart(4, "0")}.json`;
  const path = join(dir, filename);

  if (!existsSync(path)) return null;
  return JSON.parse(readFileSync(path, "utf-8"));
}

/** List all patches in order. */
export function listPatches(root: string = "."): Array<{ num: number; patch: VocePatch }> {
  const dir = vocePath(HISTORY_DIR, root);
  if (!existsSync(dir)) return [];

  return readdirSync(dir)
    .filter((f) => f.endsWith(".json"))
    .sort()
    .map((f) => {
      const num = parseInt(f.replace(".json", ""), 10);
      const patch: VocePatch = JSON.parse(readFileSync(join(dir, f), "utf-8"));
      return { num, patch };
    });
}

/** Get the latest patch number. */
export function latestPatchNum(root: string = "."): number {
  const patches = listPatches(root);
  return patches.length > 0 ? patches[patches.length - 1].num : 0;
}

function pruneOldest(dir: string, count: number): void {
  const files = readdirSync(dir).filter((f) => f.endsWith(".json")).sort();
  for (let i = 0; i < count && i < files.length; i++) {
    const { unlinkSync } = require("node:fs");
    unlinkSync(join(dir, files[i]));
  }
}
