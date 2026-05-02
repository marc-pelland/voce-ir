// IR-handling commands — show / save / load / ir / compile / validate /
// preview / diff. compile + validate + preview shell out to the voce CLI.

import { execFileSync, execSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir, platform } from "node:os";
import { join } from "node:path";
import type { CommandSpec } from "./registry.js";

export const showCommand: CommandSpec = {
  name: "show",
  summary: "Print the current IR JSON.",
  handler: (_rest, ctx) => {
    if (ctx.state.currentIr === null) {
      ctx.log("No IR yet. Describe what you want to build first.");
    } else {
      ctx.log(ctx.state.currentIr);
    }
    return { handled: true };
  },
};

export const saveCommand: CommandSpec = {
  name: "save",
  summary: "Save the current IR to a file. /save my.voce.json (default: generated.voce.json).",
  handler: (rest, ctx) => {
    if (ctx.state.currentIr === null) {
      ctx.log("No IR yet.");
      return { handled: true };
    }
    const filename = rest.length > 0 ? rest : "generated.voce.json";
    writeFileSync(filename, ctx.state.currentIr);
    ctx.log(`Saved to ${filename}.`);
    return { handled: true };
  },
};

export const loadCommand: CommandSpec = {
  name: "load",
  summary: "Load IR from a file path. /load examples/landing-page.voce.json.",
  handler: (rest, ctx) => {
    if (rest.length === 0) {
      ctx.log("Usage: /load <file>");
      return { handled: true };
    }
    if (!existsSync(rest)) {
      ctx.log(`Not found: ${rest}`);
      return { handled: true };
    }
    try {
      const content = readFileSync(rest, "utf-8");
      JSON.parse(content); // sanity check
      pushIrHistory(ctx, content);
      ctx.log(`Loaded ${rest} (${content.length} bytes).`);
    } catch (err) {
      ctx.log(`Failed to load ${rest}: ${(err as Error).message}`);
    }
    return { handled: true };
  },
};

export const irCommand: CommandSpec = {
  name: "ir",
  summary: "Paste IR JSON inline as a multi-line block. Empty line submits.",
  handler: (rest, ctx) => {
    if (rest.length === 0) {
      ctx.log("Usage: /ir <pasted JSON> (or follow with a multi-line input)");
      return { handled: true };
    }
    try {
      JSON.parse(rest);
    } catch (err) {
      ctx.log(`Pasted text isn't valid JSON: ${(err as Error).message}`);
      return { handled: true };
    }
    pushIrHistory(ctx, rest);
    ctx.log(`IR loaded (${rest.length} bytes).`);
    return { handled: true };
  },
};

export const compileCommand: CommandSpec = {
  name: "compile",
  summary: "Compile current IR to HTML. Tail of output is shown; full file path printed.",
  handler: (_rest, ctx) => {
    if (ctx.state.currentIr === null) {
      ctx.log("No IR yet.");
      return { handled: true };
    }
    const tmpFile = join(tmpdir(), `voce-chat-${Date.now()}.voce.json`);
    const outFile = join(tmpdir(), `voce-chat-${Date.now()}.html`);
    try {
      writeFileSync(tmpFile, ctx.state.currentIr);
      execFileSync(ctx.voceBin, ["compile", tmpFile, "-o", outFile, "--skip-fonts", "--no-cache"], {
        encoding: "utf-8",
        stdio: ["pipe", "pipe", "pipe"],
      });
      const html = readFileSync(outFile, "utf-8");
      const lines = html.split("\n");
      ctx.log(`Compiled (${html.length} bytes):`);
      ctx.log(lines.slice(0, 20).join("\n"));
      if (lines.length > 20) ctx.log(`... (${lines.length - 20} more lines)`);
      ctx.log(`Full output: ${outFile}`);
    } catch (err) {
      const e = err as { stderr?: Buffer | string };
      const stderr = typeof e.stderr === "string" ? e.stderr : e.stderr?.toString() ?? "Unknown";
      ctx.log(`Compilation failed: ${stderr}`);
    }
    return { handled: true };
  },
};

