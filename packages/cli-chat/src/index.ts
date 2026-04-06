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

const SYSTEM_PROMPT = `You are Voce IR, an AI-native UI generation system. You help users build user interfaces through conversation.

When the user describes what they want, you:
1. Ask clarifying questions (one at a time) to understand their needs
2. Generate Voce IR JSON that implements their request
3. Explain what you built and suggest improvements

IMPORTANT RULES:
- Always output valid Voce IR JSON when generating UI
- Wrap generated IR in a \`\`\`json code fence
- Include schema_version_major: 1, schema_version_minor: 0
- Every node needs a unique node_id
- Use SemanticNode roles for nav, main, footer
- Add href fields to TextNode/Surface for links
- Include metadata (title, description) in the root

When the user says something vague, ask ONE clarifying question. Don't generate until you understand what they want.

After generating IR, tell the user they can type /compile to see the HTML output, /validate to check for issues, or /save to save the IR.

Available node types: Container (layout), Surface (cards/backgrounds), TextNode (text with optional href), MediaNode (images/video), FormNode (forms), SemanticNode (a11y roles), StateMachine (state), GestureHandler (interactions).
`;

// ── State ───────────────────────────────────────────────────────

interface Message {
  role: "user" | "assistant";
  content: string;
}

let conversationHistory: Message[] = [];
let currentIr: string | null = null;
let client: Anthropic | null = null;

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

  process.stdout.write(chalk.cyan("\nvoce "));

  const stream = client.messages.stream({
    model: MODEL,
    max_tokens: 4096,
    system: SYSTEM_PROMPT,
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

  // Extract IR if present
  const ir = extractIrFromResponse(fullResponse);
  if (ir) {
    currentIr = ir;
    console.log(
      chalk.dim(
        "\nIR captured. Type /compile to see HTML, /validate to check, /save to save."
      )
    );
  }

  return fullResponse;
}

// ── Main ────────────────────────────────────────────────────────

async function main() {
  console.log(chalk.bold.cyan("\n  Voce IR"));
  console.log(chalk.dim("  The code is gone. The experience remains.\n"));

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
  const initialPrompt = process.argv.slice(2).join(" ");
  if (initialPrompt && client) {
    await chat(initialPrompt);
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
