// Wraps @voce-ir/mcp-server/tools' executeTool with cli-chat's interactive
// surfaces. Two interception points:
//
//   - voce_generate_propose: pause, show the readiness score, prompt the
//     user [Y/n/q]. Block the propose call if the user wants to keep
//     discovering.
//
//   - voce_check_drift result: if any drifts came back, walk each one with
//     [r/s/c]. The chosen path is logged to decisions.jsonl so the audit
//     trail captures user judgment.

import { executeTool, type ToolResult } from "@voce-ir/mcp-server/tools";
import { logDecision, type DriftReport } from "@voce-ir/mcp-server/memory";
import {
  driftResolutionAsDecision,
  formatDriftReport,
  parseDriftChoice,
  DRIFT_PROMPT,
} from "./drift-ui.js";
import {
  formatReadinessReport,
  parseReadinessChoice,
  READINESS_PROMPT,
} from "./readiness-ui.js";

export interface WrapDeps {
  /** Async readline question — returns the trimmed answer. */
  ask: (prompt: string) => Promise<string>;
  /** Where the user-facing UI text gets printed. */
  log: (msg: string) => void;
}

/**
 * Build a tool executor that intercepts propose/drift for interactive UX.
 * The returned function has the same signature as the bare executeTool
 * but is async and may pause for user input.
 */
export function wrapExecutor(
  deps: WrapDeps,
): (name: string, args: Record<string, unknown> | undefined) => Promise<ToolResult> {
  return async (name, args) => {
    if (name === "voce_generate_propose") {
      const sessionId = (args as { session_id?: string } | undefined)?.session_id;
      if (typeof sessionId === "string" && sessionId.length > 0) {
        const readinessRes = executeTool("voce_generation_readiness", { session_id: sessionId });
        const parsed = safeParse(readinessRes.content[0]?.text);
        if (parsed && typeof parsed === "object") {
          deps.log(formatReadinessReport(parsed as Parameters<typeof formatReadinessReport>[0]));
        } else {
          // Readiness lookup failed (e.g. session not found) — surface the
          // raw text so the user knows why and can answer the prompt.
          deps.log(readinessRes.content[0]?.text ?? "Readiness lookup failed.");
        }
        const answer = await deps.ask(READINESS_PROMPT);
        const choice = parseReadinessChoice(answer);
        if (choice === "abort") {
          return synthetic({
            ok: false,
            intercepted: "user_paused",
            message:
              "User declined to proceed with the proposal. Continue discovery — " +
              "do not retry voce_generate_propose until the user says ready.",
          });
        }
        if (choice === "ask-question") {
          const question = await deps.ask("What do you want me to ask the user? ");
          return synthetic({
            ok: false,
            intercepted: "user_wants_question_first",
            user_followup: question,
            message:
              "User wants the model to ask a clarifying question first. Address user_followup, " +
              "then voce_generate_answer with their reply, then re-evaluate readiness.",
          });
        }
        // proceed → fall through and run the real propose
      }
    }

    const result = await Promise.resolve(executeTool(name, args));

    if (name === "voce_check_drift") {
      await handleDriftResolution(result, deps);
    }

    return result;
  };
}

async function handleDriftResolution(result: ToolResult, deps: WrapDeps): Promise<void> {
  const parsed = safeParse(result.content[0]?.text);
  if (!parsed || typeof parsed !== "object") return;
  const drifts = (parsed as { drift?: DriftReport[] }).drift;
  if (!Array.isArray(drifts) || drifts.length === 0) return;

  for (const report of drifts) {
    deps.log(formatDriftReport(report));
    let choice: ReturnType<typeof parseDriftChoice> = null;
    while (choice === null) {
      const answer = await deps.ask(DRIFT_PROMPT);
      choice = parseDriftChoice(answer);
      if (choice === null) deps.log("Type r, s, or c.");
    }
    if (choice === "revise") {
      // No persisted record — the model will revise on the next turn.
      deps.log("Will ask the model to revise the IR.");
      continue;
    }
    const decisionInput = driftResolutionAsDecision(report, choice);
    const logged = logDecision(decisionInput);
    deps.log(`Logged ${choice} as decision [${logged.id.slice(0, 8)}].`);
  }
}

function synthetic(payload: unknown): ToolResult {
  return { content: [{ type: "text", text: JSON.stringify(payload) }] };
}

function safeParse(text: string | undefined): unknown {
  if (typeof text !== "string") return null;
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}
