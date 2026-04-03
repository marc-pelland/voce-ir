# Voce IR — Product Requirements Document

**Version:** 1.0.0
**Status:** Stable
**Owner:** Marc Pelland
**Last updated:** 2026-04-02

---

## 1. Vision

Voce IR is an open-source intermediate representation and compiler toolchain that enables AI systems to generate high-performance, accessible, visually rich user interfaces without producing human-readable code.

The developer's interface is the conversation. The code is an internal artifact. The user's experience is the only output that matters.

**The conversation is not vibe coding.** The AI is an inquisitive collaborator — it asks questions, builds context, pushes back on anti-patterns, and ensures features are fully implemented before generating. The goal is to get the output 90% right on the first pass by investing in understanding upfront, not to generate fast and iterate endlessly.

## 2. Problem Statement

### 2.1 The Current State

Modern UI frameworks (React, Next.js, Vue, Svelte) were designed for human developers. Every abstraction exists to manage human cognitive load:

- **Virtual DOM** exists because humans can't manually track DOM mutations
- **CSS cascade** exists because humans need inheritance to avoid repetition
- **Component models** exist because humans need encapsulation for comprehension
- **Hook rules** exist because React's reconciler needs ordering guarantees that humans must manually maintain

These abstractions carry runtime cost: ~80KB+ framework runtime, CSS parsing, hydration passes, virtual DOM reconciliation, and synthetic event systems. For a typical Next.js page, this adds 200-400ms to time-to-interactive and 200KB+ to the initial payload.

Research indicates that ~40-60% of this overhead is genuinely "human readability tax" — abstractions that serve only human cognition. The remaining 40-60% solves real browser/platform complexity (event handling, DOM mutation batching, layout thrashing prevention). Voce IR addresses the first category by eliminating it, and the second by solving these problems through compilation rather than runtime frameworks.

### 2.2 The Shift

AI-assisted code generation (Claude, GPT, Copilot) is rapidly becoming the primary authorship mechanism for UI code. But these systems generate React/Vue/HTML because that's what browsers understand — not because it's optimal. The AI translates intent → human-readable framework code → browser rendering pipeline, paying the tax of human readability at every layer.

Every AI UI generation tool on the market — v0, bolt.new, Cursor, Copilot, Lovable, Replit Agent — uses AI to write code faster in human-designed systems. This is using a car engine to pull a horse carriage. The question isn't "how does AI write better React?" — it's "what does the system look like when AI is the only author?"

### 2.3 The Opportunity

Remove the human-readable layer entirely. Define an intermediate representation optimized for:
- **Machine generation** (typed, unambiguous, no syntax to get wrong)
- **Machine compilation** (binary, pre-resolved, whole-program optimizable)
- **Human experience** (expressive enough for 3D, animation, personalization, accessibility)

## 3. Target Users

### 3.1 Primary: AI-First Builders

Developers and founders who build primarily through conversation with AI. They describe what they want, review the output, and iterate through dialogue. They do not want to read, write, or debug framework code. They want the fastest path from idea to deployed experience.

### 3.2 Secondary: Creative Technologists / Experiential Studios

Teams building immersive, interactive experiences (installations, kiosks, branded digital products) who need the expressiveness of game engines but the deployment flexibility of the web. Currently forced to choose between web frameworks (limited expressiveness) and game engines (limited web deployment).

### 3.3 Tertiary: Open Source Contributors / Tooling Builders

Engineers interested in compilers, language design, and AI tooling who want to contribute to the IR specification, build new compile targets, or create tooling around the format.

## 4. Core Capabilities

### 4.1 IR Specification (The Format)

A formal, versioned binary format defining the complete set of primitives needed to describe user interfaces:

