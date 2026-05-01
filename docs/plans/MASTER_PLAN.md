# Voce IR — Master Implementation Plan

**Created:** 2026-04-02
**Last updated:** 2026-04-02
**Status:** Phases 1-6 COMPLETE (v1.0.0). 50 sprints done. Phase 7 (Production Readiness): S51-S59 COMPLETE. S60 (Community Launch) planned. S61–S63 (Self-Demonstrating Site trilogy: live hero → multi-target → gallery) planned. S64 (Compiler-Emits-Rich-Defaults) planned, motivated by S61 regression.

---

## How This Plan Works

The implementation is broken into **sprints** — logical chunks of work that each produce a demonstrable result. Each sprint has its own detailed document in `docs/plans/` with:

- Specific deliverables and acceptance criteria
- File-by-file implementation details
- Dependencies on previous sprints
- Estimated effort
- Review checklist before moving to the next sprint

**Workflow:** Review the sprint plan → build it → verify acceptance criteria → review next sprint plan → repeat.

Sprint plans are living documents — update them as you learn things during implementation. The plan serves the work, not the other way around.

---

## Completed Work

### S01 — Project Bootstrap (Done)

- Rust workspace (edition 2024, Rust 1.93) with 3 packages: schema, validator, compiler-dom
- Unified `voce` CLI skeleton with 4 subcommands (validate, inspect, json2bin, bin2json)
- Typed validation diagnostics (Severity, Diagnostic, ValidationResult)
- GitHub Actions CI (build, test, clippy, fmt)
- `.voce/` memory directory with brief template
- Full directory structure: tests/, examples/, style-packs/

### S02 — Core Schema: Types & Layout (Done)

- `types.fbs` — 13 length units, Color/Vec2/Vec3/Vec4 structs, Easing (Linear/CubicBezier/Spring/Steps/CustomLinear), Alignment, LayoutDirection, EdgeInsets, CornerRadii, Shadow, Border, DataBinding
- `layout.fbs` — ViewRoot, Container (Stack/Flex/Grid/Absolute), Surface, TextNode (full typography), MediaNode (responsive hints, loading strategy)
- Generated Rust bindings, JSON round-trip verified via flatc (908 bytes for minimal page)
- 3 passing tests

### S03 — Extended Schema: State, Motion, Navigation (Done)

- `state.fbs` — StateMachine (states + transitions + guards + effects), DataNode (6 providers, cache strategies, cache tags), ComputeNode, EffectNode (7 types + idempotency), ContextNode
- `motion.fbs` — AnimationTransition, Sequence (steps + stagger + iterations), GestureHandler (8 gesture types + keyboard equivalent), ScrollBinding (ViewProgress/ScrollProgress), PhysicsBody (interruptible flag), ReducedMotion (Remove/Simplify/ReduceDuration/Functional)
- `navigation.fbs` — RouteMap, RouteEntry (path + guard + sitemap metadata + nested children), RouteGuard (auth/roles/redirect), RouteTransitionConfig (None/Crossfade/Slide/SharedElement/Custom), SharedElementPair
- ChildUnion expanded to 15 types across all domains
- Combined schema generation script (avoids FlatBuffers cross-module issues)
- 6 passing tests including StateMachine creation, spring animation, and union completeness

### S04 — Schema: A11y & Theming (Done)

- `a11y.fbs` — SemanticNode (full ARIA: role, label, labelled_by, described_by, heading_level, tab_index, ARIA states), LiveRegion (Polite/Assertive/Off, atomic, relevant), FocusTrap (container, initial focus, escape behavior)
- `theming.fbs` — ThemeNode (ColorPalette 21 slots, TypographyScale, SpacingScale, ShadowScale, RadiusScale), PersonalizationSlot (6 condition types, variants with show/hide/override), ResponsiveRule (breakpoints + per-breakpoint overrides)
- Added `semantic_node_id` to Container, Surface, TextNode, MediaNode
- ChildUnion expanded to 21 types, 9 passing tests

### S05 — Schema: Data, Forms, SEO, i18n (Done)

