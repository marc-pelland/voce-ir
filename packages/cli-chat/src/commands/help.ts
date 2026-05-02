// /help — generated from the registry so new commands surface automatically.

import type { CommandRegistry, CommandSpec } from "./registry.js";

export function helpCommand(registry: CommandRegistry): CommandSpec {
  return {
    name: "help",
    aliases: ["?"],
    summary: "List available commands.",
    handler: (_rest, ctx) => {
      const lines = ["Commands:"];
      for (const spec of registry.list()) {
        const aliases = spec.aliases && spec.aliases.length > 0
          ? ` (${spec.aliases.map((a) => `/${a}`).join(", ")})`
          : "";
        lines.push(`  /${spec.name}${aliases} — ${spec.summary}`);
      }
      lines.push("");
      lines.push("Multi-line input: end a line with \\ to continue. Ctrl+C cancels in-flight responses.");
      ctx.log(lines.join("\n"));
      return { handled: true };
    },
  };
}
