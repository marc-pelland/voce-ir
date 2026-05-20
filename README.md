# Voce IR

[![CI](https://github.com/marc-pelland/voce-ir/actions/workflows/ci.yml/badge.svg)](https://github.com/marc-pelland/voce-ir/actions/workflows/ci.yml)
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

### 9 Validation Passes (52 Rules, 17 auto-fixable)

Every IR document is validated before compilation. Every diagnostic carries a stable code, a `hint`, a `docs_url`, and — for 17 codes — a JSON Patch `fix` proposal:

| Pass | Rules | What It Catches |
|------|-------|-----------------|
| Structural | STR001-005 | Missing root, duplicate IDs, empty content |
| References | REF001-009 | Broken node references, missing targets |
| State Machine | STA001-004 | Invalid transitions, unreachable states |
| Accessibility | A11Y001-010 | Labels, headings, contrast (WCAG 2.2 AA), focus order, touch targets, live regions for dynamic content |
| Security | SEC001-009 | Missing CSRF, no auth redirect, HTTP URLs, JSON-LD injection, hardened CSP |
| SEO | SEO001-007 | Missing title, description length, OG completeness |
| Forms | FRM001-009 | Unlabeled fields, missing validation, missing submission |
| Internationalization | I18N001-003 | Empty localized keys, missing defaults |
| Motion | MOT001-005 | No ReducedMotion, excessive duration |

Run `voce skills --json` for the live machine-readable list (see [Agent Contract](#agent-contract-6-envelopes) below).

### Agent Contract (6 Envelopes)

Voce has no human-readable source text, so the **agent contract is the only interface** — and it's a real one. Six contract-versioned, schema-locked JSON envelopes, drift-gated in CI:

| Envelope | Command | Use |
|----------|---------|-----|
| `skills` | `voce skills --json` | What this build can do: passes, codes, node types, targets, CLI |
| `graph` | `voce graph <file> --json` | IR semantic graph: composition, typed reference edges with resolved/dangling status, state-machine reachability |
| `doctor` | `voce doctor --json` | Toolchain + `.voce/` project health with stable check IDs |
| `fix-plan` | `voce fix <file> --plan` | Convergent multi-step repair plan (apply→re-validate→repeat) with explicit `converges` + `residual_codes` |
| `perf-report` | `voce compile <file> --perf-report <out>` | Per-phase compile timing (S71) |
| `conformance` | `voce conformance run --target <id> --json` | Cross-target semantic equivalence report at Core / Standard / Full level |

Schemas live under [`docs/schema/contract/v1/`](docs/schema/contract/v1/). Versioning policy: additive = minor, breaking = major; codes / target ids / check ids are never reassigned.

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
git clone https://github.com/marc-pelland/voce-ir.git
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
# Validate against all 52 quality rules
voce validate hello.voce.json

# Compile to a single HTML file (output: dist/hello.html)
voce compile hello.voce.json

# Open in browser
voce preview hello.voce.json

# Deploy
voce deploy hello.voce.json --adapter static
voce deploy hello.voce.json --adapter cloudflare --dry-run
```

## Conversational Interfaces

Voce ships two surfaces for working with the IR in plain English — one standalone, one embedded.

### Standalone REPL: `voce-chat`

A self-contained conversational terminal — sister to running `claude` or `gemini`. Tool-use loop driven by the Anthropic SDK, persistent `.voce/` memory (project brief, decision log, drift detection), 18 slash commands, multi-line input, prompt caching, ~78 ms cold start.

```bash
npm install -g @voce-ir/cli-chat
voce-chat
```

### Embed via MCP: `@voce-ir/mcp-server`

Plug Voce into any [Model Context Protocol](https://modelcontextprotocol.io) client — Claude Code, Cursor, Continue, your own agent — and the same 22 tools (validate, compile, inspect, generate, brief, decisions, drift, generation workflow, skills, graph, doctor) plus 4 resources become first-class for the assistant.

```bash
npm install -g @voce-ir/mcp-server
```

```jsonc
// Claude Code config
{ "mcpServers": { "voce-ir": { "command": "voce-mcp-server" } } }
```

Both share the same store, the same workflow gates, and the same agent contract — pick the one that fits your flow.

## CLI Reference

```
# IR pipeline
voce validate <file>     Validate IR (9 passes, 52 rules)
                         [--format terminal|json] [--verbose-passes]
                         [--list-passes] [--list-codes] [--warn-as-error]
voce fix <file>          Apply auto-fix proposals (17 codes have JSON Patch fixes)
                         [--apply]                              Write changes (default: preview)
                         [--confidence safe|suggested|risky]    Threshold (default: safe)
                         [--until-clean]                        Convergent loop: apply → re-validate → repeat
                         [--plan]                               Emit the multi-step plan as JSON contract envelope
voce compile <file>      Compile to HTML [--minify] [--skip-fonts] [--no-cache] [--debug] [--perf-report <out>]
voce deploy <file>       Deploy [--adapter static|vercel|cloudflare|netlify] [--dry-run]
voce inspect <file>      IR summary (node counts, types, features)
voce preview <file>      Compile and open in browser
voce report <file>       Quality report (a11y, security, performance)
voce manifest <file>     Application manifest
voce json2bin <file>     JSON to FlatBuffers binary
voce bin2json <file>     FlatBuffers binary to JSON

# Agent contract — see "Agent Contract" above
voce skills              Reflected capability manifest [--json]
voce graph <file>        IR semantic graph: composition + typed refs + state-machine reachability [--json]
voce doctor              Toolchain + .voce/ project health [--json] [--strict] [--ir-set] [--cwd <path>]
voce conformance run     Cross-target conformance against the fixture corpus
                         --target <id> [--level core|standard|full] [--corpus <dir>] [--json]
```

Project config: drop a `.voce/validator.toml` next to your IR to escalate rule severity per project — e.g. `[severity] SEO007 = "error"` to require og:image.

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
│   ├── validator/           Validator + voce CLI binary (9 passes, 52 rules)
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

**15 Rust crates** | **4 TypeScript packages** | **391 tests** (321 Rust + 70 vitest) | **6 contract envelopes** | **30 documentation pages**

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

# Run all tests (391 total: 321 Rust + 70 vitest)
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

### For agent and tool authors

- **[Agent Contract](docs/schema/contract/v1/README.md)** -- The six contract envelopes, versioning policy, drift-gate machinery
- **[Compatibility Matrix](docs/compatibility-matrix.md)** -- Per-target semantic-parity classification (✓ / ◐ / ✗ / ⚠)
- **[Accessibility](docs/accessibility/OVERVIEW.md)** -- Compile-time WCAG 2.2 AA model, every A11Y rule mapped to a WCAG SC, manual-testing checklist, machine-checked evidence

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
- [x] S61: Live pipeline hero (self-demonstrating site)
- [x] S64: Compiler rich defaults (typography, lists, tables, theme palette)
- [x] S65: MCP server polish (19 tools, `.voce/` memory, generate workflow)
- [x] S66: Standalone conversational REPL
- [x] S67: Validator diagnostic quality (hints, JSON Patch fixes, `voce fix`)
- [x] S69: Test coverage uplift (proptest, coverage gate, mutation pilot)
- [x] S70: Security hardening (hardened CSP, prompt-injection defense, threat model)
- [x] S71: Perf budgets (WASM 522 KB, Lighthouse CI floor)
- [x] S72: Schema completeness audit (FormFieldStyle, FormLayout)
- [~] S82: Accessibility deep dive (WCAG 2.2 AA) — in progress
- [ ] S68: Cross-target parity matrix
- [ ] S74: Dev experience (`voce dev`, IDE plugins)
- [ ] S79: Agent capability surface (`voce skills`/`doctor`/`graph`)
- [ ] S91: Conformance specification & certification suite
- [ ] S60: Community launch and v1.1.0

See [MASTER_PLAN.md](docs/plans/MASTER_PLAN.md) for the full 60-sprint breakdown.

## License

[Apache-2.0](LICENSE)

## Links

- **Website:** [voce-ir.xyz](https://voce-ir.xyz)
- **Docs:** [voce-ir.xyz/docs](https://voce-ir.xyz/docs)
- **Playground:** [voce-ir.xyz/playground](https://voce-ir.xyz/playground)
- **Issues:** [github.com/marc-pelland/voce-ir/issues](https://github.com/marc-pelland/voce-ir/issues)

---

Created by [Marc Pelland](https://github.com/marc-pelland).
