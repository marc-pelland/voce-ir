#!/usr/bin/env node
/**
 * Voce IR Conversational CLI
 *
 * An interactive prompt where you describe what you want to build
 * and Voce IR generates, validates, and compiles it for you.
 *
 * Usage:
 *   voce-chat                    # Start interactive session
 *   voce-chat "build a nav bar"  # Start with an initial prompt
 *
 * Commands:
 *   /help          Show available commands
 *   /compile       Compile the current IR to HTML
 *   /validate      Validate the current IR
 *   /save <file>   Save the current IR to a file
 *   /show          Show the current IR
 *   /clear         Clear conversation and start over
 *   /exit          Exit the CLI
 */

import Anthropic from "@anthropic-ai/sdk";
import { createInterface } from "node:readline";
import { execSync } from "node:child_process";
import { writeFileSync, readFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import chalk from "chalk";
import { listDecisions, readBrief } from "@voce-ir/mcp-server/memory";
import { parseArgs } from "./cli-args.js";
import { buildSystemPrompt } from "./prompt.js";
import {
  logAssistantTurn,
  logSystemEvent,
  logUserTurn,
  resolveSession,
} from "./session-manager.js";

// ── Config ──────────────────────────────────────────────────────

const MODEL = process.env.VOCE_MODEL || "claude-sonnet-4-20250514";
const API_KEY = process.env.ANTHROPIC_API_KEY;

// Find voce binary
function findVoce(): string {
  const candidates = [
    join(process.cwd(), "target/release/voce"),
    join(process.cwd(), "target/debug/voce"),
  ];
  for (const c of candidates) {
    if (existsSync(c)) return c;
  }
  return "voce";
}
const VOCE_BIN = findVoce();

// ── Schema Context ──────────────────────────────────────────────

function loadSchemaContext(): string {
  const schemaDir = join(process.cwd(), "docs/site/src/schema");
  const files = ["overview.md", "layout.md", "state.md", "forms.md"];
  let context = "";
  for (const f of files) {
    const path = join(schemaDir, f);
    if (existsSync(path)) {
      context += readFileSync(path, "utf-8") + "\n\n";
    }
  }
  return context || "Schema docs not found.";
}

// ── State ───────────────────────────────────────────────────────

interface Message {
  role: "user" | "assistant";
  content: string;
}

let conversationHistory: Message[] = [];
let currentIr: string | null = null;
let client: Anthropic | null = null;
let sessionId: string = ""; // assigned in main() after resolveSession
let systemPromptCache: string = ""; // built once per process; brief/decisions are stable per-session

// ── Commands ────────────────────────────────────────────────────

function handleCommand(input: string): boolean {
  const parts = input.trim().split(/\s+/);
  const cmd = parts[0].toLowerCase();

  switch (cmd) {
    case "/help":
      console.log(chalk.cyan("\nCommands:"));
      console.log("  /help          Show this help");
      console.log("  /compile       Compile the current IR to HTML");
      console.log("  /validate      Validate the current IR");
      console.log("  /save <file>   Save the current IR to a file");
      console.log("  /show          Show the current IR");
      console.log("  /clear         Clear conversation and start over");
      console.log("  /exit          Exit\n");
      return true;

    case "/compile":
      if (!currentIr) {
        console.log(chalk.yellow("No IR generated yet. Describe what you want to build first."));
        return true;
      }
      compileIr();
      return true;

    case "/validate":
      if (!currentIr) {
        console.log(chalk.yellow("No IR generated yet. Describe what you want to build first."));
        return true;
      }
      validateIr();
      return true;

    case "/save": {
      if (!currentIr) {
        console.log(chalk.yellow("No IR generated yet."));
        return true;
      }
      const filename = parts[1] || "generated.voce.json";
      writeFileSync(filename, currentIr);
      console.log(chalk.green(`Saved to ${filename}`));
      return true;
    }

    case "/show":
      if (!currentIr) {
        console.log(chalk.yellow("No IR generated yet."));
      } else {
        console.log(chalk.dim(currentIr));
      }
      return true;

    case "/clear":
      conversationHistory = [];
      currentIr = null;
      console.log(chalk.cyan("Conversation cleared."));
      return true;

    case "/exit":
    case "/quit":
      process.exit(0);

    default:
      return false;
  }
}

function compileIr() {
  if (!currentIr) return;
  const tmpFile = join(tmpdir(), `voce-chat-${Date.now()}.voce.json`);
  const outFile = join(tmpdir(), `voce-chat-${Date.now()}.html`);

  try {
    writeFileSync(tmpFile, currentIr);
    execSync(`"${VOCE_BIN}" compile "${tmpFile}" -o "${outFile}" --skip-fonts --no-cache`, {
      encoding: "utf-8",
      stdio: ["pipe", "pipe", "pipe"],
    });
    const html = readFileSync(outFile, "utf-8");
    console.log(chalk.green(`\nCompiled (${html.length} bytes):`));
    // Show first 20 lines
    const lines = html.split("\n");
    console.log(chalk.dim(lines.slice(0, 20).join("\n")));
    if (lines.length > 20) {
      console.log(chalk.dim(`... (${lines.length - 20} more lines)`));
    }
    console.log(chalk.green(`\nFull output saved to: ${outFile}`));
  } catch (error: unknown) {
    const err = error as { stderr?: string };
    console.log(chalk.red("Compilation failed:"), err.stderr || "Unknown error");
  }
}

function validateIr() {
  if (!currentIr) return;
  const tmpFile = join(tmpdir(), `voce-chat-${Date.now()}.voce.json`);

  try {
    writeFileSync(tmpFile, currentIr);
    const output = execSync(`"${VOCE_BIN}" validate --format json "${tmpFile}"`, {
      encoding: "utf-8",
      stdio: ["pipe", "pipe", "pipe"],
    });
    console.log(chalk.green("\nValidation passed:"), output.trim());
  } catch (error: unknown) {
    const err = error as { stdout?: string; stderr?: string };
    const output = err.stdout || err.stderr || "";
    console.log(chalk.yellow("\nValidation results:"));
    console.log(output);
  } finally {
    try {
      execSync(`rm "${tmpFile}"`, { stdio: "ignore" });
    } catch {
      /* ignore */
    }
  }
}

// ── Extract IR from AI response ─────────────────────────────────

function extractIrFromResponse(response: string): string | null {
  // Look for JSON code fences
  const jsonMatch = response.match(/```json\s*\n([\s\S]*?)\n```/);
  if (jsonMatch) {
    try {
      JSON.parse(jsonMatch[1]);
      return jsonMatch[1];
    } catch {
      return null;
    }
  }
  return null;
}

// ── Chat ────────────────────────────────────────────────────────

async function chat(userMessage: string): Promise<string> {
  if (!client) throw new Error("API client not initialized");

  conversationHistory.push({ role: "user", content: userMessage });
  logUserTurn(sessionId, userMessage);

  process.stdout.write(chalk.cyan("\nvoce "));

  const stream = client.messages.stream({
    model: MODEL,
    max_tokens: 4096,
    system: systemPromptCache,
    messages: conversationHistory,
  });

  let fullResponse = "";

  for await (const event of stream) {
    if (
      event.type === "content_block_delta" &&
      event.delta.type === "text_delta"
    ) {
      process.stdout.write(event.delta.text);
      fullResponse += event.delta.text;
    }
  }

  console.log(); // newline after streaming

  conversationHistory.push({ role: "assistant", content: fullResponse });

  // Extract IR if present — passed as ir_snapshot so voce_session_resume
  // surfaces it as current_ir for any subsequent invocation or MCP client.
  const ir = extractIrFromResponse(fullResponse);
  if (ir) {
    currentIr = ir;
    logAssistantTurn(sessionId, fullResponse, ir);
    console.log(
      chalk.dim(
        "\nIR captured. Type /compile to see HTML, /validate to check, /save to save."
      )
    );
  } else {
    logAssistantTurn(sessionId, fullResponse);
  }

  return fullResponse;
}

// ── Main ────────────────────────────────────────────────────────

async function main() {
  console.log(chalk.bold.cyan("\n  Voce IR"));
  console.log(chalk.dim("  The code is gone. The experience remains.\n"));

  // Parse argv first so the resume flag is honored before any IO.
  const args = parseArgs(process.argv.slice(2));

  // Resolve / open a session and hydrate the system prompt from the .voce/
  // store. Both happen even without an API key so /compile, /validate still
  // get a session log.
  const session = resolveSession({ resume: args.resume });
  sessionId = session.id;

  systemPromptCache = buildSystemPrompt({
    brief: readBrief(),
    recentDecisions: listDecisions().slice(-20),
  });

  // Replay prior text turns into conversationHistory so the model sees the
  // same context as before. System / tool entries are skipped — only the
  // user/assistant ledger is part of the chat completion contract.
  for (const entry of session.history) {
    if (entry.role === "user" || entry.role === "assistant") {
      conversationHistory.push({ role: entry.role, content: entry.content });
      if (entry.role === "assistant" && entry.ir_snapshot !== undefined) {
        currentIr = entry.ir_snapshot;
      }
    }
  }

  if (session.resumed) {
    console.log(
      chalk.dim(`  Resumed session ${session.id.slice(0, 8)} — ${session.history.length} prior entries.`),
    );
    logSystemEvent(sessionId, "voce-chat resumed");
  } else {
    logSystemEvent(sessionId, "voce-chat started");
  }

  if (!API_KEY) {
    console.log(chalk.yellow("  Set ANTHROPIC_API_KEY to enable AI generation."));
    console.log(chalk.dim("  Without it, you can still use /compile, /validate on .voce.json files.\n"));
    console.log(chalk.dim("  export ANTHROPIC_API_KEY=sk-ant-...\n"));
  } else {
    client = new Anthropic({ apiKey: API_KEY });
    console.log(chalk.dim(`  Model: ${MODEL}`));
    console.log(chalk.dim("  Type what you want to build, or /help for commands.\n"));
  }

  // Handle initial prompt from args
  if (args.initialPrompt && client) {
    await chat(args.initialPrompt);
  }

  // Interactive REPL
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
    prompt: chalk.gray("you > "),
    historySize: 100,
  });

  rl.prompt();

  rl.on("line", async (line: string) => {
    const input = line.trim();
    if (!input) {
      rl.prompt();
      return;
    }

    // Handle slash commands
    if (input.startsWith("/")) {
      const handled = handleCommand(input);
      if (handled) {
        rl.prompt();
        return;
      }
    }

    // If no API key, only commands work
    if (!client) {
      console.log(
        chalk.yellow("Set ANTHROPIC_API_KEY to chat. Use /help for available commands.")
      );
      rl.prompt();
      return;
    }

    try {
      await chat(input);
    } catch (error: unknown) {
      const err = error as Error;
      console.log(chalk.red("Error:"), err.message);
    }

    rl.prompt();
  });

  rl.on("close", () => {
    console.log(chalk.dim("\nGoodbye."));
    process.exit(0);
  });
}

main().catch((err) => {
  console.error(chalk.red("Fatal:"), err.message);
  process.exit(1);
});
