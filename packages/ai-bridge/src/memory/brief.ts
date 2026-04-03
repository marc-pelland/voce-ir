/**
 * Brief persistence — save/load the project brief to .voce/brief.yaml.
 *
 * The brief is the "north star" — every generation request is checked
 * against it. Changes require explicit confirmation.
 */

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { vocePath, ensureVoceDir } from "./directory.js";
import type { DiscoveryBrief } from "../agents/types.js";

export interface SavedBrief {
  version: number;
  created: string;
  lastModified: string;
  stylePack: string | null;
  brief: DiscoveryBrief;
}

/** Save a brief to .voce/brief.yaml. */
export function saveBrief(
  brief: DiscoveryBrief,
  stylePack: string | null = null,
  root: string = "."
): void {
  ensureVoceDir(root);

  const existing = loadBrief(root);
  const now = new Date().toISOString();

  const saved: SavedBrief = {
    version: existing ? existing.version + 1 : 1,
    created: existing?.created || now,
    lastModified: now,
    stylePack,
    brief,
  };

  const yaml = briefToYaml(saved);
  writeFileSync(vocePath("brief.yaml", root), yaml);
}

/** Load the brief from .voce/brief.yaml. Returns null if not found. */
export function loadBrief(root: string = "."): SavedBrief | null {
  const path = vocePath("brief.yaml", root);
  if (!existsSync(path)) return null;

  try {
    const content = readFileSync(path, "utf-8");
    return parseBriefYaml(content);
  } catch {
    return null;
  }
}

/** Check if a brief exists. */
export function hasBrief(root: string = "."): boolean {
  return existsSync(vocePath("brief.yaml", root));
}

/** Convert SavedBrief to YAML string. */
function briefToYaml(saved: SavedBrief): string {
  const b = saved.brief;
  return `# Voce IR Project Brief
# Version: ${saved.version}
# Created: ${saved.created}
# Last modified: ${saved.lastModified}

project:
  version: ${saved.version}
  created: "${saved.created}"
  last_modified: "${saved.lastModified}"

style_pack: ${saved.stylePack ? `"${saved.stylePack}"` : "null"}

vision: "${escYaml(b.purpose)}"

target_audience:
  primary: "${escYaml(b.audience)}"

sections:
${b.sections.map((s) => `  - "${escYaml(s)}"`).join("\n")}

ctas:
${b.ctas.map((c) => `  - "${escYaml(c)}"`).join("\n")}

tone: "${escYaml(b.tone)}"

constraints:
${b.constraints.length > 0 ? b.constraints.map((c) => `  - "${escYaml(c)}"`).join("\n") : "  []"}

readiness_score: ${b.readinessScore}
`;
}

/** Parse YAML brief back to SavedBrief. Simplified parser for our known format. */
function parseBriefYaml(content: string): SavedBrief {
  const get = (key: string): string => {
    const match = content.match(new RegExp(`${key}:\\s*"?([^"\\n]*)"?`));
    return match?.[1]?.trim() || "";
  };

  const getArray = (key: string): string[] => {
    const section = content.match(new RegExp(`${key}:\\n((?:\\s+-\\s+"[^"]*"\\n?)*)`));
    if (!section) return [];
    return [...section[1].matchAll(/-\s+"([^"]*)"/g)].map((m) => m[1]);
  };

  return {
    version: parseInt(get("version")) || 1,
    created: get("created"),
    lastModified: get("last_modified"),
    stylePack: get("style_pack") === "null" ? null : get("style_pack"),
    brief: {
      purpose: get("vision"),
      audience: get("primary"),
      sections: getArray("sections"),
      ctas: getArray("ctas"),
      tone: get("tone"),
      constraints: getArray("constraints"),
      readinessScore: parseInt(get("readiness_score")) || 0,
      followUpQuestions: [],
    },
  };
}

function escYaml(s: string): string {
  return s.replace(/"/g, '\\"');
}
