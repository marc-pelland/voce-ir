import { describe, expect, it } from "vitest";
import { CommandRegistry, type CommandContext } from "./registry.js";

function stubContext(overrides: Partial<CommandContext> = {}): CommandContext {
  const logs: string[] = [];
  const ctx: CommandContext = {
    state: {
      conversationHistory: [],
      currentIr: null,
      irHistory: [],
      sessionId: "test",
      model: "claude-test",
      systemPrompt: "",
      tokenUsage: { input: 0, output: 0, cache_read: 0, cache_creation: 0 },
    },
    client: null,
    voceBin: "voce",
    log: (m) => logs.push(m),
    exit: () => {},
    ...overrides,
  };
  // Attach the log array for assertions.
  (ctx as unknown as { logs: string[] }).logs = logs;
  return ctx;
}

describe("CommandRegistry.dispatch", () => {
  it("returns handled:false when input doesn't start with /", async () => {
    const r = new CommandRegistry();
    expect(await r.dispatch("hello", stubContext())).toEqual({ handled: false });
  });

  it("returns handled:false when the command name is unknown", async () => {
    const r = new CommandRegistry();
    expect(await r.dispatch("/nope", stubContext())).toEqual({ handled: false });
  });

  it("dispatches to the registered handler with the trailing rest", async () => {
    const r = new CommandRegistry();
    let captured = "";
    r.register({
      name: "echo",
      summary: "echo",
      handler: (rest) => {
        captured = rest;
        return { handled: true };
      },
    });
    await r.dispatch("/echo  hello world  ", stubContext());
    expect(captured).toBe("hello world");
  });

  it("aliases route to the same handler", async () => {
    const r = new CommandRegistry();
    let calls = 0;
    r.register({
      name: "exit",
      aliases: ["quit"],
      summary: "exit",
      handler: () => {
        calls += 1;
        return { handled: true };
      },
    });
    await r.dispatch("/exit", stubContext());
    await r.dispatch("/quit", stubContext());
    expect(calls).toBe(2);
  });

  it("matching is case-insensitive on the name", async () => {
    const r = new CommandRegistry();
    let called = false;
    r.register({ name: "show", summary: "x", handler: () => { called = true; return { handled: true }; } });
    await r.dispatch("/SHOW", stubContext());
    expect(called).toBe(true);
  });
});