export const validateCommand: CommandSpec = {
  name: "validate",
  summary: "Run validator on current IR. Shows per-pass diagnostics.",
  handler: (_rest, ctx) => {
    if (ctx.state.currentIr === null) {
      ctx.log("No IR yet.");
      return { handled: true };
    }
    const tmpFile = join(tmpdir(), `voce-chat-${Date.now()}.voce.json`);
    try {
      writeFileSync(tmpFile, ctx.state.currentIr);
      const output = execFileSync(ctx.voceBin, ["validate", "--format", "json", tmpFile], {
        encoding: "utf-8",
        stdio: ["pipe", "pipe", "pipe"],
      });
      ctx.log(`Validation passed: ${output.trim()}`);
    } catch (err) {
      const e = err as { stdout?: Buffer | string; stderr?: Buffer | string };
      const out =
        (typeof e.stdout === "string" ? e.stdout : e.stdout?.toString() ?? "") ||
        (typeof e.stderr === "string" ? e.stderr : e.stderr?.toString() ?? "");
      ctx.log("Validation results:");
      ctx.log(out);
    }
    return { handled: true };
  },
};

export const previewCommand: CommandSpec = {
  name: "preview",
  summary: "Compile and open in your default browser.",
  handler: (_rest, ctx) => {
    if (ctx.state.currentIr === null) {
      ctx.log("No IR yet.");
      return { handled: true };
    }
    const tmpFile = join(tmpdir(), `voce-chat-${Date.now()}.voce.json`);
    const outFile = join(tmpdir(), `voce-chat-${Date.now()}.html`);
    try {
      writeFileSync(tmpFile, ctx.state.currentIr);
      execFileSync(ctx.voceBin, ["compile", tmpFile, "-o", outFile, "--skip-fonts", "--no-cache"], {
        encoding: "utf-8",
        stdio: ["pipe", "pipe", "pipe"],
      });
      const opener = openerForPlatform();
      execSync(`${opener} "${outFile}"`, { stdio: "ignore" });
      ctx.log(`Opened ${outFile}`);
    } catch (err) {
      ctx.log(`Preview failed: ${(err as Error).message}`);
    }
    return { handled: true };
  },
};

export const diffCommand: CommandSpec = {
  name: "diff",
  summary: "Show what changed between the last two IR snapshots in this session.",
  handler: (_rest, ctx) => {
    const previous = ctx.state.irHistory[0];
    if (previous === undefined || ctx.state.currentIr === null) {
      ctx.log("Need at least two IR snapshots to diff. Generate or refine to populate history.");
      return { handled: true };
    }
    const summary = summarizeDiff(previous, ctx.state.currentIr);
    ctx.log(summary);
    return { handled: true };
  },
};

// ─── Helpers ───────────────────────────────────────────────────────

/** Push the current IR onto the history stack and replace it with `next`. */
export function pushIrHistory(
  ctx: { state: { currentIr: string | null; irHistory: string[] } },
  next: string,
): void {
  if (ctx.state.currentIr !== null) {
    ctx.state.irHistory.unshift(ctx.state.currentIr);
    // Cap at a reasonable depth so memory doesn't grow unbounded over a long session.
    if (ctx.state.irHistory.length > 32) ctx.state.irHistory.length = 32;
  }
  ctx.state.currentIr = next;
}

function openerForPlatform(): string {
  const p = platform();
  if (p === "darwin") return "open";
  if (p === "win32") return "start";
  return "xdg-open";
}

/**
 * Lightweight diff summary — bytes added/removed + first 5 differing lines.
 * For richer diffing, /save both versions and run `diff` externally.
 */
export function summarizeDiff(before: string, after: string): string {
  const beforeLines = before.split("\n");
  const afterLines = after.split("\n");
  const lengths = `before: ${before.length}B (${beforeLines.length} lines)\nafter:  ${after.length}B (${afterLines.length} lines)`;

  // Walk both line-by-line, collect first 5 differing lines for context.
  const max = Math.max(beforeLines.length, afterLines.length);
  const diffs: string[] = [];
  for (let i = 0; i < max && diffs.length < 5; i++) {
    const a = beforeLines[i] ?? "";
    const b = afterLines[i] ?? "";
    if (a !== b) diffs.push(`@${i + 1}\n  -- ${a}\n  ++ ${b}`);
  }
  if (diffs.length === 0) return `${lengths}\n(no line-level differences)`;
  return `${lengths}\n\nFirst diffs:\n${diffs.join("\n")}`;
}
