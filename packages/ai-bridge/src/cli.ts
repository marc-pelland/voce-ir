#!/usr/bin/env node
/**
 * Voce IR AI Bridge CLI — generates IR from natural language.
 *
 * Usage:
 *   voce-ai-bridge generate "a hero section with headline and CTA"
 *   voce-ai-bridge generate --no-compile "a contact form"
 */

import { IrGenerator } from "./generator/ir-generator.js";

async function main() {
  const args = process.argv.slice(2);
  const command = args[0];

  if (command !== "generate" || args.length < 2) {
    console.error("Usage: voce-ai-bridge generate <prompt> [--no-compile]");
    process.exit(2);
  }

  const noCompile = args.includes("--no-compile");
  const prompt = args.filter((a) => a !== "generate" && a !== "--no-compile").join(" ");

  if (!prompt) {
    console.error("Error: prompt is required");
    process.exit(2);
  }

  console.error(`Generating IR from: "${prompt}"`);

  try {
    const generator = new IrGenerator();
    const result = await generator.generate(prompt, { compile: !noCompile });

    if (result.success) {
      console.error(`\u2713 Generated valid IR`);
      if (result.htmlPath) {
        console.error(
          `\u2713 Compiled to ${result.htmlPath} (${result.htmlSize} bytes)`
        );
      }
      // Output the IR JSON to stdout
      if (result.irJson) {
        console.log(result.irJson);
      }
    } else {
      console.error(`\u2717 Generation produced invalid IR:`);
      for (const err of result.validationErrors) {
        console.error(`  - ${err}`);
      }
      process.exit(1);
    }
  } catch (error) {
    console.error(`Error: ${(error as Error).message}`);
    process.exit(2);
  }
}

main();
