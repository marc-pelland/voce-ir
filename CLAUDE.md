# Voce IR — Claude Code Instructions

## Project Overview

Voce IR is an open-source AI-native UI intermediate representation — "SPIR-V for UI." AI generates typed binary IR from natural language, a compiler emits optimized output (DOM, WebGPU, WASM, Native). No human-readable code in the pipeline. This is not AI writing framework code faster — it is a system designed from scratch for AI authorship where the end-user experience is the only output that matters.

- Full requirements: `docs/PRD.md`
- Roadmap & phases: `docs/ROADMAP.md`
- Execution details: `docs/PROJECT_PLAN.md`
- Narrative spec: `docs/spec/voce-ir-spec.html`
- Deep research & landscape analysis: `docs/research/DEEP_RESEARCH.md`
- Data layer & backend integration: `docs/research/DATA_INTEGRATION.md`
- Security, testing, tooling, AI strategy: `docs/research/SECURITY_TESTING_TOOLING.md`
- Forms, SEO, internationalization: `docs/research/FORMS_SEO_I18N.md`
- Adoption, migration, ecosystem: `docs/research/ADOPTION_MIGRATION.md`
- Conversational design philosophy: `docs/research/CONVERSATIONAL_DESIGN.md`
- Voice interface & AI tool integration: `docs/research/VOICE_AND_AI_INTEGRATION.md`
- Animation, assets & deployment: `docs/research/ANIMATION_ASSETS_DEPLOY.md`
- Memory, decisions & project continuity: `docs/research/MEMORY_AND_DECISIONS.md`
- **Implementation plans:** `docs/plans/MASTER_PLAN.md` (overview), `docs/plans/sprint-*.md` (per-sprint detail)

## Current Phase

**v1.0.0 — All 6 Phases COMPLETE**

50 sprints. 9 Rust crates. 4 TypeScript packages. 7 compile targets (DOM, WebGPU, WASM, Hybrid, iOS/SwiftUI, Android/Compose, Email HTML). 73 tests. Schema → Validator → Compiler → AI Bridge → Inspector → Ecosystem.

Current task status — update this section as work progresses:
- [x] Deep research & landscape analysis completed (`docs/research/DEEP_RESEARCH.md`)
- [x] Rust workspace initialized (edition 2024, 3 packages, CI, CLI skeleton)
- [x] FlatBuffers schema: types.fbs (primitive + composite types)
- [x] FlatBuffers schema: layout.fbs (ViewRoot, Container, Surface, TextNode, MediaNode)
- [x] FlatBuffers schema: state.fbs (StateMachine, DataNode, ComputeNode, EffectNode, ContextNode)
- [x] FlatBuffers schema: motion.fbs (AnimationTransition, Sequence, GestureHandler, ScrollBinding, PhysicsBody, ReducedMotion)
- [x] FlatBuffers schema: navigation.fbs (RouteMap, RouteEntry, RouteTransition, RouteGuard)
- [x] FlatBuffers schema: a11y.fbs (SemanticNode, LiveRegion, FocusTrap)
- [x] FlatBuffers schema: theming.fbs (ThemeNode, ColorPalette, TypographyScale, SpacingScale, PersonalizationSlot, ResponsiveRule)
- [x] FlatBuffers schema: data.fbs (ActionNode, SubscriptionNode, AuthContextNode, ContentSlot, RichTextNode)
- [x] FlatBuffers schema: forms.fbs (FormNode, FormField, ValidationRule, FormSubmission)
- [x] FlatBuffers schema: seo.fbs (PageMetadata, OpenGraphData, StructuredData)
- [x] FlatBuffers schema: i18n.fbs (LocalizedString, MessageCatalog, FormatOptions, I18nConfig)
- [x] FlatBuffers schema: voce.fbs (master file — 27-type ChildUnion, VoceDocument with auth + i18n)
- [ ] Generated Rust bindings
- [ ] Generated TypeScript bindings
- [x] Validator: structural completeness pass (STR001-STR005)
- [x] Validator: reference resolution pass (REF001-REF009)
- [x] Validator: state machine validation pass (STA001-STA004)
- [x] Validator: accessibility pass (A11Y001-A11Y005 — keyboard equiv, heading hierarchy, alt text, form semantics)
- [x] Validator: motion safety pass (MOT001-MOT005 — ReducedMotion required, damping > 0, duration warning)
- [x] Validator: security pass (SEC001-SEC004 — CSRF on mutations, auth redirects, HTTPS, password autocomplete)
- [x] Validator: SEO pass (SEO001-SEO007 — title, description length, h1 count, OG completeness)
- [x] Validator: forms pass (FRM001-FRM009 — fields required, labels, unique names, email validation)
- [x] Validator: i18n pass (I18N001-I18N003 — localized key non-empty, default value, consistency)
- [x] Unified `voce` CLI: validate (colored + JSON output), inspect (summary), json2bin, bin2json (via flatc)
- [x] Test suite: 37 tests passing (12 schema + 24 validator + 1 doctest)
- [x] JSON ↔ binary round-trip tooling (json2bin/bin2json via flatc)
- [x] Hand-authored reference landing page IR (37 nodes, 11 node types, validates cleanly)
- [x] Intent-IR pairs: 2 pairs (hero section, contact form) with intent.md + ir.voce.json
- [x] ARCHITECTURE.md written
- [x] CHANGELOG.md v0.1.0 release notes
- [x] Phase 1 complete — ready for Phase 2 (DOM Compiler)