- `data.fbs` — ActionNode (optimistic updates, cache invalidation, CSRF, retry), SubscriptionNode (WebSocket/SSE/Polling), AuthContextNode (6 providers), ContentSlot (Static/ISR/Dynamic cache), RichTextNode (12 block types, 7 marks)
- `forms.fbs` — FormNode (19 field types, 4 validation modes, autosave), FormField (validations, async validations, autocomplete), ValidationRule (9 types), FormSubmission (progressive enhancement)
- `seo.fbs` — PageMetadata (title, OG, Twitter, robots, hreflang, structured data), StructuredData (JSON-LD)
- `i18n.fbs` — LocalizedString (message key + typed parameters), MessageCatalog, I18nConfig (static/runtime mode)
- **Schema is now feature-complete**: 27-type ChildUnion, 12 .fbs files, ~100 FlatBuffers types, 23,776 lines generated, 12 tests

### S06 — Validator: Core Passes (Done)

- ValidationPass trait with `fn name()`, `fn run()`, `fn dependencies()`
- Serde-based IR deserialization model (JSON canonical format → typed Rust structs)
- NodeIndex for O(1) cross-reference lookups by node ID
- 3 core passes: structural completeness, type checking, reference resolution
- Typed Diagnostic with severity, error code, node path, expected/actual values

### S07 — Validator: Pillar Passes (Done)

- 6 pillar validation passes: accessibility, security, SEO, forms, i18n, motion safety
- 46 distinct error codes covering all validation categories
- Accessibility: SemanticNode coverage for interactive nodes, heading hierarchy, alt text
- Security: auth guards on protected routes, allowed origins, no eval/innerHTML patterns
- Motion safety: ReducedMotion required for all Transitions/Sequences, `motion_functional` for loading indicators
- Forms: validation rules, progressive enhancement, autocomplete attributes
- SEO: title/description presence, heading structure, OG metadata
- i18n: message key format validation, missing translation detection

### S08 — CLI & Tooling (Done)

- Working `voce validate` with colored terminal output, error grouping by severity, exit codes
- Working `voce inspect` — human-readable IR summary (node tree, stats, warnings)
- Working `voce json2bin` and `voce bin2json` — lossless format conversion
- Clap derive API with consistent flags and help text across all subcommands

### S09 — Examples & Polish (Done)

- Reference landing page IR: 37 nodes (ViewRoot, Containers, Surfaces, TextNodes, MediaNodes, StateMachines, GestureHandlers, Transitions, SemanticNodes, ThemeNode, ResponsiveRules)
- 2 complete intent-IR pairs with natural language descriptions
- 37 passing tests covering all node types and validation passes

### S10 — Documentation & Release (Done)

- ARCHITECTURE.md documenting SPIR-V analogy, FlatBuffers decision, Taffy layout model, compile-time constraint resolution
- CHANGELOG.md for v0.1.0
- Tagged v0.1.0 release

### Key Technical Decisions Made

- **Edition 2024** for validator/compiler, **edition 2021** for schema package (FlatBuffers codegen compat)
- **Single combined file** for FlatBuffers Rust codegen — individual .fbs files are source of truth for editing, `scripts/regenerate-schema.sh` combines them for compilation
- **Wrapper table pattern** for union vectors — `ChildNode { value: ChildUnion }` because FlatBuffers Rust doesn't support `[union]` directly
- **Required fields** change the Rust API — return `&str` / `Vector` directly instead of `Option`

---

## Phase 1: Schema & Foundation (Complete)

| Sprint | Focus | Key Deliverables | Status |
| ------ | ----- | ---------------- | ------ |
| **S01** | Project Bootstrap | Rust workspace, CI, FlatBuffers toolchain, `.voce/` skeleton | **Done** |
| **S02** | Core Schema: Types & Layout | `types.fbs`, `layout.fbs`, generated bindings, JSON round-trip | **Done** |
| **S03** | Schema: State, Motion, Nav | `state.fbs`, `motion.fbs`, `navigation.fbs`, expanded ChildUnion | **Done** |
| **S04** | Schema: A11y, Theming | `a11y.fbs`, `theming.fbs` — SemanticNode, LiveRegion, FocusTrap, ThemeNode, ResponsiveRule | **Done** |
| **S05** | Schema: Data, Forms, SEO, i18n | `data.fbs`, `forms.fbs`, `seo.fbs`, `i18n.fbs` — 27-type ChildUnion, complete schema | **Done** |
| **S06** | Validator: Core Passes | ValidationPass trait, serde IR model, NodeIndex, 3 core passes | **Done** |
| **S07** | Validator: Pillar Passes | 6 pillar passes (a11y, security, SEO, forms, i18n, motion), 46 error codes | **Done** |
| **S08** | CLI & Tooling | Working validate/inspect/json2bin/bin2json, colored output, exit codes | **Done** |
| **S09** | Examples & Polish | Reference landing page IR (37 nodes), 2 intent-IR pairs, 37 tests | **Done** |
| **S10** | Documentation & Release | ARCHITECTURE.md, CHANGELOG.md, v0.1.0 | **Done** |

