/**
 * Decision log — records every significant design/architecture decision
 * with rationale, alternatives, and timestamp.
 *
 * Decisions form a traceable chain. New decisions are checked for
 * conflicts with existing ones.
 */

import { existsSync, readFileSync, writeFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { vocePath, ensureVoceDir } from "./directory.js";

export interface Decision {
  /** Unique ID (e.g., "D001"). */
  id: string;
  /** ISO timestamp. */
  date: string;
  /** Category: "design", "architecture", "content", "feature". */
  category: string;
  /** What was decided. */
  decision: string;
  /** Why this choice was made. */
  rationale: string;
  /** What other options were considered. */
  alternatives: string[];
  /** What this decision implies for future work. */
  implications: string[];
  /** "active" or ID of the decision that supersedes this one. */
  status: string;
}

/** Append a new decision to the log. Returns the assigned ID. */
export function appendDecision(
  decision: Omit<Decision, "id" | "date" | "status">,
  root: string = "."
): string {
  ensureVoceDir(root);

  const existing = listDecisions(root);
  const nextNum = existing.length + 1;
  const id = `D${String(nextNum).padStart(3, "0")}`;
  const date = new Date().toISOString();
  const slug = decision.decision
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .slice(0, 40);

  const entry: Decision = {
    ...decision,
    id,
    date,
    status: "active",
  };

  const filename = `${id}-${slug}.yaml`;
  const yaml = decisionToYaml(entry);
  writeFileSync(join(vocePath("decisions", root), filename), yaml);

  return id;
}

/** List all decisions, sorted by ID. */
export function listDecisions(root: string = "."): Decision[] {
  const dir = vocePath("decisions", root);
  if (!existsSync(dir)) return [];

  return readdirSync(dir)
    .filter((f) => f.endsWith(".yaml"))
    .sort()
    .map((f) => {
      const content = readFileSync(join(dir, f), "utf-8");
      return parseDecisionYaml(content);
    })
    .filter((d): d is Decision => d !== null);
}

/** Find decisions that might conflict with a new one. */
export function findConflicts(
  newDecision: string,
  existing: Decision[]
): Array<{ decision: Decision; reason: string }> {
  const conflicts: Array<{ decision: Decision; reason: string }> = [];
  const newLower = newDecision.toLowerCase();

  for (const d of existing) {
    if (d.status !== "active") continue;

    const existingLower = d.decision.toLowerCase();

    // Simple heuristic: check for contradictory keywords
    const contradictions = [
      ["minimal", "heavy"],
      ["dark", "light"],
      ["simple", "complex"],
      ["single", "multi"],
      ["remove", "add"],
      ["no animation", "animation"],
    ];

    for (const [a, b] of contradictions) {
      if (
        (newLower.includes(a) && existingLower.includes(b)) ||
        (newLower.includes(b) && existingLower.includes(a))
      ) {
        conflicts.push({
          decision: d,
          reason: `New decision mentions "${a}" but ${d.id} mentions "${b}"`,
        });
      }
    }
  }

  return conflicts;
}

/** Supersede an existing decision. */
export function supersedeDecision(
  id: string,
  supersededById: string,
  root: string = "."
): void {
  const dir = vocePath("decisions", root);
  const files = readdirSync(dir).filter((f) => f.startsWith(id));
  for (const file of files) {
    const path = join(dir, file);
    let content = readFileSync(path, "utf-8");
    content = content.replace(
      /status: "active"/,
      `status: "superseded by ${supersededById}"`
    );
    writeFileSync(path, content);
  }
}

function decisionToYaml(d: Decision): string {
  return `# Decision ${d.id}
id: "${d.id}"
date: "${d.date}"
category: "${d.category}"
decision: "${escYaml(d.decision)}"
rationale: "${escYaml(d.rationale)}"
alternatives:
${d.alternatives.map((a) => `  - "${escYaml(a)}"`).join("\n") || "  []"}
implications:
${d.implications.map((i) => `  - "${escYaml(i)}"`).join("\n") || "  []"}
status: "${d.status}"
`;
}

function parseDecisionYaml(content: string): Decision | null {
  try {
    const get = (key: string): string => {
      const match = content.match(new RegExp(`${key}:\\s*"([^"]*)"`));
      return match?.[1] || "";
    };
    const getArray = (key: string): string[] => {
      const section = content.match(
        new RegExp(`${key}:\\n((?:\\s+-\\s+"[^"]*"\\n?)*)`)
      );
      if (!section) return [];
      return [...section[1].matchAll(/-\s+"([^"]*)"/g)].map((m) => m[1]);
    };

    return {
      id: get("id"),
      date: get("date"),
      category: get("category"),
      decision: get("decision"),
      rationale: get("rationale"),
      alternatives: getArray("alternatives"),
      implications: getArray("implications"),
      status: get("status") || "active",
    };
  } catch {
    return null;
  }
}

function escYaml(s: string): string {
  return s.replace(/"/g, '\\"');
}
