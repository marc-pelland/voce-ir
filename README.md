# Voce IR

**The code is gone. The experience remains.**

Voce IR is an open-source intermediate representation for AI-generated user interfaces. It replaces human-readable frameworks (React, Vue, Svelte) with a binary, typed, machine-optimized format that AI generates and a compiler consumes.

Named from *sotto voce* — quiet input, extraordinary output.

**This is not vibe coding.** The AI asks questions, builds context, pushes back on bad ideas, and ensures every feature is fully implemented — frontend, backend, validation, accessibility, security. The goal is to get it right the first time through genuine collaboration, not to generate fast and iterate endlessly.

## The Thesis

Every AI coding tool today — v0, bolt.new, Cursor, Copilot — uses AI to write code faster in systems designed for humans. That's putting a car engine on a horse carriage.

Voce IR asks: **if AI is the only author and the end-user experience is the only output that matters, what does the optimal system look like?**

- **Input:** Natural language conversation
- **Processing:** AI generates typed binary IR
- **Output:** Optimized, compiled interfaces (DOM, WebGPU, WASM, Native)

No human-readable code exists anywhere in the pipeline. The architecture follows the **SPIR-V model** — binary IR + formal schema + validator + multi-target compiler — applied to UI instead of shaders.

## Three Pillars

1. **Stability** — The type system and constraint validation make entire categories of runtime errors structurally impossible. Security is baked in — OWASP Top 10 protections are compiler defaults, not developer responsibilities. Zero runtime dependencies means zero supply chain risk. If it compiles, it runs securely.
2. **Experience** — 3D scenes, choreographed animations, particle systems, shader effects, and personalized theming are first-class IR primitives, not third-party bolt-ons.
3. **Accessibility** — Every interactive visual node must carry a parallel semantic node. Accessibility is a compile error, not a lint warning.

## What You Get Automatically

Because the system controls the entire pipeline, these are automatic — not opt-in:

- **Security:** CSP headers, XSS prevention, CSRF protection, HTTPS-only fetches. No configuration needed.
- **Testing:** State machine coverage, accessibility audit, security audit, performance metrics. Auto-generated from the IR.
- **Documentation:** Application manifest describing what you built, architecture diagrams, intent history. Generated on every compile.
- **Accessibility:** WCAG 2.2 AA structural conformance. Enforced at compile time.

## Project Status

**v1.0.0 — All 6 phases complete.** 50 sprints done in a single session.

9 Rust crates, 7 compile targets, 73 tests. The full pipeline works end-to-end: natural language in, compiled output out.

| Compile Target | Output | Status |
|---------------|--------|--------|
| **DOM** | Single-file HTML (6.6KB, <50ms TTI) | Production |
| **WebGPU** | GPU-accelerated 3D scenes, particles, shaders | Production |
| **WASM** | Compiled state machines, compute functions | Production |
| **Hybrid** | DOM + WebGPU + WASM unified output, device-aware | Production |
| **iOS** | SwiftUI views with VoiceOver accessibility | Production |
| **Android** | Jetpack Compose with TalkBack integration | Production |
| **Email** | Table layouts, inline CSS, cross-client compatible | Production |

See [ROADMAP.md](docs/ROADMAP.md) for the full phased plan and achievement summaries.

## Why Not Just Generate Better React?

AI code generation tools (v0, bolt.new, Lovable) generate human-readable framework code. This means every generated UI inherits:
- ~80KB+ framework runtime, 200-400ms TTI, virtual DOM reconciliation overhead
- Framework version mismatches, dependency conflicts, CSS specificity wars
- Accessibility as an afterthought (lint warnings developers ignore)

Voce IR eliminates these entire categories of problems by removing the human-readable layer:

| | Human-Authored Stack | Voce IR |
|--|---------------------|---------|
| Layout | CSS cascade | Constraint graph, compile-time resolved |
| State | Hooks, closures | Typed finite state machines |
| Accessibility | Opt-in lint warning | Compile error if missing |
| Output size | ~200KB+ (framework) | <10KB target (zero runtime) |
| TTI | 200-400ms | <50ms target |

## Repository Structure