**Phase 1: COMPLETE (10/10 sprints) — v0.1.0**

---

## Phase 2: DOM Compiler (Complete)

Takes validated IR and emits production-quality HTML. The reference landing page compiles to a 6.6KB HTML file with <50ms TTI, 0 a11y violations, security headers, and optimized assets.

| Sprint | Focus | Key Deliverables | Status |
| ------ | ----- | ---------------- | ------ |
| **S11** | Compiler Pipeline | CompilerIr, NodeArena, pipeline (ingest→optimize→lower→emit), HTML skeleton, voce compile | **Done** |
| **S12** | Layout Compilation | Container → flexbox/grid, Surface → styled div, TextNode → h1-h6, grid-template-columns, sizing | **Done** |
| **S13** | State & Interaction | StateMachine → JS (3 lines/machine), GestureHandler → addEventListener + keyboard | **Done** |
| **S14** | Animation | Compile-time spring ODE → CSS linear(), CSS transitions, @media reduced-motion | **Done** |
| **S15** | Theming, SEO, i18n | ThemeNode → :root CSS vars, PageMetadata → head, StructuredData → JSON-LD | **Done** |
| **S16** | Forms & Data | FormNode → form HTML + validation JS, progressive enhancement, ARIA | **Done** |
| **S17** | Asset Pipeline | Responsive picture/srcset, preload hints, lazy loading, asset module | **Done** |
| **S18** | Security & A11y | CSP meta tag, SemanticNode → ARIA attrs + tabindex, decorative aria-hidden | **Done** |
| **S19** | Testing & Reports | voce report + voce manifest commands, compilation quality metrics | **Done** |
| **S20** | Deployment & v0.2.0 | voce preview, CHANGELOG v0.2.0 | **Done** |

---

## Phase 3: AI Bridge & Experience (Complete)

Connects natural language to IR generation. Describe a landing page and get a deployed site in under 5 minutes, with >95% IR validity on first attempt.

| Sprint | Focus | Key Deliverables | Status |
| ------ | ----- | ---------------- | ------ |
| **S21** | AI Bridge Foundation | TypeScript project, Claude API, first end-to-end: prompt → IR → validate → compile → HTML | **Done** |
| **S22** | Multi-Agent Architecture | Discovery Agent (quality gate), Design Agent, Generator, Repair Agent, orchestrator | **Done** |
| **S23** | Conversational Design | One-question-at-a-time, readiness score, brief builder, plan confirmation | **Done** |
| **S24** | Style Packs & RAG | 3 style packs (minimal-saas, editorial, ecommerce), RAG retrieval, few-shot | **Done** |
| **S25** | Memory & Decisions | .voce/ persistence, brief enforcement, decision log, drift detection | **Done** |
| **S26** | Incremental Generation | JSON Patch deltas, patch agent, hierarchical generation, undo | **Done** |
| **S27** | MCP Server & Integrations | MCP server (6 tools), TypeScript SDK, voce.config.toml | **Done** |
| **S28** | Voice Interface | STT/TTS, `voce talk`, push-to-talk, voice/text switching | **Done** |
| **S29** | Demo Projects | 3 demos via conversation, screen recordings, benchmarks | **Done** |
| **S30** | Public Launch | Blog post, web playground, npm/crates publish, v0.3.0, public repo | **Done** |

**Phase 3: COMPLETE (10/10 sprints) — v0.3.0**

### Completed Work: Phase 3 AI Bridge

**S21 — AI Bridge Foundation (Done)**
- TypeScript project (packages/ai-bridge) with Claude API integration
- Structured prompting: system prompt with full IR schema + example library
- First end-to-end pipeline: natural language → IR JSON → validate → compile → HTML
- Validate-repair loop: validation errors fed back as follow-up prompt (max 2 cycles)
- >80% first-attempt validity rate on initial test suite

**S22 — Multi-Agent Architecture (Done)**
- Discovery Agent (Opus): quality gate, scope clarification, constraint checking
- Design Agent (Opus): ambiguity resolution, UX pattern selection, OWASP/WCAG/Nielsen enforcement
- Generator (Sonnet): IR JSON emission with structured output
- Repair Agent (Haiku/Sonnet): fast validation error correction
- Orchestrator routing between agents based on conversation phase

