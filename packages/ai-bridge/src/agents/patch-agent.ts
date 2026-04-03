/**
 * Patch Generator Agent — produces minimal JSON Patch operations
 * for incremental IR edits.
 *
 * Much cheaper than full regeneration: targets under 5K tokens per edit.
 */

import { ClaudeClient } from "../api/claude-client.js";
import { Agent } from "./base-agent.js";
import type { VocePatch, PatchOperation } from "../incremental/diff.js";

interface PatchInput {
  /** Current IR JSON (full). */
  currentIr: string;
  /** User's change request. */
  changeRequest: string;
  /** Brief context summary. */
  briefSummary: string;
}

export class PatchAgent extends Agent<PatchInput, VocePatch> {
  name = "patch";

  systemPrompt = `You are a surgical IR editor. Given a current Voce IR JSON and a change request, produce a minimal JSON Patch (RFC 6902) that implements the change.

Rules:
1. Touch ONLY what the user asked to change. Do not reorganize, restyle, or modify unrelated nodes.
2. Output a JSON object with: { "description": "...", "operations": [ { "op": "replace|add|remove", "path": "/...", "value": ... } ] }
3. Paths use JSON Pointer format: /root/children/0/value/content for the first child's content.
4. Preserve all node_ids, semantic references, and structural validity.
5. If adding an interactive element, include the SemanticNode addition.
6. Keep the patch as small as possible.

Output ONLY valid JSON. No other text.`;

  constructor(client: ClaudeClient) {
    super(client, { temperature: 0.1, maxTokens: 4096 });
  }

  buildUserPrompt(input: PatchInput): string {
    // Summarize the IR structure instead of sending the full JSON
    // to keep token count low
    const irSummary = summarizeIr(input.currentIr);

    return `Change request: "${input.changeRequest}"

Brief context: ${input.briefSummary}

Current IR structure:
${irSummary}

Full IR JSON:
${input.currentIr}

Generate the minimal JSON Patch to implement this change.`;
  }

  parseResponse(response: string): VocePatch {
    try {
      let json = response;
      // Extract from code fence if present
      const match = response.match(/```(?:json)?\s*\n([\s\S]*?)\n```/);
      if (match) json = match[1];

      const first = json.indexOf("{");
      const last = json.lastIndexOf("}");
      if (first !== -1 && last > first) {
        json = json.slice(first, last + 1);
      }

      const parsed = JSON.parse(json);
      return {
        description: parsed.description || "Edit",
        timestamp: new Date().toISOString(),
        operations: (parsed.operations || []).map((op: PatchOperation) => ({
          op: op.op,
          path: op.path,
          value: op.value,
          from: op.from,
        })),
      };
    } catch {
      return {
        description: "Failed to parse patch",
        timestamp: new Date().toISOString(),
        operations: [],
      };
    }
  }
}

/** Create a compact summary of the IR structure for the patch agent. */
function summarizeIr(irJson: string): string {
  try {
    const ir = JSON.parse(irJson);
    const lines: string[] = [];
    if (ir.root?.children) {
      for (let i = 0; i < ir.root.children.length; i++) {
        const child = ir.root.children[i];
        const type = child.value_type || "unknown";
        const id = child.value?.node_id || "?";
        const content = child.value?.content
          ? ` — "${child.value.content.slice(0, 40)}..."`
          : "";
        lines.push(`  [${i}] ${type} "${id}"${content}`);

        // One level of children
        if (child.value?.children) {
          for (let j = 0; j < child.value.children.length; j++) {
            const gc = child.value.children[j];
            const gcType = gc.value_type || "?";
            const gcId = gc.value?.node_id || "?";
            lines.push(`    [${i}/${j}] ${gcType} "${gcId}"`);
          }
        }
      }
    }
    return lines.join("\n") || "(empty IR)";
  } catch {
    return "(could not parse IR)";
  }
}
