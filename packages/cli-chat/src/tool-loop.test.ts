// Tool-use loop tests. The loop is exercised through a scripted mock
// Anthropic client + a deterministic executor — no network, no real model.

import { describe, expect, it, vi } from "vitest";
import { runToolLoop, type LoopMessage } from "./tool-loop.js";
import type { ToolDefinition, ToolResult } from "@voce-ir/mcp-server/tools";

interface ScriptedMessage {
  content: Array<
    | { type: "text"; text: string }
    | { type: "tool_use"; id: string; name: string; input: unknown }
  >;
  stop_reason?: string;
}

function mockClient(scripted: ScriptedMessage[]): {
  messages: { create: ReturnType<typeof vi.fn> };
  callCount: () => number;
} {
  let i = 0;
  const create = vi.fn(async () => {
    const next = scripted[i++];
    if (!next) throw new Error("scripted client exhausted");
    return next;
  });
  return {
    messages: { create },
    callCount: () => i,
  };
}

function mockExecutor(): {
  fn: (name: string, args: Record<string, unknown> | undefined) => ToolResult;
  calls: Array<{ name: string; args: Record<string, unknown> | undefined }>;
} {
  const calls: Array<{ name: string; args: Record<string, unknown> | undefined }> = [];
  const fn = (name: string, args: Record<string, unknown> | undefined): ToolResult => {
    calls.push({ name, args });
    if (name === "voce_generation_readiness") {
      return { content: [{ type: "text", text: JSON.stringify({ score: 80, ready: true }) }] };
    }
    return { content: [{ type: "text", text: JSON.stringify({ ok: true, name, args }) }] };
  };
  return { fn, calls };
}

const TOOLS: readonly ToolDefinition[] = [
  {
    name: "voce_generation_readiness",
    description: "test",
    inputSchema: { type: "object", properties: { session_id: { type: "string" } }, required: ["session_id"] },
  },
];

