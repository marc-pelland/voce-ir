# Changelog

## v1.0.0 — Phase 6: Ecosystem & Community (2026-04-02)

Native compile targets and email HTML compiler — the full multi-platform story.

### Native Compilers
- **iOS (SwiftUI):** Container→VStack/HStack, TextNode→Text, Surface→Card, MediaNode→AsyncImage, VoiceOver accessibility
- **Android (Compose):** Container→Column/Row, TextNode→Text composable, Surface→Card, Material3, TalkBack

### Email HTML Compiler
- Table-based layout, inline CSS only, XHTML transitional, 600px max-width, email-client safe

### Stats at v1.0.0
- 9 Rust crates, 4 TypeScript packages, 7 compile targets
- 73 automated tests, 50 sprints across 6 phases
- Same IR → DOM, WebGPU, WASM, iOS, Android, Email

---

## v0.5.0 — Phase 5: Visual Inspector & Tooling (2026-04-02)

Runtime debugging tool that lets humans inspect and debug compiled Voce IR applications without reading code.

### Inspector Core (S39)
- Click-to-inspect: click any element → highlight + IR node properties
- Scene graph tree: color-coded node hierarchy with type labels
- Property panel: computed styles, ARIA attributes, dimensions

### State & Animation (S40)
- State machine visualizer: live current-state highlighting, transition history (last 50)
- Animation timeline: scrubable, play/pause/step, speed control (0.25x-2x)
- Data flow monitor: live DataNode status (loading/loaded/error)

### Accessibility & Performance (S41)
- A11y tree viewer: ARIA roles, accessible names, missing semantic detection
- Focus order overlay: numbered badges showing tab order
- Screen reader preview: text-only output of what a screen reader announces
- Performance profiler: FPS, ms/frame, jank detection, frame graph

### CMS Visual Editing (S42)
- Inline text editing: click any TextNode to edit with contenteditable
- Content change → IR patch generation
- CMS bridge adapter interface (pluggable: Contentful, Sanity, etc.)
- Preview/publish flow: edit → preview → confirm → persist
- Undo/redo for content edits

### Conversational Debugging (S43)
- Bug description → AI-assisted diagnosis with pattern matching
- State machine tracing: identifies failing transitions/guards
- Fix proposals: generates IR patches for common issues
- State replay: reproduce bugs via recorded transition sequences
- Multi-turn debugging conversation with context

### Stats
- Inspector package: 12 TypeScript source files, 12.75 KB type declarations
- 12 inspector panel modules across 4 categories (inspection, visualization, editing, debugging)

---

## v0.4.0 — Phase 4: Multi-Target Compilation (2026-04-02)

Same IR compiles to DOM, WebGPU, and WASM with automatic per-component target selection.

### WebGPU Compiler (`compiler-webgpu`)
- Scene3D → WebGPU device initialization + render loop
- Camera system (perspective, orbit controls, auto-rotate)
- Lighting (directional, point, ambient)
- Mesh rendering (cube, sphere, plane primitives)
- WGSL shaders: PBR fragment (metallic/roughness), unlit, standard vertex
- ParticleSystem → GPU compute shader (spawn, update, render as billboarded quads)
- Material system with 14 presets (brushed metal, chrome, gold, wood, glass, etc.)
- Single-file HTML output with embedded WebGPU + WGSL

### WASM Compiler (`compiler-wasm`)
- StateMachine → WAT functions (state table in linear memory)
- ComputeNode → WAT pure functions (expression → f64.add/mul/etc)
- JS interop bridge with auto-initialization
- Expression compiler for simple math operations

### Hybrid Compiler (`compiler-hybrid`)
- Per-component target analysis: DOM vs WebGPU vs WASM based on node type + device profile
- Device profiles: desktop (GPU), mobile-high, mobile-low (no GPU)
- Unified output bundling: single HTML with DOM + WebGPU canvas + WASM module
- Graceful degradation: WebGPU → Canvas 2D → static image → noscript text
- Runtime capability detection (<50ms overhead)

### AI Bridge 3D Extensions
- Material description → PBR parameter mapping (14 presets)
- Lighting description → light configuration (6 presets)
- Camera behavior mapping (5 presets)
- 3D context builder for Generator Agent system prompt

### Demo
- 3D product viewer: hybrid DOM + WebGPU page (18 nodes, validates cleanly)

### Stats
- 6 Rust crates in workspace
- 73 automated tests
- Benchmark templates for WebGPU vs Three.js and WASM vs JS comparisons

---

## v0.3.0 — Phase 3: AI Bridge & Experience (2026-04-02)

The AI bridge connects natural language to IR generation via a multi-agent architecture with conversational design, style packs, persistent memory, and voice interface.

### AI Bridge
- **Multi-agent pipeline:** Discovery Agent (quality gate), Design Agent (UX patterns), Generator Agent (IR emission), Repair Agent (validation fix loop, >99% validity)
- **Conversational design:** One question at a time, readiness scoring, brief confirmation before generation. Anti-vibe-coding: blocks generation on vague prompts
- **Style packs:** 3 curated packs (minimal-saas, editorial, ecommerce) with design tokens + RAG retrieval for few-shot prompting
- **Incremental generation:** JSON Patch (RFC 6902) for surgical edits without full regeneration. Undo support via patch history.
- **Memory system:** `.voce/` persistence — brief enforcement (check requests against north star), decision logging with conflict detection, session recovery, drift detection
- **Provider config:** `voce.config.toml` for API keys, model selection, style pack defaults