| Domain | Primitives | Purpose |
|--------|-----------|---------|
| Scene & Layout | ViewRoot, Container, Surface, TextNode, MediaNode | Spatial composition without CSS |
| 3D & Graphics | Scene3D, MeshNode, ShaderNode, ParticleSystem | First-class 3D and visual effects |
| State & Logic | StateMachine, DataNode, ComputeNode, EffectNode, ContextNode | Explicit, typed application state |
| Data & Backend | ActionNode, SubscriptionNode, AuthContextNode, ContentSlot | Declarative data layer (mutations, real-time, auth, CMS) |
| Content | RichTextNode, ContentSource, ContentModel | Structured CMS content with compile-time injection |
| Forms | FormNode, FormField, ValidationRule, FormSubmission | Declarative forms with compiler-generated validation |
| SEO | PageMetadata, OpenGraphData, StructuredData | Compile-time SEO enforcement |
| i18n | LocalizedString, MessageCatalog, FormatOptions | Static or runtime internationalization |
| Motion | Transition, Sequence, GestureHandler, ScrollBinding, PhysicsBody | Choreographed animation and interaction |
| Navigation | RouteMap, RouteTransition | Application routing as state machines |
| Accessibility | SemanticNode, LiveRegion, FocusTrap, ReducedMotion | Structurally mandatory a11y |
| Theming | ThemeNode, PersonalizationSlot, ResponsiveRule | Design tokens and adaptive layout |

**Key constraint:** The IR is not designed to be human-readable. It is a binary, typed, machine-to-machine format.

### 4.2 Reference Validator

A program that accepts or rejects IR blobs with typed error messages. Enforces:

- Type correctness (every field matches its declared type)
- Structural completeness (required nodes present, references resolve)
- Pillar constraints:
  - **Stability:** No unreachable states, no unhandled errors, no unresolved data dependencies
  - **Experience:** Valid animation definitions, valid 3D scene graphs
  - **Accessibility:** Every interactive node has a SemanticNode, every animation has a ReducedMotion fallback, every gesture has a keyboard equivalent
  - **Security:** DataNodes declare allowed origins, protected routes have auth guards, mutation EffectNodes have idempotency annotations, no XSS-possible patterns (see `docs/research/SECURITY_TESTING_TOOLING.md`)

### 4.3 Compiler (Multi-Target)

Accepts validated IR + a device profile, emits optimized output:

| Target | Output | Use Case |
|--------|--------|----------|
| DOM | Minimal HTML + inline JS, surgical mutations, no framework runtime | Web deployment, SEO, progressive enhancement |
| WebGPU | GPU draw calls, shader programs, render pipeline | 3D scenes, experiential, high-fidelity visuals |
| WASM | Near-native binary for complex compute | Performance-critical state logic, physics |
| Native | Platform-specific UI (SwiftUI, Compose) | Mobile apps, desktop apps |

**MVP target: DOM only.** Other targets are future phases.

### 4.4 AI Generation Bridge

The layer that connects natural language intent to valid IR output:

- **Near-term:** Multi-agent architecture with mandatory discovery phase — Discovery Agent (Opus) builds project/user profile and blocks generation until sufficient context exists, Design Agent (Opus) applies UX best practices and pushes back on anti-patterns, Architecture Agent (Opus) ensures full-stack feature completeness (no TODOs), Generator (Sonnet) emits valid IR JSON via structured output, Repair Agent (Haiku/Sonnet) fixes validation errors. See `docs/research/CONVERSATIONAL_DESIGN.md` for full philosophy
- **Mid-term:** Fine-tuned model trained on intent → IR corpus (bootstrapped from style packs and synthetic data)
- **Long-term:** Native IR generation as a model capability (partnership with AI labs)

### 4.5 Visual Inspector

Debugging without code. A runtime tool that exposes:

- Scene graph overlay (click any element, see IR node properties)
- State machine visualizer (current state, available transitions, history)
- Data flow monitor (fetch status, cache state, dependency graph)
- Animation timeline (scrubable, frame-by-frame)
- Accessibility auditor (live a11y tree, focus order, contrast)
- Performance profiler (frame timing, GPU utilization, memory)

**This is a later phase** but architecturally important to plan for from the start. Research indicates the inspector is load-bearing for adoption — consider pulling it to Phase 2-3.

### 4.6 Application Documentation System

Because there is no source code, the system must generate understanding automatically:

- **Application Manifest** — auto-generated human-readable document describing pages, state machines, data sources, accessibility features, security configuration, and design decisions. Derived from IR analysis + intent history
- **Intent History Log** — every conversation session that modifies the IR is logged with: intent, changes made, validation results, and notes. Provides AI context across sessions, collaboration context for teams, and audit trail for compliance
- **Architecture Diagrams** — auto-generated from IR: page map, state machine diagrams, data flow, accessibility tree

