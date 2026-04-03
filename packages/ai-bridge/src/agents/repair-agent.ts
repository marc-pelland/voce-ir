/**
 * Repair Agent — fixes validation errors in generated IR.
 * Runs in a loop: fix → validate → repeat (max 3 cycles).
 * Target: >99% validity after 2 cycles.
 */

import { execSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

import { ClaudeClient } from "../api/claude-client.js";
import { Agent } from "./base-agent.js";
import type { RepairResult } from "./types.js";

interface RepairInput {
  irJson: string;
  errors: string[];
}

const MAX_REPAIR_CYCLES = 3;

export class RepairAgent extends Agent<RepairInput, RepairResult> {
  name = "repair";

  systemPrompt = `You are a Voce IR repair agent. You receive IR JSON with validation errors and must fix them.

Common fixes:
- Missing SemanticNode: Add a SemanticNode to ViewRoot.semantic_nodes with the referenced node_id, role, and label
- Missing keyboard_key on GestureHandler: Add keyboard_key: "Enter" for buttons, "Space" for toggles
- Missing reduced_motion on AnimationTransition: Add reduced_motion: { strategy: "Remove" }
- Heading level skip (h1 → h3): Change heading_level to maintain sequence
- Missing csrf_protected on ActionNode: Add csrf_protected: true
- Missing PageMetadata title: Add metadata with a descriptive title
- Duplicate node_id: Make IDs unique by appending a suffix

Output ONLY the corrected JSON. No explanation, no markdown, just the fixed JSON.`;

  constructor(client: ClaudeClient) {
    super(client, { temperature: 0.1, maxTokens: 8192 });
  }

  buildUserPrompt(input: RepairInput): string {
    return `Fix these validation errors in the IR JSON:

Errors:
${input.errors.map((e) => `- ${e}`).join("\n")}

IR JSON to fix:
${input.irJson}`;
  }

  parseResponse(response: string): RepairResult {
    // Extract JSON
    let irJson = response;
    try {
      JSON.parse(response);
    } catch {
      const match = response.match(/```(?:json)?\s*\n([\s\S]*?)\n```/);
      if (match) {
        irJson = match[1];
      } else {
        const first = response.indexOf("{");
        const last = response.lastIndexOf("}");
        if (first !== -1 && last > first) {
          irJson = response.slice(first, last + 1);
        }
      }
    }

    return {
      irJson,
      allFixed: true, // Will be updated by the repair loop
      remainingErrors: [],
      cyclesUsed: 1,
    };
  }

  /**
   * Run the full repair loop: fix → validate → repeat.
   */
  async repairLoop(irJson: string): Promise<RepairResult> {
    let currentIr = irJson;
    let bestIr = irJson;
    let bestErrorCount = Infinity;

    for (let cycle = 1; cycle <= MAX_REPAIR_CYCLES; cycle++) {
      // Validate current IR
      const errors = validateIr(currentIr);

      if (errors.length === 0) {
        return {
          irJson: currentIr,
          allFixed: true,
          remainingErrors: [],
          cyclesUsed: cycle - 1,
        };
      }

      if (errors.length < bestErrorCount) {
        bestErrorCount = errors.length;
        bestIr = currentIr;
      }

      // Attempt repair
      const result = await this.execute({ irJson: currentIr, errors });
      currentIr = result.irJson;

      // Validate after repair
      try {
        JSON.parse(currentIr);
      } catch {
        // Repair produced invalid JSON — revert to best
        currentIr = bestIr;
        continue;
      }
    }

    // Final validation
    const finalErrors = validateIr(currentIr);
    return {
      irJson: finalErrors.length <= bestErrorCount ? currentIr : bestIr,
      allFixed: finalErrors.length === 0,
      remainingErrors: finalErrors,
      cyclesUsed: MAX_REPAIR_CYCLES,
    };
  }
}

/** Validate IR JSON and return error messages. */
function validateIr(irJson: string): string[] {
  const tmpPath = join(tmpdir(), "voce-repair-tmp.voce.json");
  writeFileSync(tmpPath, irJson);

  try {
    execSync(`voce validate --format json "${tmpPath}"`, {
      encoding: "utf-8",
      stdio: ["pipe", "pipe", "pipe"],
    });
    return []; // No errors
  } catch (error: unknown) {
    try {
      const err = error as { stdout?: string };
      const result = JSON.parse(err.stdout || "{}");
      return (result.diagnostics || [])
        .filter((d: { severity: string }) => d.severity === "error")
        .map(
          (d: { code: string; message: string; path: string }) =>
            `${d.code}: ${d.message} (at ${d.path})`
        );
    } catch {
      return ["Validation failed"];
    }
  }
}