**S23 — Conversational Design (Done)**
- One-question-at-a-time discovery flow (anti-vibe-coding)
- Readiness scoring: AI tracks completeness before generating
- Brief builder: structured project brief from conversation
- Plan confirmation: user reviews plan before generation begins
- Pushback on underspecified or conflicting requirements

**S24 — Style Packs & RAG (Done)**
- 3 style packs: minimal-saas, editorial, ecommerce (design tokens + layout patterns + example IR)
- RAG retrieval: semantic matching of user intent to closest golden pairs
- 50+ intent-IR golden pairs for few-shot demonstrations
- Style pack selection integrated into discovery flow

**S25 — Memory & Decisions (Done)**
- .voce/ directory persistence: brief.json, decisions.json, sessions/
- Brief enforcement: AI validates new requests against established brief
- Decision log: every design decision recorded with rationale
- Drift detection: warns when requests contradict previous decisions
- Session persistence: resume conversations with full context

**S26 — Incremental Generation (Done)**
- JSON Patch delta generation for targeted modifications
- Patch agent: specialized for single-property and structural changes
- Hierarchical generation: structure-first, then section-by-section population
- Undo support via patch reversal

**S27 — MCP Server & Integrations (Done)**
- MCP server exposing 6 tools: generate, patch, validate, compile, inspect, preview
- TypeScript SDK for programmatic access
- voce.config.toml for project-level configuration
- Integration with Claude Desktop and other MCP clients

**S28 — Voice Interface (Done)**
- Speech-to-text and text-to-speech integration
- `voce talk` command for voice-first workflow
- Push-to-talk mode with voice activity detection
- Seamless voice/text switching within sessions

**S29 — Demo Projects (Done)**
- Marketing landing page: hero, features grid, CTA, testimonials, footer
- Product detail page: image gallery, description, add-to-cart, reviews
- Portfolio site: multi-route navigation, project grid, contact form
- Screen recordings of each demo session
- Benchmark comparison vs v0/bolt.new equivalent output

**S30 — Public Launch (Done)**
- Technical blog post published
- Web playground for browser-based demos
- npm package published (ai-bridge), crates.io packages (schema, validator, compiler-dom)
- v0.3.0 tagged, public GitHub repo live

---

## Phase 4: Multi-Target Compilation (Complete)

Same IR compiles to DOM, WebGPU, and WASM with device-profile-aware target selection.

| Sprint | Focus | Key Deliverables | Status |
| ------ | ----- | ---------------- | ------ |
| **S31** | WebGPU Renderer Foundation | Scene3D compilation, camera system, lighting pipeline, mesh rendering | **Done** |
| **S32** | WebGPU Shaders & Particles | ShaderNode → WGSL transpilation, ParticleSystem → GPU compute shaders, PBR materials | **Done** |
| **S33** | WASM Code Generation | StateMachine → WASM functions, ComputeNode → WASM pure functions, JS interop bridge | **Done** |
| **S34** | Hybrid Compilation | Per-component target analysis (DOM/GPU/WASM), device profile matching, unified output bundling | **Done** |
| **S35** | Graceful Degradation | WebGPU → Canvas 2D → static image fallback chain, capability detection | **Done** |
| **S36** | Integration & AI Bridge | 3D scene descriptions in AI bridge, 3D intent vocabulary, style pack 3D extensions | **Done** |
| **S37** | 3D Product Viewer Demo | Interactive product viewer via conversation, orbit camera, environment lighting, annotations | **Done** |
| **S38** | Benchmarks & v0.4.0 | Performance benchmarks vs Three.js, output size analysis, documentation, v0.4.0 release | **Done** |

**Phase 4: COMPLETE (8/8 sprints) — v0.4.0**

### Completed Work: Phase 4 Multi-Target Compilation

**S31 — WebGPU Renderer Foundation (Done)**
- Scene3D node compilation to WebGPU render pipeline
- Camera system (perspective, orthographic, orbit controls)
- Directional, point, and ambient lighting
- Basic mesh rendering with vertex/index buffers

**S32 — WebGPU Shaders & Particles (Done)**
- ShaderNode → WGSL transpilation pipeline
- ParticleSystem → GPU compute shaders with configurable emitters
- PBR material system with metallic-roughness workflow

**S33 — WASM Code Generation (Done)**
- StateMachine → WASM functions via wasm-bindgen
- ComputeNode → WASM pure functions for compute-heavy operations
- JS interop bridge with typed bindings, memory management

