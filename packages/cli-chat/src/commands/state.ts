// Session-state commands — clear, exit, model, cost, undo. None of these
// shell out to voce or call the model; pure state mutations.

import type { CommandSpec } from "./registry.js";

export const clearCommand: CommandSpec = {
  name: "clear",
  summary: "Clear the in-memory conversation; .voce/sessions/<id>.jsonl remains.",
  handler: (_rest, ctx) => {
    ctx.state.conversationHistory = [];
    ctx.state.currentIr = null;
    ctx.state.irHistory = [];
    ctx.log("Conversation cleared.");
    return { handled: true };
  },
};

export const exitCommand: CommandSpec = {
  name: "exit",
  aliases: ["quit"],
  summary: "Exit voce-chat. Session log is durable.",
  handler: (_rest, ctx) => {
    ctx.exit(0);
    return { handled: true };
  },
};

export const modelCommand: CommandSpec = {
  name: "model",
  summary: "Switch model mid-session, e.g. /model claude-opus-4-7 or /model claude-sonnet-4-6.",
  handler: (rest, ctx) => {
    if (rest.length === 0) {
      ctx.log(`Current model: ${ctx.state.model}`);
      return { handled: true };
    }
    const previous = ctx.state.model;
    ctx.state.model = rest;
    ctx.log(`Model: ${previous} → ${rest}`);
    return { handled: true };
  },
};

export const costCommand: CommandSpec = {
  name: "cost",
  summary: "Show cumulative token usage. Cost is rough — verify against your console.",
  handler: (_rest, ctx) => {
    const u = ctx.state.tokenUsage;
    const lines = [
      `Tokens used in this session:`,
      `  input:               ${u.input.toLocaleString()}`,
      `  output:              ${u.output.toLocaleString()}`,
      `  cache_read:          ${u.cache_read.toLocaleString()}`,
      `  cache_creation:      ${u.cache_creation.toLocaleString()}`,
    ];
    ctx.log(lines.join("\n"));
    return { handled: true };
  },
};

export const undoCommand: CommandSpec = {
  name: "undo",
  summary: "Pop the last IR snapshot. /undo /undo /undo to walk back further.",
  handler: (_rest, ctx) => {
    if (ctx.state.irHistory.length === 0) {
      ctx.log("Nothing to undo — no prior IR snapshot in this session.");
      return { handled: true };
    }
    const previous = ctx.state.irHistory.shift()!;
    ctx.state.currentIr = previous;
    ctx.log(`Reverted to prior IR (${previous.length} bytes).`);
    return { handled: true };
  },
};
