// Slash command framework for voce-chat. Each command is a small handler
// registered against one or more names. New commands plug in by adding a
// file and an entry in the registry.
//
// Handlers receive a CommandContext that exposes the chat's mutable state
// + the side-channels they need (Anthropic client, voce binary path,
// session id). The dispatcher is the only thing that knows about prompt
// echo, error formatting, etc. — handlers stay focused on logic.

import type Anthropic from "@anthropic-ai/sdk";
import type { LoopMessage } from "../tool-loop.js";

/** Mutable state shared across the chat loop and every command. */
export interface ChatState {
  conversationHistory: LoopMessage[];
  /** Most recent IR snapshot the model proposed; null when discovery hasn't produced one. */
  currentIr: string | null;
  /** Stack of prior IRs for /undo. Most recent first. */
  irHistory: string[];
  sessionId: string;
  model: string;
  systemPrompt: string;
  /** Cumulative token usage across the session. /cost reads this. */
  tokenUsage: {
    input: number;
    output: number;
    cache_read: number;
    cache_creation: number;
  };
}

export interface CommandContext {
  state: ChatState;
  client: Anthropic | null;
  voceBin: string;
  /** Print to the user. Indirection so tests can capture. */
  log: (message: string) => void;
  /** Triggered by /exit and /quit. Wired to process.exit in production. */
  exit: (code?: number) => void;
}

export type CommandResult =
  | { handled: true }
  | { handled: false }
  | { handled: true; submitText: string };

/** A handler may be sync or async. The first positional arg is the raw text after the command. */
export type CommandHandler = (
  rest: string,
  ctx: CommandContext,
) => CommandResult | Promise<CommandResult>;

export interface CommandSpec {
  /** Primary name without the leading slash. */
  name: string;
  /** Optional aliases (also without leading slash). */
  aliases?: readonly string[];
  /** One-line summary used by /help. */
  summary: string;
  handler: CommandHandler;
}

export class CommandRegistry {
  private readonly specs: CommandSpec[] = [];
  private readonly index = new Map<string, CommandSpec>();

  register(spec: CommandSpec): this {
    this.specs.push(spec);
    this.index.set(spec.name, spec);
    for (const alias of spec.aliases ?? []) this.index.set(alias, spec);
    return this;
  }

  list(): readonly CommandSpec[] {
    return this.specs;
  }

  /**
   * Dispatch an input line. Returns the handler's result, or {handled:false}
   * if the input doesn't match any registered command. Input must start with
   * '/' — the dispatcher does NOT strip whitespace; the caller controls that.
   */
  async dispatch(input: string, ctx: CommandContext): Promise<CommandResult> {
    if (!input.startsWith("/")) return { handled: false };
    const trimmed = input.slice(1).trim();
    if (trimmed.length === 0) return { handled: false };
    const spaceIdx = trimmed.indexOf(" ");
    const name = (spaceIdx === -1 ? trimmed : trimmed.slice(0, spaceIdx)).toLowerCase();
    const rest = spaceIdx === -1 ? "" : trimmed.slice(spaceIdx + 1).trim();
    const spec = this.index.get(name);
    if (!spec) return { handled: false };
    return spec.handler(rest, ctx);
  }
}
