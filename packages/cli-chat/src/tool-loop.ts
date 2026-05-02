// Tool-use loop. Replaces the Day 1 messages.stream text-only flow.
//
// Each turn:
//   1. Send accumulated messages + tool definitions to the model.
//   2. Walk the response content. Print text blocks. For each tool_use
//      block, run the executor and stash the result.
//   3. If any tool_use blocks ran, append the assistant turn + a new user
//      turn carrying tool_result blocks, then loop. If only text came back,
//      append the assistant turn and return.
//
// The loop is intentionally non-streaming — Day 2 prioritizes correctness
// over streamed UX. A future day can layer messages.stream back in once the
// surrounding state machine is stable.

import type Anthropic from "@anthropic-ai/sdk";
import type { ToolDefinition, ToolResult } from "@voce-ir/mcp-server/tools";

export interface LoopMessage {
  role: "user" | "assistant";
  /** Anthropic's content block array. Strings are normalized to text blocks at the boundary. */
  content: ContentBlock[];
}

export type ContentBlock =
  | { type: "text"; text: string }
  | { type: "tool_use"; id: string; name: string; input: unknown }
  | { type: "tool_result"; tool_use_id: string; content: string };

export interface ToolEvent {
  name: string;
  input: unknown;
  result: ToolResult;
}

export interface LoopOpts {
  client: Anthropic;
  model: string;
  system: string;
  /** Conversation so far. The loop appends assistant + user (tool_result) turns to this. */
  messages: LoopMessage[];
  /** Tool definitions in MCP shape — converted to Anthropic shape internally. */
  tools: readonly ToolDefinition[];
  /** Tool runner — sync OR async. cli-chat wraps it to gate proposes on
   *  readiness UI and surface drift push-back, both of which need to await
   *  user input. The MCP server passes a sync version. */
  executor: (
    name: string,
    args: Record<string, unknown> | undefined,
  ) => ToolResult | Promise<ToolResult>;
  /** Hook invoked for each text chunk from the model. */
  onText?: (text: string) => void;
  /** Hook invoked when the model decides to call a tool. */
  onToolUse?: (event: { name: string; input: unknown }) => void;
  /** Hook invoked after each tool result is gathered. */
  onToolResult?: (event: ToolEvent) => void;
  /** Hook invoked after each model turn returns, with the SDK's usage stats. */
  onUsage?: (usage: {
    input_tokens?: number;
    output_tokens?: number;
    cache_read_input_tokens?: number | null;
    cache_creation_input_tokens?: number | null;
  }) => void;
  /** Safety cap so a runaway loop terminates. Default 12 — enough for the full
   *  start → 5 answers → propose → finalize chain plus a few sanity checks. */
  maxTurns?: number;
  /** Abort the in-flight request when the user hits Ctrl+C. The loop also
   *  short-circuits between turns so an abort fired during tool execution
   *  is honored on the next iteration. */
  signal?: AbortSignal;
}

/** Marker thrown when the loop is aborted. Callers re-prompt; do not crash. */
export class ToolLoopAborted extends Error {
  constructor() {
    super("voce-chat tool-use loop aborted");
    this.name = "ToolLoopAborted";
  }
}

export interface LoopResult {
  /** Concatenation of every text block the model emitted, across all turns. */
  finalText: string;
  /** The most recent IR snapshot the model proposed via voce_generate_propose,
   *  voce_generate_refine, or by emitting a ```json fence. null if none. */
  capturedIr: string | null;
  /** All tool calls in order. Useful for /cost telemetry and tests. */
  toolEvents: ToolEvent[];
  /** True when the loop exited because the model produced no tool_use. */
  completed: boolean;
}

