# Voce IR

[![CI](https://github.com/marcpelland/voce-ir/actions/workflows/ci.yml/badge.svg)](https://github.com/marcpelland/voce-ir/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**The code is gone. The experience remains.**

Voce IR is an open-source, AI-native intermediate representation for user interfaces. Instead of AI writing framework code that humans then maintain, Voce IR is a binary, typed, machine-optimized format that AI generates directly and a compiler consumes -- like [SPIR-V](https://www.khronos.org/spir/) for UI instead of shaders.

Named from *sotto voce* -- quiet input, extraordinary output.

---

## Why Voce IR?

Every AI coding tool today (v0, bolt.new, Cursor, Copilot) uses AI to write code faster in systems designed for humans. Voce IR asks a different question:

> If AI is the only author and the end-user experience is the only output that matters, what does the optimal system look like?

The answer: remove the human-readable layer entirely.

```
Natural Language -> [AI Bridge] -> Binary IR -> [Validator] -> [Compiler] -> Output
```

No React. No CSS cascade. No dependency tree. Just a typed IR that compiles to optimized output across 7 targets.

### Comparison

| | Traditional Stack | Voce IR |
|--|-------------------|---------|
| **Layout** | CSS cascade, specificity wars | Constraint graph, compile-time resolved |
| **State** | Hooks, closures, re-renders | Typed finite state machines |
| **Accessibility** | Opt-in lint warnings | Compile error if missing |
| **Security** | Manual CSP, CSRF, HTTPS config | Automatic -- compiler defaults |
| **Output size** | ~200KB+ (framework runtime) | <10KB (zero runtime) |
| **TTI** | 200-400ms | <50ms |
| **Supply chain** | npm install = trust chain | Zero runtime dependencies |

## Features

### 7 Compile Targets

| Target | Output | Use Case |
|--------|--------|----------|
| **DOM** | Single-file HTML, inline CSS, surgical JS | Websites, landing pages |
| **WebGPU** | WGSL shaders, PBR materials, particle systems | 3D product viewers, data viz |
| **WASM** | State machines and compute as WebAssembly | Performance-critical logic |
| **Hybrid** | DOM + WebGPU + WASM, device-aware | Complex apps needing all three |
| **iOS** | SwiftUI views with VoiceOver | Native iOS apps |
| **Android** | Jetpack Compose with TalkBack | Native Android apps |
| **Email** | Table layouts, inline CSS, Outlook compat | Marketing emails |

### 9 Validation Passes (46 Rules)

Every IR document is validated before compilation:

| Pass | Rules | What It Catches |
|------|-------|-----------------|
| Structural | STR001-005 | Missing root, duplicate IDs, empty content |
| References | REF001-009 | Broken node references, missing targets |
| State Machine | STA001-004 | Invalid transitions, unreachable states |
| Accessibility | A11Y001-005 | Missing labels, heading skips, no keyboard equiv |
| Security | SEC001-004 | Missing CSRF, no auth redirect, HTTP URLs |
| SEO | SEO001-007 | Missing title, description length, OG completeness |
| Forms | FRM001-009 | Unlabeled fields, missing validation |
| Internationalization | I18N001-003 | Empty localized keys, missing defaults |
| Motion | MOT001-005 | No ReducedMotion, excessive duration |

### 27 Node Types

Layout, state machines, animations, gestures, navigation, routing, accessibility semantics, theming, data binding, forms, SEO metadata, internationalization -- all as first-class IR primitives defined across 12 FlatBuffers schema files.

### Anti-Vibe-Coding

This is not "generate fast and iterate." The AI bridge implements a conversational design system:

- Asks one question at a time to build context
- Maintains a readiness score (0-100) before generating
- Pushes back on anti-patterns
- Enforces the project brief across sessions
- No TODOs, no half-implementations, no placeholders

### Production Pipeline

```
voce validate page.voce.json    # 9 validation passes
voce compile page.voce.json     # Compile to HTML (7.6KB output)
voce deploy page.voce.json      # Deploy to Vercel/Cloudflare/Netlify
```

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.85 or later

### Install

```bash
cargo install voce-validator
```

Or build from source:

```bash
git clone https://github.com/marcpelland/voce-ir.git
cd voce-ir
cargo build --release --workspace
```

### Your First IR File

Create `hello.voce.json`:

```json
{
  "schema_version_major": 1,
  "schema_version_minor": 0,
  "root": {
    "node_id": "root",
    "viewport_width": { "value": 1024, "unit": "Px" },
    "metadata": {
      "title": "Hello World",
      "description": "My first Voce IR page."
    },
    "children": [
      {
        "value_type": "TextNode",
        "value": {
          "node_id": "greeting",
          "content": "Hello, World!",
          "heading_level": 1,
          "font_size": { "value": 48, "unit": "Px" },
          "font_weight": "Bold"
        }
      }
    ]
  }
}
```

### Validate, Compile, Preview

```bash
# Validate against all 46 quality rules
voce validate hello.voce.json

# Compile to a single HTML file (output: dist/hello.html)
voce compile hello.voce.json

# Open in browser
voce preview hello.voce.json

# Deploy
voce deploy hello.voce.json --adapter static
voce deploy hello.voce.json --adapter cloudflare --dry-run
```

## CLI Reference

```
voce validate <file>     Validate IR (9 passes, 46 rules)
voce compile <file>      Compile to HTML [--minify] [--skip-fonts] [--no-cache] [--debug]
voce deploy <file>       Deploy [--adapter static|vercel|cloudflare|netlify] [--dry-run]
voce inspect <file>      IR summary (node counts, types, features)
voce preview <file>      Compile and open in browser
voce report <file>       Quality report (a11y, security, performance)
voce manifest <file>     Application manifest
voce json2bin <file>     JSON to FlatBuffers binary
voce bin2json <file>     FlatBuffers binary to JSON
```

Global flags: `--verbose`, `--json-errors`

## Architecture

```
                        ┌─────────────────────────┐
                        │      FlatBuffers IR      │
                        │   (12 .fbs, 27 types)    │
                        └────────────┬────────────┘
                                     │
┌──────────────┐     ┌──────────┐    │    ┌───────────┐     ┌──────────┐
│  Natural     │     │  JSON    │    │    │           │     │ DOM      │
│  Language    │────>│  IR      │────┼───>│ Validator │────>│ WebGPU   │
│  (AI Bridge) │     │ (.voce)  │    │    │ (9 passes)│     │ WASM     │
└──────────────┘     └──────────┘    │    └───────────┘     │ iOS      │
                                     │                      │ Android  │
                                     │                      │ Email    │
                                     │                      │ Hybrid   │
                                     │                      └─────┬────┘
                                     │                            │
                                     │     ┌──────────────────────┘
                                     │     │
                                     │  ┌──┴───────┐
                                     │  │ Deploy   │
                                     │  │ Adapters │
                                     │  └──────────┘
                                     │  Vercel, Cloudflare,
                                     │  Netlify, Static
                                     │
                              ┌──────┴──────┐
                              │  Inspector  │
                              │  Playground │
                              └─────────────┘
```

The IR is defined as [FlatBuffers](https://flatbuffers.dev/) tables. The JSON canonical format (`.voce.json`) is the primary development format; binary `.voce` files use FlatBuffers zero-copy deserialization for production.

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full technical design.

## Three Pillars

Every decision in Voce IR is anchored to three non-negotiable principles:

1. **Stability** -- Security is a compile error, not configuration. CSRF required on mutations, HTTPS enforced, auth routes need redirects. Zero runtime dependencies = zero supply chain risk.

2. **Experience** -- Zero framework overhead. Spring physics solved at compile time via ODE solver to CSS `linear()`. Every byte in the output serves the user.

3. **Accessibility** -- Missing `SemanticNode` is a validation error, not a warning. Keyboard equivalents required on every gesture handler. Heading hierarchy enforced. `ReducedMotion` mandatory on all animations.

## Repository Structure

```
voce-ir/
├── packages/
│   ├── schema/              FlatBuffers IR schema (12 .fbs files, 27 node types)
│   ├── validator/           Validator + voce CLI binary (9 passes, 46 rules)
│   ├── compiler-dom/        DOM compiler (HTML/CSS/JS, image pipeline, font pipeline)
│   ├── compiler-webgpu/     WebGPU compiler (WGSL, PBR, particles)
│   ├── compiler-wasm/       WASM compiler (state machines -> WAT)
│   ├── compiler-hybrid/     Hybrid compiler (DOM + WebGPU + WASM)
│   ├── compiler-ios/        iOS compiler (SwiftUI)
│   ├── compiler-android/    Android compiler (Jetpack Compose)
│   ├── compiler-email/      Email compiler (table layouts, inline CSS)
│   ├── adapter-core/        Deployment adapter trait and types
│   ├── adapter-static/      Static file deployment
│   ├── adapter-vercel/      Vercel Build Output API v3
│   ├── adapter-cloudflare/  Cloudflare Pages + Workers
│   ├── adapter-netlify/     Netlify + Functions
│   ├── playground-wasm/     WASM bindings for browser playground
│   ├── playground/          Browser-based "try it" playground (Vite + TS)
│   ├── ai-bridge/           Multi-agent AI generation (TypeScript)
│   ├── mcp-server/          MCP server for Claude/AI tool integration
│   ├── sdk/                 Programmatic TypeScript SDK
│   └── inspector/           Visual debugging tools (TypeScript)
├── tests/
│   ├── schema/              Valid + invalid IR fixtures (12 invalid, 1 valid)
│   └── fixtures/            Per-node-type test IR files (12 fixtures)
├── examples/
│   ├── landing-page/        Reference 37-node landing page
│   ├── production/          Production voce-ir.xyz site (IR + compiled output)
│   └── intents/             Natural language -> IR training pairs
├── docs/
│   ├── site/                mdBook documentation site (30 pages)
│   ├── plans/               Sprint plans (S01-S60)
│   ├── research/            9 deep research documents
│   └── ARCHITECTURE.md      Technical architecture & key decisions
└── scripts/
    └── regenerate-schema.sh FlatBuffers codegen
```

**15 Rust crates** | **4 TypeScript packages** | **172 tests** | **30 documentation pages**

## Performance

Measured with [criterion](https://github.com/bheisler/criterion.rs) on Apple Silicon:

| Benchmark | Time |
|-----------|------|
| Compile reference landing page (37 nodes) | 209 us |
| Compile production page (30+ nodes) | 330 us |
| Compile minimal TextNode | 4.4 us |
| Validate reference landing page | <100 us |

Production landing page output: **7.6KB** (unminified), **7.3KB** (minified).

## Development

```bash
# Build everything
cargo build --workspace

# Run all tests (172 tests)
cargo test --workspace

# Lint (zero warnings policy)
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --check

# Benchmarks
cargo bench -p voce-compiler-dom --bench compile_bench

# Build WASM playground
cd packages/playground-wasm && wasm-pack build --target web

# Build documentation site
cd docs/site && mdbook build
```

## Documentation

- **[Documentation Site](https://voce-ir.xyz/docs)** -- Getting started, CLI reference, full schema reference, architecture guide
- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** -- Technical architecture, crate dependency graph, key decisions
- **[Schema Reference](docs/site/src/schema/overview.md)** -- Every node type with field tables and JSON examples
- **[Playground](https://voce-ir.xyz/playground)** -- Try Voce IR in the browser (WASM-powered)

Build docs locally:

```bash
cd docs/site && mdbook serve
```

## Contributing

Contributions are welcome! Whether it's a bug fix, new compile target, validation rule, or documentation improvement.

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, code conventions, and the PR process.

### Areas for Contribution

- **New validation rules** -- add passes in `packages/validator/src/passes/`
- **Compiler improvements** -- better HTML output, new CSS features
- **New compile targets** -- Flutter, React Native, etc.
- **Schema extensions** -- new node types for new UI patterns
- **Test fixtures** -- more IR examples covering edge cases
- **Documentation** -- tutorials, guides, API reference improvements
- **Playground** -- CodeMirror/Monaco editor integration, more examples

## Roadmap

Voce IR is at v1.0.0 with all core functionality complete. The current focus (Phase 7) is production readiness:

- [x] S51: Real image processing pipeline (WebP, JPEG, BlurHash)
- [x] S52: Deployment adapters (Vercel, Cloudflare, Netlify, static)
- [x] S53: Browser playground (WASM-powered)
- [x] S54: Integration test suite (172 tests)
- [x] S55: Documentation site (mdBook, 30 pages)
- [x] S56: Font subsetting and optimization
- [x] S57: Production error handling and structured errors
- [x] S58: Production demo site (voce-ir.xyz)
- [x] S59: Performance optimization and benchmarks
- [ ] S60: Community launch and v1.1.0

See [MASTER_PLAN.md](docs/plans/MASTER_PLAN.md) for the full 60-sprint breakdown.

## License

[Apache-2.0](LICENSE)

## Links

- **Website:** [voce-ir.xyz](https://voce-ir.xyz)
- **Docs:** [voce-ir.xyz/docs](https://voce-ir.xyz/docs)
- **Playground:** [voce-ir.xyz/playground](https://voce-ir.xyz/playground)
- **Issues:** [github.com/marcpelland/voce-ir/issues](https://github.com/marcpelland/voce-ir/issues)

---

Created by [Marc Pelland](https://github.com/marcpelland).