### 4.7 Testing Framework

Testing in Voce IR verifies the contract between intent and output:

- **State machine testing** — exhaustive reachability, deadlock-freedom, transition coverage. Auto-generated from IR
- **Compiled output testing** — structural (correct elements), behavioral (correct state transitions), visual regression, accessibility (axe-core), performance (TTI, size), security (CSP, XSS resistance)
- **Intent-level testing** (Phase 3+) — golden intent tests: (intent description, expected output properties) pairs
- **AI-generated test scenarios** (Phase 3+) — the AI generates test scenarios from the intent alongside the IR

### 4.8 Design Pattern Library ("Style Packs")

Curated collections of IR examples that embody specific design languages — the Voce IR equivalent of LoRAs for design patterns:

- Each pack contains: design tokens, common UI patterns as IR snippets, full-page examples
- Used as few-shot examples for the AI bridge (no fine-tuning required)
- Community-contributable and composable ("luxury-ecommerce layout with brutalist colors")
- Also serves as structured training data for future fine-tuning

### 4.9 CLI Toolchain

The `voce` CLI is the primary development interface alongside conversation:

- `voce validate` / `voce compile` / `voce preview` — core pipeline
- `voce inspect` / `voce diff` / `voce manifest` — understanding and debugging
- `voce generate` / `voce patch` — AI bridge interface (Phase 3)
- `voce test` / `voce report` — automated testing and audit reports

See `docs/research/SECURITY_TESTING_TOOLING.md` for full CLI design.

## 5. Non-Goals (Explicit Exclusions)

- **Not a design tool.** Voce IR does not include a visual editor, drag-and-drop builder, or Figma-like interface. The authoring interface is conversation.
- **Not a hosting platform.** Voce IR compiles to static output that deploys anywhere. It does not include deployment, CDN, or serverless infrastructure.
- **Not a replacement for the browser.** The DOM compile target works within existing browsers. Voce IR does not require a custom browser or runtime (though it can target custom runtimes like kiosks).
- **Not a general-purpose programming language.** Voce IR describes interfaces and their behavior. It does not compile general-purpose programs, CLIs, or backend services.

## 6. Technical Requirements

### 6.1 IR Format

- Binary serialization using FlatBuffers (zero-copy deserialization, schema evolution support)
- Strongly typed with structural typing (no `any`, no implicit coercion)
- Versioned schema with forward-compatible field additions
- Maximum IR blob size: 50MB (covers complex multi-route applications)
- Validation time: <100ms for typical single-page IR
- **JSON canonical representation:** A lossless JSON text serialization of the IR that round-trips to/from binary. Serves as: debugging/inspection format, version control diffing, AI generation intermediate format, and escape hatch for manual patching. This is not human-authored code — it is a machine-readable text representation of the IR
- **Explicit node identity:** Every node carries a stable ID for cross-referencing, delta updates, and accessibility tree mapping

### 6.2 Compiler

- Written in Rust for performance and correctness
- DOM target output must produce zero-framework, standards-compliant HTML5 + ES2020 JS
- Compiled output TTI target: <50ms for typical landing page on modern desktop hardware (50-100ms on mid-range mobile is acceptable)
- Compiled output size target: <10KB for a basic landing page (vs ~200KB+ for equivalent Next.js). For multi-route apps, apply per-route budgets rather than a single absolute target
- Incremental compilation: changing one node should not require recompiling the full IR
- Deterministic output: same IR + same device profile = identical output, always

### 6.3 AI Generation

- Must produce valid IR that passes the reference validator with >95% first-attempt structural validity. The validate-repair loop (feed errors back to AI) must achieve >99% validity within 2 cycles
- Generation latency target: <3 seconds for a single view from natural language
- Incremental patching: describe a change, only affected IR nodes are regenerated. Start with subtree replacement; evolve to node-ID-based patch format
- Must not require the human to understand IR, node types, or compiler internals
- AI generates JSON (the canonical text representation), which is then encoded to binary FlatBuffers. Direct binary generation from LLMs is not feasible with current architectures
- For complex UIs, use hierarchical generation (structure first, then populate sections) to avoid output quality degradation beyond ~4,000 tokens of continuous structured output

