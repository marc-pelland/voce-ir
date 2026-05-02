// One turn of the chat. Extracted from index.ts so tests can drive it
// without owning the readline loop or signal handling.
//
// processChatTurn pushes the user message into history, runs the tool-use
// loop with the wrapped (interactive) executor, persists every tool call
// to the session ledger, captures any IR snapshot, and rolls token usage
// into ChatState.

import type Anthropic from "@anthropic-ai/sdk";
import { appendSession } from "@voce-ir/mcp-server/memory";
import { TOOL_DEFINITIONS, type ToolResult } from "@voce-ir/mcp-server/tools";
import { runToolLoop, ToolLoopAborted, type LoopMessage } from "./tool-loop.js";
import type { ChatState } from "./commands/registry.js";
import { logAssistantTurn, logUserTurn } from "./session-manager.js";
import { pushIrHistory } from "./commands/ir.js";

export interface ChatFlowDeps {
  /** Wrapped executor — already includes the interactive readiness/drift UI. */
  executor: (
    name: string,
    args: Record<string, unknown> | undefined,
  ) => ToolResult | Promise<ToolResult>;
  /** Streaming text out — typically process.stdout.write. */
  onText?: (text: string) => void;
  /** Compact tool-name notification, e.g. "↳ voce_validate". */
  onToolUse?: (event: { name: string; input: unknown }) => void;
  /** Cancellable signal; the readline-side Ctrl+C handler fires the controller. */
  signal?: AbortSignal;
}

export interface ChatFlowResult {
  finalText: string;
  capturedIr: string | null;
  /** True when the loop exited normally (model produced no tool_use). */
  completed: boolean;
  /** True when ToolLoopAborted bubbled — the caller should re-prompt without printing an error. */
  aborted: boolean;
  /** Set when an unexpected error other than ToolLoopAborted came through. */
  error: Error | null;
}

export async function processChatTurn(
  state: ChatState,
  client: Anthropic,
  deps: ChatFlowDeps,
  userMessage: string,
): Promise<ChatFlowResult> {
  state.conversationHistory.push({
    role: "user",
    content: [{ type: "text", text: userMessage }],
  });
  logUserTurn(state.sessionId, userMessage);

  try {
    const result = await runToolLoop({
      client,
      model: state.model,
      system: state.systemPrompt,
      messages: state.conversationHistory as LoopMessage[],
      tools: TOOL_DEFINITIONS,
      executor: deps.executor,
      signal: deps.signal,
      onText: deps.onText,
      onToolUse: deps.onToolUse,
      onToolResult: (event) => {
        appendSession(state.sessionId, {
          role: "tool",
          tool: event.name,
          content: JSON.stringify({
            input: event.input,
            result: event.result.content[0]?.text ?? "",
            isError: event.result.isError ?? false,
          }),
        });
      },
      onUsage: (usage) => {
        state.tokenUsage.input += usage.input_tokens ?? 0;
        state.tokenUsage.output += usage.output_tokens ?? 0;
        state.tokenUsage.cache_read += usage.cache_read_input_tokens ?? 0;
        state.tokenUsage.cache_creation += usage.cache_creation_input_tokens ?? 0;
      },
    });

    if (result.capturedIr !== null) {
      pushIrHistory({ state }, result.capturedIr);
      logAssistantTurn(state.sessionId, result.finalText, result.capturedIr);
    } else {
      logAssistantTurn(state.sessionId, result.finalText);
    }

    return {
      finalText: result.finalText,
      capturedIr: result.capturedIr,
      completed: result.completed,
      aborted: false,
      error: null,
    };
  } catch (err) {
    if (err instanceof ToolLoopAborted) {
      return { finalText: "", capturedIr: null, completed: false, aborted: true, error: null };
    }
    return {
      finalText: "",
      capturedIr: null,
      completed: false,
      aborted: false,
      error: err as Error,
    };
  }
}