## Architecture

```
voce-ir/
├── packages/
│   ├── schema/          Rust lib — FlatBuffers schema + generated bindings
│   ├── validator/       Rust bin — Reference IR validator
│   ├── compiler-dom/    Rust bin — DOM compile target
│   └── ai-bridge/       TypeScript — AI generation layer (Phase 3)
├── tests/               Integration + golden file tests
└── examples/            Reference IR blobs + compiled output
```

## Conventions

### Rust
- Edition 2024 (latest stable), minimum Rust version 1.85. Fall back to 2021 per-crate only if a dependency requires it (e.g., FlatBuffers codegen)
- Error handling: `thiserror` for library errors, `anyhow` for CLI entry points
- CLI args: `clap` derive API
- Tests: unit tests in-file (`#[cfg(test)]`), integration tests in `tests/`
- Naming: `snake_case` for files/functions, `PascalCase` for types/enums
- Every public function and type has a `///` doc comment
- No `unwrap()` in library code — propagate with `?`
- Format: `cargo fmt` (default settings)
- Lint: `cargo clippy -- -D warnings` (zero warnings policy)

### Schema
- FlatBuffers `.fbs` files live in `packages/schema/schemas/`
- One file per domain (layout, state, motion, navigation, a11y, theming, types)
- `voce.fbs` is the master include file
- After editing schemas, regenerate: `flatc --rust -o packages/schema/src/generated/ packages/schema/schemas/*.fbs`
- Also generate TS bindings: `flatc --ts -o packages/ai-bridge/src/generated/ packages/schema/schemas/*.fbs`

### Testing
- Every new node type needs valid + invalid test IR in `tests/schema/`
- Compiler output uses `insta` snapshot testing
- Run full suite before committing: `cargo test --workspace`

## Build & Test

```bash
# Build everything
cargo build --workspace

# Run all tests
cargo test --workspace

# Lint (must pass with zero warnings)
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --check

# Regenerate FlatBuffers bindings (after schema changes)
flatc --rust -o packages/schema/src/generated/ packages/schema/schemas/voce.fbs
```

## Key Design Decisions

1. **FlatBuffers** for IR format (zero-copy deserialization, schema evolution). FlatBuffers are immutable — runtime mutable state lives in a separate reactive layer, not in the buffer
2. **JSON canonical representation** round-trips to/from binary. Used for: AI generation (LLMs emit JSON), debugging, version control diffing, escape hatch. Not human-authored code — machine-readable text serialization of the IR
3. **Rust** for validator + compiler (performance, correctness, WASM compilation path)
4. **Taffy** (Rust) for compile-time layout resolution (flexbox/grid engine, successor to Yoga)
5. **TypeScript** for AI bridge (Claude API integration, preview server)
6. **Single-file HTML output** from DOM compiler (simplest deployment, zero network deps)
7. **Accessibility is a compile error** — missing SemanticNode = validation failure. Support explicit opt-outs (`decorative: true`, `presentation: true`) for valid exceptions
8. **Binary IR is not human-readable** — this is by design, not a limitation
9. **SPIR-V pipeline analogy** — binary IR + formal schema + validator + multi-target compilation. Study SPIR-V for pipeline design, Compose's slot table for data structure design

## Reference Architectures

When making design decisions, refer to these precedents:
- **SPIR-V** — overall pipeline (binary IR → validator → multi-target compiler)
- **Compose slot table** — flat binary representation of UI composition state
- **SolidJS/Svelte compiled output** — surgical DOM mutation patterns (the DOM compiler should emit similar code)
- **Flutter semantics tree** — parallel accessibility tree alongside visual rendering

## Working Pattern

When implementing a new feature:
1. Start with the `.fbs` schema (if new types needed)
2. Regenerate bindings
3. Add validation pass in `packages/validator/src/passes/`
4. Add compiler support in `packages/compiler-dom/src/codegen/`
5. Write tests (valid IR, invalid IR, compiler snapshot)
6. Run `cargo test --workspace && cargo clippy --workspace -- -D warnings`
7. Commit with message: `feat(scope): description` or `fix(scope): description`

## Important Context

- This is a Fire Burns Up project (not Resonia, not Deyan)
- Named from "sotto voce" — quiet input, extraordinary output
- The three pillars are non-negotiable: Stability, Experience, Accessibility. Security is part of Stability
- The IR will eventually support 3D (Scene3D, MeshNode, ShaderNode, ParticleSystem) but Phase 1 focuses on 2D layout primitives
- Target users are AI-first builders who work through conversation, not code
- **Anti-vibe-coding:** The AI is an inquisitive collaborator, not a servant. It asks questions (one at a time), builds context, pushes back on anti-patterns, and ensures full-stack feature completeness before generating. No TODOs. No half-implementations. See `docs/research/CONVERSATIONAL_DESIGN.md`
- **Memory is non-negotiable:** The `.voce/` directory stores the project brief (north star), decision log (with rationale and conflict detection), session history (survives interruptions), and user preferences. Every request is checked against the brief. Drift is caught. Decisions form a traceable chain. See `docs/research/MEMORY_AND_DECISIONS.md`
- Security, testing, and documentation are automatic — the system generates them, users don't configure them
- The compiled output has zero runtime dependencies, eliminating the supply chain attack surface entirely
- See `docs/research/SECURITY_TESTING_TOOLING.md` for security framework, testing strategy, documentation system, CLI design, AI model strategy, and style pack system