**S34 — Hybrid Compilation (Done)**
- Per-component target analysis: DOM for layout/text, GPU for 3D/particles, WASM for compute
- Device profile matching for automatic target selection
- Unified output bundling combining DOM + WebGPU + WASM in single deployment

**S35 — Graceful Degradation (Done)**
- WebGPU → Canvas 2D → static image fallback chain
- Runtime capability detection with progressive enhancement
- Fallback rendering maintains visual fidelity within target constraints

**S36 — Integration & AI Bridge (Done)**
- AI bridge extended with 3D scene description vocabulary
- 3D intent patterns added to RAG library
- Style pack 3D extensions for product viewers, hero scenes, data visualization

**S37 — 3D Product Viewer Demo (Done)**
- Interactive product viewer built entirely via conversation
- Orbit camera with smooth damping, environment map lighting
- Annotation hotspots with DOM overlay integration

**S38 — Benchmarks & v0.4.0 (Done)**
- Benchmark suite: Voce WebGPU vs Three.js equivalent (smaller output, competitive FPS, faster TTI)
- Output size analysis across all compile targets
- v0.4.0 tagged and released

---

## Phase 5: Visual Inspector & Tooling (Complete)

Runtime debugging and content editing without code.

| Sprint | Focus | Key Deliverables | Status |
| ------ | ----- | ---------------- | ------ |
| **S39** | Inspector Core | Scene graph overlay, click-to-inspect, IR node property panel, IR-to-DOM mapping | **Done** |
| **S40** | State & Animation Inspector | State machine visualizer, animation timeline (pause/scrub/step), data flow monitor | **Done** |
| **S41** | A11y & Performance | Live a11y tree, focus order visualization, frame timing profiler, GPU utilization | **Done** |
| **S42** | CMS Visual Editing | Content click-to-edit, CMS bridge protocol, preview/publish flow, inline image replacement | **Done** |
| **S43** | Conversational Debugging | "Why doesn't X work?" → AI traces state machine, identifies node, proposes patch | **Done** |
| **S44** | Inspector Polish & v0.5.0 | Extension API for community plugins, keyboard shortcuts, documentation, v0.5.0 release | **Done** |

**Phase 5: COMPLETE (6/6 sprints) — v0.5.0**

### Completed Work: Phase 5 Visual Inspector

**S39 — Inspector Core (Done)**
- Scene graph overlay injection into compiled Voce output
- Click-to-inspect element selection with IR node property panel
- Source-map-like IR-to-DOM bidirectional mapping

**S40 — State & Animation Inspector (Done)**
- Live state machine visualizer: current state, transition history, guard evaluation
- Animation timeline with pause, scrub, and step-frame controls
- Data flow monitor showing DataNode/ComputeNode value propagation

**S41 — A11y & Performance Inspector (Done)**
- Live accessibility tree view mirroring semantic node structure
- Focus order visualization and tab-through simulator
- Frame timing profiler with per-node render cost breakdown
- GPU utilization monitoring for WebGPU compile targets

**S42 — CMS Visual Editing (Done)**
- Content click-to-edit overlay for TextNode and MediaNode content
- CMS bridge protocol: headless CMS → ContentSlot live updates
- Preview/publish flow with draft state management
- Inline image replacement with asset pipeline integration

**S43 — Conversational Debugging (Done)**
- Natural language bug description → AI traces state machine execution path
- Automatic IR node identification and patch proposal
- Error reproduction via state machine replay
- Integration with inspector panels for visual debugging context

**S44 — Inspector Polish & v0.5.0 (Done)**
- Extension API for community inspector plugins
- Keyboard shortcuts for all inspector operations
- Comprehensive documentation and usage guides
- v0.5.0 tagged and released

---

## Phase 6: Ecosystem & Community (Complete)

Native targets, plugin system, community growth.

| Sprint | Focus | Key Deliverables | Status |
| ------ | ----- | ---------------- | ------ |
| **S45** | Native: iOS (SwiftUI) | SwiftUI compile target, VoiceOver, gesture mapping, responsive layout | **Done** |
| **S46** | Native: Android (Compose) | Jetpack Compose compile target, TalkBack, Material Design tokens | **Done** |
| **S47** | Plugin System | Plugin API for validator passes, compile targets, content adapters, registry | **Done** |
| **S48** | Style Pack Marketplace | Community contribution workflow, marketplace UI, preview/install, revenue share | **Done** |
| **S49** | Email HTML Compile Target | Table layouts, inline CSS, Outlook/Gmail hacks, cross-client preview | **Done** |
| **S50** | Community & Governance | Community governance model, third-party integrations, v1.0.0 release | **Done** |

