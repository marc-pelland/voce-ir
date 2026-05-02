#!/usr/bin/env node
/**
 * Voce IR Conversational CLI — voce-chat.
 *
 * Tool-use loop (S66 Day 2) backed by .voce/ persistence (Day 1) and a
 * pluggable slash command surface (Day 3). Multi-line input via trailing
 * `\`. Ctrl+C cancels in-flight model responses without killing the process.
 *
 * Usage:
 *   voce-chat                          Start a fresh session
 *   voce-chat "build a nav bar"        Start with an initial prompt
 *   voce-chat --resume                 Resume the most recent session
 *   voce-chat --resume <session-id>    Resume a specific session
 *
 * Type /help in the REPL to list commands.
 */

import Anthropic from "@anthropic-ai/sdk";
import { createInterface } from "node:readline";
import { existsSync } from "node:fs";
import { join } from "node:path";
import chalk from "chalk";

import { appendSession, listDecisions, readBrief } from "@voce-ir/mcp-server/memory";
import { TOOL_DEFINITIONS, executeTool } from "@voce-ir/mcp-server/tools";

import { parseArgs } from "./cli-args.js";
import { buildSystemPrompt } from "./prompt.js";
import {
  logAssistantTurn,
  logSystemEvent,
  logUserTurn,
  resolveSession,
} from "./session-manager.js";
import { runToolLoop, ToolLoopAborted } from "./tool-loop.js";
import type { LoopMessage } from "./tool-loop.js";
import { clearPending, createAccumulator, feedLine } from "./multi-line.js";
import { buildRegistry, type ChatState, type CommandContext } from "./commands/index.js";
import { pushIrHistory } from "./commands/ir.js";

// ── Config ──────────────────────────────────────────────────────

const DEFAULT_MODEL = process.env.VOCE_MODEL || "claude-sonnet-4-20250514";
const API_KEY = process.env.ANTHROPIC_API_KEY;

function findVoceBin(): string {
  const candidates = [
    join(process.cwd(), "target/release/voce"),
    join(process.cwd(), "target/debug/voce"),
  ];
  for (const c of candidates) {
    if (existsSync(c)) return c;
  }
  return "voce";
}

// ── State ───────────────────────────────────────────────────────

const state: ChatState = {
  conversationHistory: [],
  currentIr: null,
  irHistory: [],
  sessionId: "", // assigned after resolveSession
  model: DEFAULT_MODEL,
  systemPrompt: "",
  tokenUsage: { input: 0, output: 0, cache_read: 0, cache_creation: 0 },
};

let client: Anthropic | null = null;
let activeAbort: AbortController | null = null;
let chatBusy = false;

// ── Chat ────────────────────────────────────────────────────────

async function chat(userMessage: string): Promise<void> {
  if (!client) throw new Error("API client not initialized");

  state.conversationHistory.push({
    role: "user",
    content: [{ type: "text", text: userMessage }],
  });
  logUserTurn(state.sessionId, userMessage);

  process.stdout.write(chalk.cyan("\nvoce "));

  const abort = new AbortController();
  activeAbort = abort;
  chatBusy = true;

  try {
    const result = await runToolLoop({
      client,
      model: state.model,
      system: state.systemPrompt,
      messages: state.conversationHistory as LoopMessage[],
      tools: TOOL_DEFINITIONS,
      executor: executeTool,
      signal: abort.signal,
      onText: (text) => process.stdout.write(text),
      onToolUse: (event) => {
        process.stdout.write(chalk.dim(`\n  ↳ ${event.name}\n  `));
      },
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
    });

    console.log(); // newline after final text

    if (result.capturedIr !== null) {
      pushIrHistory({ state }, result.capturedIr);
      logAssistantTurn(state.sessionId, result.finalText, result.capturedIr);
      console.log(
        chalk.dim(
          "\nIR captured. /compile, /validate, /save, /preview, /diff, /undo.",
        ),
      );
    } else {
      logAssistantTurn(state.sessionId, result.finalText);
    }

    if (!result.completed) {
      console.log(chalk.yellow("Tool-use loop hit its turn cap. Reply to continue."));
    }
  } catch (err) {
    if (err instanceof ToolLoopAborted) {
      console.log(chalk.yellow("\n(cancelled — back to prompt)"));
    } else {
      console.log(chalk.red("\nError:"), (err as Error).message);
    }
  } finally {
    activeAbort = null;
    chatBusy = false;
  }
}

// ── Main ────────────────────────────────────────────────────────

