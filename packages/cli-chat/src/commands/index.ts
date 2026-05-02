// Wire-up for the slash command registry. Adding a new command means
// importing its spec and calling registry.register here.

import { CommandRegistry } from "./registry.js";
import { helpCommand } from "./help.js";
import { clearCommand, costCommand, exitCommand, modelCommand, undoCommand } from "./state.js";
import {
  compileCommand,
  diffCommand,
  irCommand,
  loadCommand,
  previewCommand,
  saveCommand,
  showCommand,
  validateCommand,
} from "./ir.js";
import { briefCommand, decisionCommand, decisionsCommand, explainCommand } from "./memory.js";

export type { ChatState, CommandContext, CommandHandler, CommandResult, CommandSpec } from "./registry.js";
export { CommandRegistry } from "./registry.js";

export function buildRegistry(): CommandRegistry {
  const registry = new CommandRegistry();
  // Order matters only for /help output.
  registry.register(helpCommand(registry));
  registry.register(showCommand);
  registry.register(saveCommand);
  registry.register(loadCommand);
  registry.register(irCommand);
  registry.register(compileCommand);
  registry.register(validateCommand);
  registry.register(previewCommand);
  registry.register(diffCommand);
  registry.register(undoCommand);
  registry.register(briefCommand);
  registry.register(decisionsCommand);
  registry.register(decisionCommand);
  registry.register(explainCommand);
  registry.register(modelCommand);
  registry.register(costCommand);
  registry.register(clearCommand);
  registry.register(exitCommand);
  return registry;
}
