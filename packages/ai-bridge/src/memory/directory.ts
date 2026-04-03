/**
 * Directory manager — initializes and manages the .voce/ structure.
 *
 * .voce/
 * ├── brief.yaml          (tracked in git)
 * ├── decisions/           (tracked in git)
 * │   └── *.yaml
 * ├── sessions/            (gitignored — ephemeral)
 * │   └── *.json
 * └── cache/               (gitignored)
 *     └── embeddings.json
 */

import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const VOCE_DIR = ".voce";

const SUBDIRS = ["decisions", "sessions", "memory", "cache", "snapshots"];

const GITIGNORE_CONTENT = `# Voce IR — ephemeral files
sessions/
cache/
`;

/** Ensure .voce/ directory exists with all subdirectories. */
export function ensureVoceDir(root: string = "."): string {
  const voceDir = join(root, VOCE_DIR);

  if (!existsSync(voceDir)) {
    mkdirSync(voceDir, { recursive: true });
  }

  for (const sub of SUBDIRS) {
    const subDir = join(voceDir, sub);
    if (!existsSync(subDir)) {
      mkdirSync(subDir, { recursive: true });
    }
  }

  // Write .gitignore for ephemeral subdirs
  const gitignorePath = join(voceDir, ".gitignore");
  if (!existsSync(gitignorePath)) {
    writeFileSync(gitignorePath, GITIGNORE_CONTENT);
  }

  return voceDir;
}

/** Get path to a .voce/ file. */
export function vocePath(filename: string, root: string = "."): string {
  return join(root, VOCE_DIR, filename);
}

/** Check if .voce/ directory exists. */
export function hasVoceDir(root: string = "."): boolean {
  return existsSync(join(root, VOCE_DIR));
}