**Phase 6: COMPLETE (6/6 sprints) — v1.0.0**

### Completed Work: Phase 6 Ecosystem

**S45 — Native: iOS SwiftUI (Done)**
- SwiftUI compile target: IR nodes → SwiftUI view hierarchy
- VoiceOver accessibility integration (SemanticNode → accessibility modifiers)
- Gesture mapping: GestureHandler → SwiftUI gesture recognizers
- Responsive layout via GeometryReader and SwiftUI layout modifiers

**S46 — Native: Android Compose (Done)**
- Jetpack Compose compile target: IR nodes → Composable functions
- TalkBack integration (SemanticNode → Compose semantics)
- Material Design token mapping from ThemeNode
- Gesture mapping: GestureHandler → Compose pointer input

**S47 — Plugin System (Done)**
- Plugin API for custom validator passes (trait-based, hot-loadable)
- Plugin API for custom compile targets
- Content adapter plugins for headless CMS integration
- Plugin registry with versioning and dependency resolution

**S48 — Style Pack Marketplace (Done)**
- Community contribution workflow: submit, review, publish
- Marketplace UI with preview and one-click install
- Revenue share model for community style pack authors

**S49 — Email HTML Compile Target (Done)**
- Email-specific compilation: table-based layouts, inline CSS
- Outlook conditional comments, Gmail CSS reset
- Cross-client preview rendering (Apple Mail, Gmail, Outlook)
- Responsive email with media query fallbacks

**S50 — Community & Governance (Done)**
- Community governance model with RFC process
- Third-party integrations (Figma, Storybook, headless CMS)
- v1.0.0 tagged and released

---

## Phase 7: Production Readiness (S51-S59 Complete)

| Sprint | Focus | Key Deliverables | Status |
| ------ | ----- | ---------------- | ------ |
| **S51** | Real Image Processing | WebP/JPEG responsive variants, BlurHash placeholders, content-hash filenames | **Done** |
| **S52** | Deployment Adapters | 5 adapter crates, `voce deploy` CLI with --adapter and --dry-run | **Done** |
| **S53** | Web Playground | WASM bridge (646KB), Vite + TypeScript three-panel playground, 5 built-in examples | **Done** |
| **S54** | Integration Test Suite | 49 cross-target tests, 14 snapshot tests, 12 test fixtures, 153 total tests | **Done** |
| **S55** | Documentation Site | mdBook with 30 pages: getting started, CLI reference, schema reference, architecture, guides | **Done** |
| **S56** | Font Subsetting | Glyph collection, @font-face with font-display:swap, fallback stacks, preload hints | **Done** |
| **S57** | Production Error Handling | Unified VoceError taxonomy, compiler resilience with error placeholders, --verbose/--json-errors CLI flags | **Done** |
| **S58** | Production Demo Site | voce-ir.xyz landing page: 30+ nodes, 7.6KB output, 0.4s compilation | **Done** |
| **S59** | Performance Optimization | Criterion benchmarks: 209us landing page, 4.4us minimal. HTML minification, compilation cache. | **Done** |

### Completed Work: Phase 7 Production Readiness

**S51 — Real Image Processing:** WebP/JPEG responsive variants, BlurHash placeholders, content-hash filenames. Image pipeline feature-gated for WASM compatibility.

**S52 — Deployment Adapters:** 5 new crates: adapter-core, adapter-static, adapter-vercel, adapter-cloudflare, adapter-netlify. `voce deploy` CLI with --adapter and --dry-run flags.

**S53 — Web Playground:** WASM bridge (646KB) compiling validator + compiler-dom to browser. Vite + TypeScript three-panel playground with 5 built-in examples.

**S54 — Integration Test Suite:** 49 cross-target tests, 14 snapshot tests, 12 test fixtures. Total: 153 tests (up from 89).

**S55 — Documentation Site:** mdBook with 30 pages: getting started, CLI reference, schema reference, architecture, guides.

**S56 — Font Subsetting:** Font pipeline: glyph collection, @font-face with font-display:swap, fallback stacks, preload hints.

**S57 — Production Error Handling:** Unified VoceError taxonomy in schema crate, compiler resilience with error placeholders, --verbose/--json-errors CLI flags.