### 6.4 Accessibility

- Compiled DOM output must meet **structural** WCAG 2.2 AA conformance — the validator enforces ~40-50% of AA criteria at compile time (roles, labels, keyboard, contrast, motion). The remaining criteria depend on content quality and require human/AI judgment
- Screen reader compatibility: NVDA, JAWS, VoiceOver (DOM target inherits browser accessibility; WebGPU target uses hidden DOM mirror approach, validated by Flutter's semantics tree precedent)
- Keyboard navigation must cover all interactive paths
- Reduced motion alternatives must be present for all animations. Functional animations (loading spinners, progress indicators) use `motion_functional: true` to indicate they should be simplified, not removed
- High contrast mode support via ThemeNode variants
- Support explicit opt-outs for valid exceptions: `decorative: true` on MediaNode (allows empty alt text), `presentation: true` on Surface (no SemanticNode required for non-interactive visual elements). Default behavior remains strict — opt-outs must be explicit

### 6.5 Security

- Compiled DOM output must include secure defaults: CSP headers, X-Frame-Options, X-Content-Type-Options, strict referrer policy
- No `eval()`, `innerHTML`, or `document.write()` in compiled output — ever. All DOM content is escaped by the compiler
- DataNode fetch calls use HTTPS by default (HTTP requires explicit opt-in with justification)
- Cookie attributes include `Secure`, `HttpOnly`, `SameSite=Strict` by default
- The security validation pass catches: unguarded protected routes, DataNodes without allowed origin declarations, mutation EffectNodes without idempotency annotations
- Zero runtime dependencies in compiled output eliminates supply chain attack surface entirely

### 6.6 Testing

- Every compilation produces a test report alongside the output (validation, security, a11y, state machine coverage, performance metrics)
- State machine testing is auto-generated from IR: reachability, deadlock-freedom, transition coverage
- Phase 3+: AI-generated test scenarios from intent alongside IR generation

### 6.7 Documentation

- Every compilation generates an Application Manifest (human-readable summary of what was built)
- Intent history log maintained across sessions for AI context and collaboration
- Architecture diagrams (routes, state machines, data flow) auto-generated from IR

## 7. Success Metrics

### 7.1 Phase 1 (Spec & Foundation) Success Criteria

- [ ] FlatBuffers schema compiles and generates bindings for Rust and TypeScript
- [ ] Reference validator correctly accepts valid IR and rejects invalid IR with typed errors
- [ ] 100% of Phase 1 node types have formal schema definitions
- [ ] At least 20 test cases per node type covering valid and invalid states

### 7.2 Phase 2 (DOM Compiler MVP) Success Criteria

- [ ] "Landing page" vertical slice compiles from IR to working HTML
- [ ] Compiled output contains 0 bytes of framework runtime
- [ ] TTI < 50ms on Chrome/latest, measured by Lighthouse
- [ ] Output passes axe-core accessibility audit with 0 violations
- [ ] Compiled output file size < 10KB for the landing page demo

### 7.3 Phase 3 (AI Bridge) Success Criteria

- [ ] Natural language description produces valid IR in >95% of attempts
- [ ] End-to-end latency (intent → compiled preview) < 5 seconds
- [ ] Iterative patching works: describe a change, only affected nodes regenerate
- [ ] Demo recording: conversation → working deployed page in under 2 minutes

### 7.4 Phase 4+ (Growth) Success Criteria

- [ ] At least 2 compile targets functional (DOM + one of WebGPU/WASM/Native)
- [ ] Open source community: 50+ GitHub stars, 5+ external contributors
- [ ] At least 3 real-world projects built entirely with Voce IR

## 8. Constraints & Assumptions

### 8.1 Constraints

- **Solo builder with AI assistance.** Initial development is one person using Claude Code, not a team. Architecture must support incremental, modular development.
- **No custom browser.** Must target existing browsers for the DOM compile path.
- **Open source.** The IR spec, validator, and compiler are open source (Apache 2.0). This constrains business model to tooling/services, not spec licensing.

### 8.2 Assumptions

- AI code generation quality will continue to improve, making the generation layer more reliable over time
- WebGPU browser support will reach >80% by the time the WebGPU compile target is ready
- The open-source community will contribute compile targets and tooling once the IR spec stabilizes
- FlatBuffers provides adequate schema evolution for backward-compatible IR versioning

### 8.3 Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| IR spec is too narrow, can't express real-world UIs | Medium | High | Vertical slices at each phase to validate expressiveness. Escape hatch node type for raw target code. JSON canonical format enables inspection. |
| AI generation layer produces invalid IR frequently | Medium | High | Validate-repair loop: feed errors back to AI. Research shows >99% validity achievable within 2 cycles. Structured output / constrained decoding for structural guarantees. |
| AI generates structurally valid but aesthetically poor UIs | High | Medium | RAG-based generation with curated example library. Bounded property space in IR constrains the design space vs open-ended CSS. Build (intent, IR) pair library starting Phase 1. |
| No adoption — too different from existing workflows | Medium | Medium | Open source + compelling demos. Target AI-first builders who don't have existing framework investments. Visual Inspector is load-bearing for adoption — consider pulling it earlier. |
| Compiler complexity exceeds solo capacity | High | High | Extreme modularity. Each compile target is independent. Community contributions for non-DOM targets. |
| Competing system emerges from a major AI lab | Medium | Medium | Open-source moat. Focus on compile quality and accessibility enforcement, not just generation. The structural advantages (eliminated error categories) are defensible. |
| Binary format creates debugging opacity | Medium | Medium | JSON canonical representation for inspection. Excellent validator error messages with node paths. Visual Inspector for runtime debugging. |
| FlatBuffers immutability limits runtime state | Medium | Medium | Runtime mutable state (signals, animation progress, fetched data) lives outside the FlatBuffer in a reactive layer. FlatBuffer is initial state + structure; runtime state references nodes by ID. |

## 9. Open Questions

### 9.1 Resolved by Research (see `docs/research/DEEP_RESEARCH.md`)

1. **Schema format:** ~~FlatBuffers vs Cap'n Proto vs custom binary?~~ **Resolved: FlatBuffers.** Zero-copy access patterns align with read-heavy UI workloads. Cap'n Proto has weaker TypeScript support (dealbreaker for AI bridge). Protobuf requires runtime library (not zero-copy). FlatBuffers' immutability means runtime mutable state should live in a separate reactive layer.
2. **WebGPU role:** WebGPU is the effects/3D layer, not a DOM replacement. Text rendering in WebGPU requires ~1.5MB of WASM (text engine). The hybrid approach (DOM for text/layout, WebGPU for 3D/effects) is correct. DOM is the primary compile target.
3. **Layout engine:** Use Taffy (pure Rust flexbox/grid engine) for compile-time layout resolution and native compile targets.
4. **Reference architectures:** SPIR-V (binary IR + validator + multi-target compilation pipeline), Compose's slot table (flat binary representation of UI composition state).

### 9.2 Still Open

1. **Escape hatch design:** When the IR can't express something, what happens? Options: raw target code injection (loses portability), extension node type (complex validation), error and human escalation. Additionally: the JSON canonical representation provides a text-based editing escape hatch.
2. **IR file extension:** `.voce`? `.vir`? Needs to be registered as a MIME type eventually.
3. **Versioning strategy:** How does the IR schema evolve? Additive-only fields with deprecation? Major version bumps? Both? FlatBuffers supports forward-compatible field additions — test backward compatibility from day one.
4. **Testing approach:** How do you test an interface with no source code? State machine assertions? Visual regression? AI-generated test scenarios?
5. **Monorepo vs multi-repo:** Current plan is monorepo (this repo). Evaluate if compilation speed or contributor workflow requires splitting.
6. **Delta update format:** How does the IR support incremental changes? Node-ID-based patch operations? Subtree replacement? CRDT-based merging for multi-agent collaboration? FlatBuffers immutability means patches produce new buffers — need efficient diff/merge.
7. **Version control for binary IR:** Binary files don't diff well in git. The JSON canonical representation can serve as the diffable format. Do we commit JSON, binary, or both?

---

*This document is the source of truth for what Voce IR is and what it is not. All implementation decisions should trace back to requirements defined here.*