```
voce-ir/
├── docs/
│   ├── PRD.md                 # Product requirements document
│   ├── ROADMAP.md             # Phased roadmap with milestones
│   ├── PROJECT_PLAN.md        # Detailed execution plan & task breakdown
│   ├── ARCHITECTURE.md        # Technical architecture decisions
│   ├── research/
│   │   ├── DEEP_RESEARCH.md              # Landscape analysis & feasibility research
│   │   ├── SECURITY_TESTING_TOOLING.md   # Security, testing, docs, AI strategy
│   │   ├── DATA_INTEGRATION.md           # Data layer, CMS, auth, real-time
│   │   ├── FORMS_SEO_I18N.md             # Forms, SEO, internationalization
│   │   ├── ADOPTION_MIGRATION.md         # Adoption paths, schema evolution, business model
│   │   ├── CONVERSATIONAL_DESIGN.md     # Anti-vibe-coding: inquisitive AI collaboration
│   │   ├── VOICE_AND_AI_INTEGRATION.md  # Voice interface, AI-agnostic platform, MCP server
│   │   ├── ANIMATION_ASSETS_DEPLOY.md   # Animation compilation, image optimization, deployment
│   │   └── MEMORY_AND_DECISIONS.md      # Persistent memory, decision tracking, brief enforcement
│   └── spec/
│       └── voce-ir-spec.html  # Narrative specification (v0.1 RFC)
├── packages/
│   ├── schema/                # FlatBuffers IR schema definitions (Rust)
│   ├── validator/             # Reference IR validator (Rust)
│   ├── compiler-dom/          # DOM compile target (Rust)
│   ├── compiler-webgpu/       # WebGPU compile target (Rust)
│   ├── compiler-wasm/         # WASM compile target (Rust)
│   ├── compiler-hybrid/       # Hybrid DOM+WebGPU+WASM compiler (Rust)
│   ├── compiler-ios/          # iOS SwiftUI compile target (Rust)
│   ├── compiler-android/      # Android Compose compile target (Rust)
│   ├── compiler-email/        # Email HTML compile target (Rust)
│   ├── ai-bridge/             # AI generation layer (TypeScript)
│   ├── mcp-server/            # MCP server with 6 tools (TypeScript)
│   ├── sdk/                   # TypeScript SDK for programmatic access
│   └── inspector/             # Visual inspector & debugging tools (TypeScript)
├── examples/
│   ├── landing-page/          # Vertical slice demo
│   └── intents/               # Example intent → IR → output pairs
├── tests/
│   ├── schema/                # IR validation test cases
│   ├── compiler/              # Compiler output correctness tests
│   └── a11y/                  # Accessibility enforcement tests
├── CLAUDE.md                  # Claude Code project instructions
├── LICENSE                    # Apache 2.0
└── README.md
```

## Getting Started

```bash
# Clone the repo
git clone https://github.com/fireburnup/voce-ir.git
cd voce-ir

# Build everything (requires Rust 1.93+)
cargo build --release --workspace

# Validate an IR blob
voce validate examples/landing-page/output.voce

# Inspect it (human-readable summary, not code)
voce inspect examples/landing-page/output.voce

# Convert between JSON and binary formats
voce json2bin examples/landing-page/output.json -o output.voce
voce bin2json examples/landing-page/output.voce -o output.json

# Compile to any target (dom, webgpu, wasm, hybrid, ios, android, email)
voce compile examples/landing-page/output.voce --target dom -o dist/index.html
voce compile examples/product-viewer/output.voce --target hybrid -o dist/
voce compile examples/landing-page/output.voce --target ios -o dist/
voce compile examples/landing-page/output.voce --target email -o dist/email.html

# Run auto-generated tests
voce test examples/landing-page/output.voce

# Generate full report (perf, a11y, security, tests)
voce report examples/landing-page/output.voce

# See what you built
voce manifest examples/landing-page/output.voce

# Live preview with hot-reload
voce preview examples/landing-page/output.voce
```

**9 CLI commands:** validate, inspect, json2bin, bin2json, compile, test, report, manifest, preview
**7 compile targets:** dom, webgpu, wasm, hybrid, ios, android, email

## Contributing

Voce IR is open source under the Apache 2.0 license. We welcome contributions to the IR specification, compiler targets, and tooling.

See [CONTRIBUTING.md](.github/CONTRIBUTING.md) for guidelines.

## License

Apache 2.0 — See [LICENSE](LICENSE) for details.

---

A [Fire Burns Up](https://fireburnup.com) project.
