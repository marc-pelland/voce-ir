---
name: AI-Agnostic Platform + Voice Interface
description: Voce IR must be a platform/protocol, not locked to one AI. Users connect their preferred AI tool. Voice conversation is a first-class input method.
type: feedback
---

The AI layer is pluggable — Voce IR is the platform (schema, validator, compiler, CLI), not the AI. Users should be able to use Claude Code, Cursor, GPT, local models, or any custom agent. The MCP server for Claude Code is the highest-priority integration.

Voice is a first-class input method alongside text. Push-to-talk in CLI, seamless text/voice switching, transcript always visible. Named from "sotto voce" — voice-based building is the purest expression of the thesis.

**Why:** Marc wants to empower users to build with tooling they're comfortable with, not force them into a specific AI. This also creates strategic defense — any AI that gets better at UI generation can target the Voce IR schema.

**How to apply:**
- The schema JSON is the contract. Any tool that produces conforming JSON can use Voce IR.
- MCP server exposes: validate, compile, preview, inspect, schema, examples, style_packs
- SDK (TS/Python) wraps the CLI for programmatic use
- voce.config.toml has [ai] and [voice] sections for provider choice
- Phase 1 gets raw CLI integration for free (JSON files in, compiled out)
- Phase 2-3: MCP server, SDK, voice interface
