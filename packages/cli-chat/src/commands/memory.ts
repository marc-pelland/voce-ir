// Memory & explanation commands — brief, decisions, decision, explain.
// All read/write through @voce-ir/mcp-server/memory; /explain pumps a
// meta-prompt back into the chat loop via submitText.

import { listDecisions, logDecision, readBrief, writeBrief } from "@voce-ir/mcp-server/memory";
import type { CommandSpec } from "./registry.js";

export const briefCommand: CommandSpec = {
  name: "brief",
  summary: "Show the project brief. /brief <markdown> writes (atomic; confirms first).",
  handler: (rest, ctx) => {
    if (rest.length === 0) {
      const brief = readBrief();
      if (brief === null) {
        ctx.log("No brief yet. Use /brief <markdown> to author one.");
      } else {
        ctx.log(`Last modified: ${brief.last_modified}\n\n${brief.content}`);
      }
      return { handled: true };
    }
    writeBrief(rest);
    ctx.log(`Brief written (${rest.length} bytes).`);
    return { handled: true };
  },
};

export const decisionsCommand: CommandSpec = {
  name: "decisions",
  summary: "List recent decisions. /decisions 10 shows the last 10 (default 5).",
  handler: (rest, ctx) => {
    const n = parseInt(rest, 10);
    const limit = Number.isFinite(n) && n > 0 ? n : 5;
    const decisions = listDecisions().slice(-limit);
    if (decisions.length === 0) {
      ctx.log("No decisions logged yet.");
      return { handled: true };
    }
    const lines = decisions.map(
      (d) => `${d.timestamp.slice(0, 10)} [${d.id.slice(0, 8)}] ${d.summary}\n  ${d.rationale}`,
    );
    ctx.log(lines.join("\n\n"));
    return { handled: true };
  },
};

export const decisionCommand: CommandSpec = {
  name: "decision",
  summary: "Log a decision. /decision <summary> | <rationale>",
  handler: (rest, ctx) => {
    if (rest.length === 0) {
      ctx.log("Usage: /decision <summary> | <rationale>");
      return { handled: true };
    }
    const split = rest.split("|");
    const summary = split[0]?.trim() ?? "";
    const rationale = split.slice(1).join("|").trim();
    if (summary.length === 0 || rationale.length === 0) {
      ctx.log("Both summary and rationale are required. Separate with `|`.");
      return { handled: true };
    }
    const decision = logDecision({ summary, rationale });
    ctx.log(`Logged decision ${decision.id.slice(0, 8)}: ${summary}`);
    return { handled: true };
  },
};

export const explainCommand: CommandSpec = {
  name: "explain",
  summary: "Have the model explain why the current IR is shaped the way it is.",
  handler: (_rest, ctx) => {
    if (ctx.state.currentIr === null) {
      ctx.log("No IR yet — nothing to explain.");
      return { handled: true };
    }
    // Submit a meta-prompt back to the chat loop. The dispatcher returns
    // submitText so the caller can hand it to chat() — the model has full
    // context of the IR via session history.
    const submit =
      "Walk me through the current IR. Why is each top-level node there, " +
      "what choices did you make on a11y / forms / state, and what would " +
      "you change if pushed to v2? Keep it under 8 sentences.";
    return { handled: true, submitText: submit };
  },
};