**S58 — Production Demo Site:** voce-ir.xyz landing page: 30+ nodes, 7.6KB output, 0.4s compilation.

**S59 — Performance Optimization:** Criterion benchmarks: 209us landing page, 4.4us minimal. HTML minification, compilation cache.

Phase 7 Stats: 15 Rust crates, 4 TypeScript packages, 172 tests.

---

## Sprint Document Index

| Sprint | Document | Status |
| ------ | -------- | ------ |
| S01 | [archive/sprint-01-bootstrap.md](archive/sprint-01-bootstrap.md) | **Complete** |
| S02 | [archive/sprint-02-core-schema.md](archive/sprint-02-core-schema.md) | **Complete** |
| S03 | [archive/sprint-03-extended-schema.md](archive/sprint-03-extended-schema.md) | **Complete** |
| S04 | [archive/sprint-04-a11y-theming.md](archive/sprint-04-a11y-theming.md) | **Complete** |
| S05 | [archive/sprint-05-data-forms-seo-i18n.md](archive/sprint-05-data-forms-seo-i18n.md) | **Complete** |
| S06 | [archive/sprint-06-validator-core.md](archive/sprint-06-validator-core.md) | **Complete** |
| S07 | [archive/sprint-07-validator-pillars.md](archive/sprint-07-validator-pillars.md) | **Complete** |
| S08 | [archive/sprint-08-cli-tooling.md](archive/sprint-08-cli-tooling.md) | **Complete** |
| S09 | [archive/sprint-09-examples-polish.md](archive/sprint-09-examples-polish.md) | **Complete** |
| S10 | [archive/sprint-10-docs-release.md](archive/sprint-10-docs-release.md) | **Complete** |
| S11 | [archive/sprint-11-compiler-pipeline.md](archive/sprint-11-compiler-pipeline.md) | **Complete** |
| S12 | [archive/sprint-12-layout-compilation.md](archive/sprint-12-layout-compilation.md) | **Complete** |
| S13 | [archive/sprint-13-state-interaction.md](archive/sprint-13-state-interaction.md) | **Complete** |
| S14 | [archive/sprint-14-animation.md](archive/sprint-14-animation.md) | **Complete** |
| S15 | [archive/sprint-15-theming-seo-i18n.md](archive/sprint-15-theming-seo-i18n.md) | **Complete** |
| S16 | [archive/sprint-16-forms-data.md](archive/sprint-16-forms-data.md) | **Complete** |
| S17 | [archive/sprint-17-asset-pipeline.md](archive/sprint-17-asset-pipeline.md) | **Complete** |
| S18 | [archive/sprint-18-security-a11y-quality.md](archive/sprint-18-security-a11y-quality.md) | **Complete** |
| S19 | [archive/sprint-19-testing-reports.md](archive/sprint-19-testing-reports.md) | **Complete** |
| S20 | [archive/sprint-20-deployment-benchmark.md](archive/sprint-20-deployment-benchmark.md) | **Complete** |
| S21 | [archive/sprint-21-ai-bridge-foundation.md](archive/sprint-21-ai-bridge-foundation.md) | **Complete** |
| S22 | [archive/sprint-22-multi-agent-architecture.md](archive/sprint-22-multi-agent-architecture.md) | **Complete** |
| S23 | [archive/sprint-23-conversational-design.md](archive/sprint-23-conversational-design.md) | **Complete** |
| S24 | [archive/sprint-24-style-packs-rag.md](archive/sprint-24-style-packs-rag.md) | **Complete** |
| S25 | [archive/sprint-25-memory-decisions.md](archive/sprint-25-memory-decisions.md) | **Complete** |
| S26 | [archive/sprint-26-incremental-generation.md](archive/sprint-26-incremental-generation.md) | **Complete** |
| S27 | [archive/sprint-27-mcp-server-integrations.md](archive/sprint-27-mcp-server-integrations.md) | **Complete** |
| S28 | [archive/sprint-28-voice-interface.md](archive/sprint-28-voice-interface.md) | **Complete** |
| S29 | [archive/sprint-29-demo-projects.md](archive/sprint-29-demo-projects.md) | **Complete** |
| S30 | [archive/sprint-30-public-launch.md](archive/sprint-30-public-launch.md) | **Complete** |
| S31 | [archive/sprint-31-webgpu-foundation.md](archive/sprint-31-webgpu-foundation.md) | **Complete** |
| S32 | [archive/sprint-32-webgpu-shaders-particles.md](archive/sprint-32-webgpu-shaders-particles.md) | **Complete** |
| S33 | [archive/sprint-33-wasm-codegen.md](archive/sprint-33-wasm-codegen.md) | **Complete** |
| S34 | [archive/sprint-34-hybrid-compilation.md](archive/sprint-34-hybrid-compilation.md) | **Complete** |
| S35 | [archive/sprint-35-graceful-degradation.md](archive/sprint-35-graceful-degradation.md) | **Complete** |
| S36 | [archive/sprint-36-integration-ai-3d.md](archive/sprint-36-integration-ai-3d.md) | **Complete** |
| S37 | [archive/sprint-37-3d-product-viewer.md](archive/sprint-37-3d-product-viewer.md) | **Complete** |
| S38 | [archive/sprint-38-benchmarks-v040.md](archive/sprint-38-benchmarks-v040.md) | **Complete** |
| S39 | [archive/sprint-39-inspector-core.md](archive/sprint-39-inspector-core.md) | **Complete** |
| S40 | [archive/sprint-40-state-animation-inspector.md](archive/sprint-40-state-animation-inspector.md) | **Complete** |
| S41 | [archive/sprint-41-a11y-performance-inspector.md](archive/sprint-41-a11y-performance-inspector.md) | **Complete** |
| S42 | [archive/sprint-42-cms-visual-editing.md](archive/sprint-42-cms-visual-editing.md) | **Complete** |
| S43 | [archive/sprint-43-conversational-debugging.md](archive/sprint-43-conversational-debugging.md) | **Complete** |
| S44 | [archive/sprint-44-inspector-polish-v050.md](archive/sprint-44-inspector-polish-v050.md) | **Complete** |
| S45 | [archive/sprint-45-native-ios-swiftui.md](archive/sprint-45-native-ios-swiftui.md) | **Complete** |
| S46 | [archive/sprint-46-native-android-compose.md](archive/sprint-46-native-android-compose.md) | **Complete** |
| S47 | [archive/sprint-47-plugin-system.md](archive/sprint-47-plugin-system.md) | **Complete** |
| S48 | [archive/sprint-48-style-pack-marketplace.md](archive/sprint-48-style-pack-marketplace.md) | **Complete** |
| S49 | [archive/sprint-49-email-html-target.md](archive/sprint-49-email-html-target.md) | **Complete** |
| S50 | [archive/sprint-50-community-governance-v100.md](archive/sprint-50-community-governance-v100.md) | **Complete** |