/** Run the loop. Mutates `opts.messages` so the caller's history reflects the conversation. */
export async function runToolLoop(opts: LoopOpts): Promise<LoopResult> {
  const maxTurns = opts.maxTurns ?? 12;
  const tools = opts.tools.map((t) => ({
    name: t.name,
    description: t.description,
    input_schema: t.inputSchema,
  }));

  const events: ToolEvent[] = [];
  let finalText = "";
  let capturedIr: string | null = null;

  for (let turn = 0; turn < maxTurns; turn++) {
    if (opts.signal?.aborted) throw new ToolLoopAborted();

    // The Anthropic SDK accepts content arrays directly — our LoopMessage
    // shape is structurally identical aside from the tool_result branch.
    // System + tools are stable across turns within a session — mark them
    // with ephemeral cache_control so subsequent turns hit the cache. The
    // tools array gets the marker on its last entry; the SDK applies it
    // to the whole tool list.
    const systemBlocks = [
      { type: "text" as const, text: opts.system, cache_control: { type: "ephemeral" as const } },
    ];
    const toolsWithCache = tools.map((t, i) =>
      i === tools.length - 1
        ? { ...t, cache_control: { type: "ephemeral" as const } }
        : t,
    );

    const response = await opts.client.messages.create(
      {
        model: opts.model,
        max_tokens: 4096,
        // Cast: SDK accepts both string and content-array forms; we use the
        // array form so the ephemeral cache_control marker can attach.
        system: systemBlocks as unknown as Parameters<
          typeof opts.client.messages.create
        >[0]["system"],
        tools: toolsWithCache as unknown as Parameters<
          typeof opts.client.messages.create
        >[0]["tools"],
        messages: opts.messages as unknown as Parameters<
          typeof opts.client.messages.create
        >[0]["messages"],
      },
      opts.signal !== undefined ? { signal: opts.signal } : undefined,
    );

    if (opts.onUsage !== undefined && response.usage !== undefined) {
      opts.onUsage(response.usage);
    }

    const assistantBlocks: ContentBlock[] = [];
    const toolResults: ContentBlock[] = [];

    for (const block of response.content) {
      if (block.type === "text") {
        opts.onText?.(block.text);
        finalText += block.text;
        const ir = extractIrFromText(block.text);
        if (ir !== null) capturedIr = ir;
        assistantBlocks.push({ type: "text", text: block.text });
      } else if (block.type === "tool_use") {
        const result = await opts.executor(block.name, block.input as Record<string, unknown>);
        const event = {
          name: block.name,
          input: block.input,
          result,
        };
        events.push(event);
        opts.onToolUse?.({ name: block.name, input: block.input });
        opts.onToolResult?.(event);
        if (block.name === "voce_generate_propose" || block.name === "voce_generate_refine") {
          const irFromInput = (block.input as { ir_json?: string })?.ir_json;
          if (typeof irFromInput === "string") capturedIr = irFromInput;
        }
        assistantBlocks.push({
          type: "tool_use",
          id: block.id,
          name: block.name,
          input: block.input,
        });
        toolResults.push({
          type: "tool_result",
          tool_use_id: block.id,
          content: event.result.content.map((c) => c.text).join("\n"),
        });
      }
      // Other block types (e.g. thinking) are ignored on Day 2.
    }

    opts.messages.push({ role: "assistant", content: assistantBlocks });

    if (toolResults.length === 0) {
      // No tool calls — the model is done.
      return { finalText, capturedIr, toolEvents: events, completed: true };
    }

    if (opts.signal?.aborted) throw new ToolLoopAborted();
    opts.messages.push({ role: "user", content: toolResults });
  }

  // Hit the cap. Return what we have; caller can decide whether to warn.
  return { finalText, capturedIr, toolEvents: events, completed: false };
}

/** Pull a JSON IR out of a text block when the model emits one inline. */
function extractIrFromText(text: string): string | null {
  const match = text.match(/```json\s*\n([\s\S]*?)\n```/);
  if (!match) return null;
  try {
    JSON.parse(match[1]);
    return match[1];
  } catch {
    return null;
  }
}
