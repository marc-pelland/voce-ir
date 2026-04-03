# Sprint 21 — AI Bridge Foundation

**Status:** Planned
**Goal:** Stand up the TypeScript AI bridge package, integrate the Claude API with structured output, and achieve the first end-to-end flow: natural language prompt to validated, compiled HTML. After this sprint, a single CLI command turns a text description into a working web page.
**Depends on:** Sprint 20 (Phase 2 complete, v0.2.0 tagged)

---

## Deliverables

1. `packages/ai-bridge/` TypeScript project with build toolchain (tsconfig, vitest, tsup)
2. Claude API client wrapper with structured JSON output mode
3. IR JSON schema context module — feeds the LLM the full Voce IR shape as system prompt context
4. `IrGenerator` class: prompt string in, validated IR JSON out
5. Shell integration: calls `voce validate` and `voce compile` on generated output
6. `voce generate <prompt>` CLI command (thin wrapper invoking the bridge)
7. First successful generation: "a landing page with a hero and CTA" produces valid HTML

---

## Tasks

### 1. Project Scaffolding (`packages/ai-bridge/`)

Initialize TypeScript package: `package.json`, `tsconfig.json`, `tsup.config.ts` for bundling, `vitest.config.ts` for tests. Add `@anthropic-ai/sdk` dependency. Export a public API from `src/index.ts`. Wire into workspace root scripts.

### 2. Claude API Client (`src/api/claude-client.ts`)

Wrap the Anthropic SDK with Voce-specific defaults: model selection, max tokens, temperature, structured output mode (JSON). Handle retries with exponential backoff. Expose `generate(systemPrompt, userPrompt) -> structured JSON`. Read API key from `ANTHROPIC_API_KEY` env var or `voce.config.toml`.

### 3. Schema Context Builder (`src/context/schema-context.ts`)

Build the system prompt context that teaches Claude the IR format. Include: node type catalog (all 27 types with required/optional fields), valid enum values, structural rules (ViewRoot must be root, SemanticNode required), and a minimal complete example IR. Keep under 8K tokens — compress ruthlessly.

### 4. IR Generator (`src/generator/ir-generator.ts`)

Orchestrate: build system prompt from schema context, send user prompt to Claude API, parse JSON response, write to temp file, shell out to `voce validate`, return result with diagnostics. If validation fails, include error list in the response (Repair Agent comes in S22).

### 5. Rust CLI Integration (`packages/validator/src/commands/generate.rs`)

Add `voce generate "<prompt>"` command. Invokes `npx voce-ai-bridge generate` (or the built JS entry point) with the prompt, streams output, then runs validate + compile on the result. Print the output HTML path on success.

### 6. End-to-End Test (`tests/ai-bridge/e2e.test.ts`)

Integration test using a real Claude API call (skipped in CI without API key). Prompt: "a simple landing page with a headline, subtitle, and call-to-action button." Assert: output is valid JSON, passes `voce validate`, compiles to HTML containing expected elements.

### 7. Prompt Engineering Baseline (`src/context/base-prompt.ts`)

Craft the initial generation prompt template. Instruct the model to: output only valid Voce IR JSON, include ViewRoot with metadata, include SemanticNode for every interactive element, use correct enum values. Document prompt version for A/B iteration.

---

## Files to Create

- `packages/ai-bridge/package.json`
- `packages/ai-bridge/tsconfig.json`
- `packages/ai-bridge/tsup.config.ts`
- `packages/ai-bridge/vitest.config.ts`
- `packages/ai-bridge/src/index.ts`
- `packages/ai-bridge/src/api/claude-client.ts`
- `packages/ai-bridge/src/context/schema-context.ts`
- `packages/ai-bridge/src/context/base-prompt.ts`
- `packages/ai-bridge/src/generator/ir-generator.ts`
- `packages/ai-bridge/src/cli.ts`
- `packages/validator/src/commands/generate.rs`
- `tests/ai-bridge/e2e.test.ts`

---

## Acceptance Criteria

- [ ] `npm run build` in `packages/ai-bridge/` succeeds with zero errors
- [ ] `npm test` passes all unit tests (mocked Claude responses)
- [ ] Schema context fits under 8K tokens measured via tiktoken
- [ ] `voce generate "a hero section with headline and button"` produces valid IR JSON
- [ ] Generated IR passes `voce validate` without errors
- [ ] Generated IR compiles via `voce compile` to valid HTML
- [ ] HTML output contains semantic elements (heading, button) matching the prompt
- [ ] API key missing produces a clear error message, not a crash
- [ ] Generated IR includes SemanticNode for all interactive elements
- [ ] End-to-end latency under 15 seconds for a simple prompt