---

## Key Principles

1. **Each sprint produces a testable result.** Never end a sprint with untestable intermediate state.
2. **Schema sprints (S02-S05) generate bindings and write tests.** The schema isn't done until it compiles, round-trips, and has test coverage.
3. **Validator sprints (S06-S07) use test-driven development.** Write the failing test (invalid IR), then implement the pass that catches it.
4. **Update the plan as you learn.** Sprint estimates will be wrong. Scope will shift. The plan adapts.
5. **Review before starting.** Read the sprint plan, adjust if needed, then build.

---

## Research Library (Complete)

All research informing this plan lives in `docs/research/`:

| Document | Key Decisions |
| -------- | ------------- |
| `DEEP_RESEARCH.md` | SPIR-V analogy, FlatBuffers confirmed, Taffy for layout, <10KB validated |
| `SECURITY_TESTING_TOOLING.md` | OWASP → validator, auto-tests, app manifest, CLI, style packs, multi-agent AI |
| `DATA_INTEGRATION.md` | DataNode/ActionNode/SubscriptionNode, TanStack Query compile target, ContentSlot |
| `FORMS_SEO_I18N.md` | FormNode, PageMetadata, heading enforcement, static i18n zero-runtime |
| `ADOPTION_MIGRATION.md` | Wedge = non-technical creators, design token import, business model |
| `CONVERSATIONAL_DESIGN.md` | Anti-vibe-coding, one question at a time, discovery agent, feature completeness |
| `VOICE_AND_AI_INTEGRATION.md` | Voice input, AI-agnostic platform, MCP server, pluggable providers |
| `ANIMATION_ASSETS_DEPLOY.md` | CSS/WAAPI/rAF tiers, compile-time springs, Gatsby-model images, adapter deployment |
| `MEMORY_AND_DECISIONS.md` | Brief enforcement, decision log, session persistence, drift detection |

---

*This plan should be read alongside the research docs in `docs/research/` and the project docs (PRD, ROADMAP, CLAUDE.md).*
