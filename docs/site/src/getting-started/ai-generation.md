# AI Generation

Voce IR is designed from the ground up for AI authorship. Rather than having an AI write framework code, the AI generates typed IR directly and a compiler handles the output. There are two ways to generate IR with AI: the **TypeScript SDK** and the **MCP server**.

## Approaches

### TypeScript SDK

The `@voce-ir/ai-bridge` package provides a high-level API for generating IR from natural language prompts. It handles schema context, validation, and iterative refinement automatically.

### MCP Server

The Voce MCP (Model Context Protocol) server exposes IR generation as a tool that any MCP-compatible client can call -- including Claude Desktop, Claude Code, and other AI assistants. This lets you generate and compile IR through conversation without writing any integration code.

## SDK quickstart

### Install

```bash
npm install @voce-ir/ai-bridge
```

### Set your API key

The SDK uses the Anthropic API to generate IR. Export your key:

```bash
export ANTHROPIC_API_KEY=sk-ant-...
```

### Generate IR from a prompt

```typescript
import { VoceGenerator } from "@voce-ir/ai-bridge";

const generator = new VoceGenerator({
  apiKey: process.env.ANTHROPIC_API_KEY,
});

// Describe what you want in natural language
const result = await generator.generate(
  "A landing page with a bold headline that says 'Ship faster with Voce', " +
  "a subtitle explaining the product, and a blue call-to-action button."
);

// result.ir contains the validated .voce.json document
console.log(JSON.stringify(result.ir, null, 2));

// result.warnings contains any validation warnings
if (result.warnings.length > 0) {
  console.warn("Warnings:", result.warnings);
}
```

### Generate and compile in one step

```typescript
import { VoceGenerator } from "@voce-ir/ai-bridge";
import { writeFileSync } from "fs";

const generator = new VoceGenerator({
  apiKey: process.env.ANTHROPIC_API_KEY,
});

const result = await generator.generateAndCompile(
  "A contact form with name, email, and message fields. " +
  "Include proper validation and a submit button.",
  { target: "dom" }
);

// result.html contains the compiled single-file HTML
writeFileSync("dist/contact.html", result.html);
```

### Iterative refinement

The SDK supports multi-turn refinement. The AI asks clarifying questions when the prompt is ambiguous, and you can provide follow-up instructions:

```typescript
const session = generator.createSession();

// Initial generation
let result = await session.generate(
  "A pricing page with three tiers"
);

// Refine based on the result
result = await session.refine(
  "Make the middle tier visually prominent and add a 'Most Popular' badge"
);

// Further refinement
result = await session.refine(
  "Add a toggle to switch between monthly and annual pricing"
);

// The final IR incorporates all refinements
writeFileSync("pricing.voce.json", JSON.stringify(result.ir, null, 2));
```

## MCP server

### Setup with Claude Desktop

Add the Voce MCP server to your Claude Desktop configuration (`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "voce-ir": {
      "command": "npx",
      "args": ["@voce-ir/mcp-server"],
      "env": {
        "ANTHROPIC_API_KEY": "sk-ant-..."
      }
    }
  }
}
```

Once configured, you can ask Claude to generate Voce IR through normal conversation. The MCP server exposes tools for generating, validating, compiling, and previewing IR.

### Available MCP tools

| Tool | Description |
|------|-------------|
| `voce_generate` | Generate IR from a natural language description |
| `voce_validate` | Validate an IR document |
| `voce_compile` | Compile IR to a target format |
| `voce_preview` | Compile and return a preview URL |
| `voce_inspect` | Return a summary of an IR document |

## The playground

Try Voce IR without installing anything at [voce-ir.xyz](https://voce-ir.xyz). The playground provides:

- A prompt input for natural language descriptions
- Live IR preview with syntax highlighting
- One-click compilation to HTML
- A visual preview of the compiled output

Note that the playground requires an API key for generation features. Compilation and validation work without one.

## How generation works

Under the hood, the AI generation pipeline works as follows:

1. **Schema context.** The SDK provides the AI model with the full Voce IR schema -- all node types, their fields, valid values, and constraints.
2. **Generation.** The model generates a `.voce.json` document from your natural language prompt.
3. **Validation.** The generated IR is run through the full validator (structural checks, accessibility, security, SEO, forms, i18n).
4. **Auto-repair.** If validation fails, the SDK sends the errors back to the model for automatic correction. This loop runs up to 3 times.
5. **Output.** The validated IR is returned, ready for compilation.

This pipeline ensures that AI-generated IR is always valid and meets all quality gates -- accessibility, security, and SEO checks are compile errors, not optional linting.

## Next steps

- Explore the [Schema Reference](../schema/overview.md) to understand all available node types
- Read about [Validation Passes](../architecture/validation.md) to understand what the validator checks
- See the [CLI Reference](../cli/validate.md) for all command options