describe("runToolLoop", () => {
  it("exits after one turn when the model returns only text", async () => {
    const client = mockClient([
      { content: [{ type: "text", text: "Hi there" }] },
    ]);
    const exec = mockExecutor();
    const messages: LoopMessage[] = [{ role: "user", content: [{ type: "text", text: "ping" }] }];

    const result = await runToolLoop({
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      client: client as any,
      model: "claude-test",
      system: "system",
      messages,
      tools: TOOLS,
      executor: exec.fn,
    });

    expect(result.completed).toBe(true);
    expect(result.finalText).toBe("Hi there");
    expect(client.callCount()).toBe(1);
    expect(exec.calls).toHaveLength(0);
    // Loop appended the assistant turn to messages.
    expect(messages).toHaveLength(2);
    expect(messages[1]?.role).toBe("assistant");
  });

  it("runs an extra turn when a tool_use block appears, then completes", async () => {
    const client = mockClient([
      {
        content: [
          { type: "text", text: "checking readiness" },
          {
            type: "tool_use",
            id: "tu_1",
            name: "voce_generation_readiness",
            input: { session_id: "abc" },
          },
        ],
      },
      { content: [{ type: "text", text: "Ready to propose." }] },
    ]);
    const exec = mockExecutor();
    const messages: LoopMessage[] = [{ role: "user", content: [{ type: "text", text: "are we ready?" }] }];

    const result = await runToolLoop({
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      client: client as any,
      model: "claude-test",
      system: "system",
      messages,
      tools: TOOLS,
      executor: exec.fn,
    });

    expect(result.completed).toBe(true);
    expect(result.finalText).toBe("checking readinessReady to propose.");
    expect(exec.calls).toEqual([
      { name: "voce_generation_readiness", args: { session_id: "abc" } },
    ]);
    expect(client.callCount()).toBe(2);
    // user → assistant (with tool_use) → user (with tool_result) → assistant
    expect(messages).toHaveLength(4);
    expect(messages[1]?.content[0]?.type).toBe("text");
    expect(messages[1]?.content[1]?.type).toBe("tool_use");
    expect(messages[2]?.role).toBe("user");
    expect(messages[2]?.content[0]?.type).toBe("tool_result");
  });

  it("captures IR from voce_generate_propose tool args", async () => {
    const ir = JSON.stringify({ value_type: "ViewRoot", children: [] });
    const client = mockClient([
      {
        content: [
          {
            type: "tool_use",
            id: "tu_1",
            name: "voce_generate_propose",
            input: { session_id: "s1", ir_json: ir },
          },
        ],
      },
      { content: [{ type: "text", text: "Done." }] },
    ]);
    const exec = mockExecutor();

    const result = await runToolLoop({
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      client: client as any,
      model: "claude-test",
      system: "system",
      messages: [{ role: "user", content: [{ type: "text", text: "go" }] }],
      tools: [
        {
          name: "voce_generate_propose",
          description: "x",
          inputSchema: { type: "object", properties: {} },
        },
      ],
      executor: exec.fn,
    });

    expect(result.capturedIr).toBe(ir);
  });

  it("captures IR from a markdown ```json fence in text", async () => {
    const ir = JSON.stringify({ value_type: "ViewRoot" }, null, 2);
    const client = mockClient([
      {
        content: [
          {
            type: "text",
            text: `Here you go:\n\n\`\`\`json\n${ir}\n\`\`\`\n\nLet me know.`,
          },
        ],
      },
    ]);
    const exec = mockExecutor();

    const result = await runToolLoop({
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      client: client as any,
      model: "claude-test",
      system: "system",
      messages: [{ role: "user", content: [{ type: "text", text: "make it" }] }],
      tools: TOOLS,
      executor: exec.fn,
    });

    expect(result.capturedIr).toBe(ir);
  });

  it("respects the maxTurns cap when the model loops on tool calls", async () => {
    const looper: ScriptedMessage[] = Array.from({ length: 10 }, (_, i) => ({
      content: [
        {
          type: "tool_use" as const,
          id: `tu_${i}`,
          name: "voce_generation_readiness",
          input: { session_id: "x" },
        },
      ],
    }));
    const client = mockClient(looper);
    const exec = mockExecutor();

    const result = await runToolLoop({
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      client: client as any,
      model: "claude-test",
      system: "system",
      messages: [{ role: "user", content: [{ type: "text", text: "hi" }] }],
      tools: TOOLS,
      executor: exec.fn,
      maxTurns: 3,
    });

    expect(result.completed).toBe(false);
    expect(client.callCount()).toBe(3);
    expect(exec.calls).toHaveLength(3);
  });

  it("invokes onText, onToolUse, and onToolResult hooks", async () => {
    const client = mockClient([
      {
        content: [
          { type: "text", text: "let me check" },
          {
            type: "tool_use",
            id: "tu_1",
            name: "voce_generation_readiness",
            input: { session_id: "abc" },
          },
        ],
      },
      { content: [{ type: "text", text: "done" }] },
    ]);
    const exec = mockExecutor();
    const onText = vi.fn();
    const onToolUse = vi.fn();
    const onToolResult = vi.fn();

    await runToolLoop({
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      client: client as any,
      model: "claude-test",
      system: "system",
      messages: [{ role: "user", content: [{ type: "text", text: "x" }] }],
      tools: TOOLS,
      executor: exec.fn,
      onText,
      onToolUse,
      onToolResult,
    });

    expect(onText).toHaveBeenCalledWith("let me check");
    expect(onText).toHaveBeenCalledWith("done");
    expect(onToolUse).toHaveBeenCalledWith({
      name: "voce_generation_readiness",
      input: { session_id: "abc" },
    });
    expect(onToolResult).toHaveBeenCalledTimes(1);
    const event = onToolResult.mock.calls[0]?.[0];
    expect(event.name).toBe("voce_generation_readiness");
    expect(event.result.content[0].text).toContain("score");
  });
});