### Integrations
- **MCP server:** 6 tools (validate, compile, inspect, schema, examples, generate) + 2 resources (brief, status). Use from Claude Code via `claude mcp add voce`
- **TypeScript SDK:** `@voce-ir/sdk` with `VoceClient` class — generate, validate, compile, inspect
- **Voice interface:** STT/TTS provider abstraction, push-to-talk engine, voice-tuned prompts, seamless voice/text switching

### CLI (expanded to 9 commands)
- `voce generate` — natural language to validated, compiled HTML
- `voce preview` — compile + open in browser
- `voce report` — compilation quality metrics
- `voce manifest` — application summary
- Previous: validate, inspect, compile, json2bin, bin2json

### Demo Projects
- 3 complete demos: SaaS landing page, contact form, multi-section marketing site
- Benchmark template comparing Voce IR vs v0/bolt.new
- Examples README documenting all reference IR and intent-IR pairs

---

## v0.2.0 — Phase 2: DOM Compiler (2026-04-02)

The DOM compiler takes validated IR and emits production-quality HTML. Zero framework runtime, pre-computed inline styles, surgical JS, and full accessibility/security.

### Compiler

- **Compiler pipeline:** Ingest → Optimize → Lower → Emit, arena-based IR
- **Layout compilation:** Container → flexbox/grid, Surface → styled div, TextNode → semantic headings, MediaNode → responsive picture
- **State & interaction:** StateMachine → JS state variable + transition table (~3 lines per machine), GestureHandler → addEventListener with keyboard equivalent
- **Animation:** Compile-time spring ODE solver → CSS `linear()` easing (zero JS for spring physics), `@media (prefers-reduced-motion: reduce)` overrides
- **Theming:** ThemeNode → CSS custom properties on `:root`
- **SEO:** PageMetadata → complete `<head>` (title, description, canonical, OG, structured data JSON-LD)
- **Forms:** FormNode → native `<form>` with progressive enhancement, client-side validation JS generated from ValidationRules, accessible labels/ARIA
- **Assets:** Responsive `<picture>` with srcset at 6 widths, above-fold preload hints, lazy loading
- **Security:** CSP meta tag, X-Frame-Options, X-Content-Type-Options, strict referrer. No eval/innerHTML/document.write
- **Accessibility:** SemanticNode → ARIA role/label/tabindex on HTML elements. Decorative elements get `aria-hidden`

### CLI (expanded)

- `voce compile` — IR → HTML output with validation check
- `voce preview` — compile + open in browser
- `voce report` — compilation quality metrics (validation, structure, features, output size, <10KB check)
- `voce manifest` — human-readable application summary
- Total: 8 CLI commands (validate, inspect, compile, preview, report, manifest, json2bin, bin2json)

### Output Quality

- **Landing page: 6,652 bytes** (6.5 KB) — 33% under the 10KB target
- Zero framework runtime
- Includes: layout, typography, colors, theme system, state machine, click/keyboard events, spring physics, reduced motion, form with validation, SEO, security headers, ARIA accessibility
- 59 automated tests

---

## v0.1.0 — Phase 1: Schema & Foundation (2026-04-02)

The first release of Voce IR establishes the complete IR schema, a working validator with 9 passes and 46 error codes, and a CLI toolchain for validation, inspection, and format conversion.

### Schema

- **12 FlatBuffers schema files** across 11 domains: types, layout, state, motion, navigation, accessibility, theming, data, forms, SEO, i18n
- **27 node types** in the ChildUnion covering all domains needed for production web applications
- **~100 FlatBuffers types** (tables, enums, structs, unions)
- **JSON canonical format** with verified round-trip to FlatBuffers binary
- `VOCE` file identifier for binary format verification

### Validator

- **9 validation passes:** structural, references, state-machine, accessibility, security, SEO, forms, i18n, motion safety
- **46 error codes** with actionable messages and node path references
- **Serde IR model** for ergonomic multi-pass validation
- **NodeIndex** for O(1) node lookup by ID and type
- Accessibility violations are `Error` severity (compile errors, not warnings)
- ReducedMotion required on all animation types
- CSRF required on mutation ActionNodes
- Heading hierarchy enforcement (no level skipping)

### CLI

- `voce validate` — colored terminal output with error/warning counts, `--format json` for CI, `--warn-as-error` flag
- `voce inspect` — human-readable IR summary (node counts, types, depth, features)
- `voce json2bin` — JSON to FlatBuffer binary conversion (via flatc)
- `voce bin2json` — FlatBuffer binary to JSON conversion (via flatc)
- Exit codes: 0 (valid), 1 (errors), 2 (tool error)

### Examples

- **Reference landing page IR** — 37 nodes, 11 node types, with PageMetadata, theme, state machine, form, animations, responsive rules. Validates with zero errors.
- **2 intent-IR training pairs** — hero section and contact form, with natural language descriptions paired with complete IR

### Tests

- **37 automated tests** across the workspace
- 12 invalid IR fixtures covering structural, reference, state machine, a11y, security, SEO, forms, i18n, and motion passes
- Schema round-trip tests for all node type domains
- Landing page and intent-IR validation tests
- Diagnostic quality assertions (non-empty codes, paths, messages)

### Documentation

- `docs/ARCHITECTURE.md` — crate structure, IR format, validation pipeline, key decisions
- `docs/plans/MASTER_PLAN.md` — 50-sprint plan across 6 phases
- 10 detailed sprint plans in `docs/plans/`
- 9 research documents in `docs/research/`

### Infrastructure

- Rust workspace (edition 2024) with 3 crates: voce-schema, voce-validator, voce-compiler-dom
- GitHub Actions CI (build, test, clippy, fmt)
- `.voce/` memory directory with project brief template
- `scripts/regenerate-schema.sh` for FlatBuffers codegen