async function main() {
  console.log(chalk.bold.cyan("\n  Voce IR"));
  console.log(chalk.dim("  The code is gone. The experience remains.\n"));

  const args = parseArgs(process.argv.slice(2));

  const session = resolveSession({ resume: args.resume });
  state.sessionId = session.id;

  state.systemPrompt = buildSystemPrompt({
    brief: readBrief(),
    recentDecisions: listDecisions().slice(-20),
  });

  for (const entry of session.history) {
    if (entry.role === "user" || entry.role === "assistant") {
      state.conversationHistory.push({
        role: entry.role,
        content: [{ type: "text", text: entry.content }],
      });
      if (entry.role === "assistant" && entry.ir_snapshot !== undefined) {
        state.currentIr = entry.ir_snapshot;
      }
    }
  }

  if (session.resumed) {
    console.log(
      chalk.dim(`  Resumed session ${session.id.slice(0, 8)} — ${session.history.length} prior entries.`),
    );
    logSystemEvent(state.sessionId, "voce-chat resumed");
  } else {
    logSystemEvent(state.sessionId, "voce-chat started");
  }

  if (!API_KEY) {
    console.log(chalk.yellow("  Set ANTHROPIC_API_KEY to enable AI generation."));
    console.log(chalk.dim("  Without it, slash commands still work on the loaded IR.\n"));
    console.log(chalk.dim("  export ANTHROPIC_API_KEY=sk-ant-...\n"));
  } else {
    client = new Anthropic({ apiKey: API_KEY });
    console.log(chalk.dim(`  Model: ${state.model}`));
    console.log(chalk.dim("  Type what you want to build, or /help for commands.\n"));
  }

  const registry = buildRegistry();
  const ctx: CommandContext = {
    state,
    client,
    voceBin: findVoceBin(),
    log: (msg) => console.log(msg),
    exit: (code = 0) => process.exit(code),
  };

  if (args.initialPrompt && client) {
    await chat(args.initialPrompt);
  }

  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
    prompt: chalk.gray("you > "),
    historySize: 100,
  });
  const acc = createAccumulator();
  rl.prompt();

  rl.on("line", async (line: string) => {
    const fed = feedLine(acc, line);
    if (fed.kind === "continue") {
      // Show a continuation prompt so the user knows to keep typing.
      rl.setPrompt(chalk.gray("    > "));
      rl.prompt();
      return;
    }
    if (fed.kind === "empty") {
      rl.setPrompt(chalk.gray("you > "));
      rl.prompt();
      return;
    }

    rl.setPrompt(chalk.gray("you > "));
    const input = fed.text.trim();

    if (input.startsWith("/")) {
      try {
        const result = await registry.dispatch(input, ctx);
        if (result.handled) {
          if ("submitText" in result && result.submitText !== undefined) {
            // /explain (or any future command) can ask the chat to handle text.
            if (client) {
              await chat(result.submitText);
            } else {
              ctx.log(chalk.yellow("Set ANTHROPIC_API_KEY to use this command."));
            }
          }
          rl.prompt();
          return;
        }
        ctx.log(chalk.yellow(`Unknown command: ${input}. Try /help.`));
        rl.prompt();
        return;
      } catch (err) {
        ctx.log(chalk.red(`Command failed: ${(err as Error).message}`));
        rl.prompt();
        return;
      }
    }

    if (!client) {
      console.log(chalk.yellow("Set ANTHROPIC_API_KEY to chat. /help for slash commands."));
      rl.prompt();
      return;
    }

    await chat(input);
    rl.prompt();
  });

  // Ctrl+C handler:
  //   1) If a multi-line message is being typed, drop it and re-prompt.
  //   2) If a request is in flight, abort it and re-prompt.
  //   3) Otherwise, exit.
  rl.on("SIGINT", () => {
    if (clearPending(acc)) {
      console.log(chalk.yellow("\n(cleared pending input)"));
      rl.setPrompt(chalk.gray("you > "));
      rl.prompt();
      return;
    }
    if (chatBusy && activeAbort !== null) {
      activeAbort.abort();
      // The chat() catch block prints the cancellation notice + re-prompts.
      return;
    }
    console.log(chalk.dim("\nGoodbye."));
    process.exit(0);
  });

  rl.on("close", () => {
    console.log(chalk.dim("\nGoodbye."));
    process.exit(0);
  });
}

const isMain = import.meta.url === `file://${process.argv[1]}`;
if (isMain) {
  main().catch((err) => {
    console.error(chalk.red("Fatal:"), err.message);
    process.exit(1);
  });
}
