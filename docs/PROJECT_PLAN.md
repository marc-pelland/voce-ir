# Voce IR — Project Plan

**Version:** 0.1.0
**Last updated:** 2026-04-01

This document is the execution companion to [ROADMAP.md](ROADMAP.md). It covers how to build Voce IR — technical decisions, tooling choices, workflow patterns, and the specific Claude Code workflow for solo development.

---

## 1. Development Environment

### 1.1 Toolchain

| Tool | Purpose | Version |
|------|---------|---------|
| Rust | Validator, compiler core, CLI tools | stable (1.85+), edition 2024 |
| Cargo | Rust package manager, workspace management | (bundled with Rust) |
| FlatBuffers (`flatc`) | Schema compilation → Rust/TS bindings | 24.x |
| Node.js | AI bridge service, preview server | 22+ LTS |
| TypeScript | AI bridge, preview tooling, inspector | 5.x (evaluate 6.x once ecosystem support stabilizes) |
| Claude Code | Primary development tool | latest |
| GitHub Actions | CI: build, test, lint, release | N/A |

### 1.2 Repository Setup

```bash
# Initialize the repo
mkdir voce-ir && cd voce-ir
git init

# Initialize Rust workspace
cat > Cargo.toml << 'EOF'
[workspace]
resolver = "2"
members = [
    "packages/schema",
    "packages/validator",
    "packages/compiler-dom",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "Apache-2.0"
repository = "https://github.com/marcpelland/voce-ir"

[workspace.dependencies]
flatbuffers = "24.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
clap = { version = "4.0", features = ["derive"] }
EOF

# Create package directories
cargo init packages/schema --lib
cargo init packages/validator
cargo init packages/compiler-dom
```

### 1.3 Monorepo Structure (Detailed)

```
voce-ir/
├── Cargo.toml                    # Rust workspace root
├── CLAUDE.md                     # Claude Code instructions
├── README.md                     # Project overview
├── LICENSE                       # Apache 2.0
│
├── docs/
│   ├── PRD.md                    # Product requirements
│   ├── ROADMAP.md                # Phased roadmap
│   ├── PROJECT_PLAN.md           # This file
│   ├── ARCHITECTURE.md           # Technical architecture (write during Phase 1)
│   └── spec/
│       ├── voce-ir-spec.html     # Narrative spec (v0.1 RFC)
│       └── ir-examples/          # Hand-authored example IR blobs
│           ├── landing-page.json # JSON representation (AI-readable)
│           └── landing-page.voce # Binary FlatBuffers (compiler-readable)
│
├── packages/
│   ├── schema/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs            # Re-exports generated FlatBuffers types
│   │   │   └── generated/        # Auto-generated from .fbs files
│   │   └── schemas/
│   │       ├── voce.fbs          # Master FlatBuffers schema
│   │       ├── layout.fbs        # Scene & layout node definitions
│   │       ├── state.fbs         # State machine & data node definitions
│   │       ├── motion.fbs        # Animation & interaction definitions
│   │       ├── navigation.fbs    # Routing definitions
│   │       ├── a11y.fbs          # Accessibility node definitions
│   │       ├── theming.fbs       # Theme & personalization definitions
│   │       └── types.fbs         # Primitive & composite type definitions
│   │
│   ├── validator/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # CLI entry point
│   │       ├── lib.rs            # Validation engine (library API)
│   │       ├── passes/
│   │       │   ├── mod.rs
│   │       │   ├── structural.rs # Structural completeness checks
│   │       │   ├── types.rs      # Type checking pass
│   │       │   ├── references.rs # Reference resolution pass
│   │       │   ├── state.rs      # State machine validation
│   │       │   ├── a11y.rs       # Accessibility enforcement
│   │       │   ├── motion.rs     # Motion safety (ReducedMotion) checks
│   │       │   └── data.rs       # Data completeness validation
│   │       └── errors.rs         # Typed error definitions
│   │
│   ├── compiler-dom/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # CLI entry point
│   │       ├── lib.rs            # Compiler engine (library API)
│   │       ├── pipeline/
│   │       │   ├── mod.rs
│   │       │   ├── ingest.rs     # IR deserialization
│   │       │   ├── optimize.rs   # Optimization passes
│   │       │   └── emit.rs       # HTML/JS emission
│   │       ├── codegen/
│   │       │   ├── mod.rs
│   │       │   ├── html.rs       # HTML generation
│   │       │   ├── css.rs        # Inline style generation
│   │       │   ├── js.rs         # JavaScript generation
│   │       │   └── a11y.rs       # ARIA attribute generation
│   │       └── layout/
│   │           ├── mod.rs
│   │           ├── resolver.rs   # Constraint resolution (static layouts)
│   │           └── runtime.rs    # Runtime layout code emission
│   │
│   └── ai-bridge/                # Phase 3 (Node.js/TypeScript)
│       ├── package.json
│       ├── tsconfig.json
│       └── src/
│           ├── index.ts          # Entry point
│           ├── prompt.ts         # System prompt construction
│           ├── schema-context.ts # IR schema → prompt context
│           ├── generate.ts       # Intent → Claude API → IR JSON
│           ├── encode.ts         # JSON → FlatBuffers binary
│           ├── repair.ts         # Validation error → repair prompt loop
│           └── preview.ts        # File watcher + compiler + HTTP server
│
├── examples/
│   ├── landing-page/
│   │   ├── intent.md             # Natural language description
│   │   ├── ir.json               # Generated IR (JSON form)
│   │   ├── ir.voce               # Binary IR
│   │   └── dist/
│   │       └── index.html        # Compiled output
│   └── intents/
│       └── *.md                  # Library of intent → IR examples
│
├── tests/
│   ├── schema/
│   │   ├── valid/                # IR blobs that must pass validation
│   │   └── invalid/              # IR blobs that must fail (with expected errors)
│   ├── compiler/
│   │   ├── snapshots/            # Expected HTML output for known IR inputs
│   │   └── a11y/                 # axe-core validation of compiled output
│   └── integration/
│       └── end-to-end/           # Intent → IR → compile → validate cycles
│
└── .github/
    ├── workflows/
    │   ├── ci.yml                # Build + test on every push
    │   └── release.yml           # Tagged release workflow
    └── CONTRIBUTING.md
```

## 2. Claude Code Workflow

### 2.1 CLAUDE.md

The `CLAUDE.md` file is the project-level instruction set that Claude Code reads on every session. It should contain:

```markdown
# Voce IR — Claude Code Instructions

## Project Overview
Voce IR is an AI-native UI intermediate representation. See docs/PRD.md for
full requirements and docs/ROADMAP.md for the current phase.

## Current Phase
Phase 1: Specification & Foundation

## Architecture
- Rust workspace with packages: schema, validator, compiler-dom
- FlatBuffers for IR binary format
- Schema files in packages/schema/schemas/*.fbs
- Validator in packages/validator/ (Rust)
- DOM compiler in packages/compiler-dom/ (Rust)
- AI bridge in packages/ai-bridge/ (TypeScript, Phase 3)

## Conventions
- All Rust code uses edition 2021
- Error handling: use thiserror for library errors, anyhow for CLI
- CLI argument parsing: use clap derive
- Tests: unit tests in-file (#[cfg(test)]), integration tests in tests/
- Naming: snake_case for files and functions, PascalCase for types
- Every public function has a doc comment
- No unwrap() in library code — propagate errors with ?

## Build & Test
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings

## Key Files
- packages/schema/schemas/voce.fbs — Master schema (source of truth)
- packages/validator/src/lib.rs — Validation engine entry point
- packages/compiler-dom/src/lib.rs — Compiler engine entry point
- docs/spec/ir-examples/ — Reference IR examples

## Working Pattern
When implementing a new feature:
1. Start with the schema (if new node types needed)
2. Regenerate bindings: flatc --rust -o packages/schema/src/generated/ packages/schema/schemas/*.fbs
3. Add validation pass in packages/validator/src/passes/
4. Add compiler support in packages/compiler-dom/src/codegen/
5. Add test cases in tests/ (both valid and invalid)
6. Run full test suite before committing
```

### 2.2 Working With Claude Code — Patterns

**Pattern: Schema-First Development**
Every new feature starts with the FlatBuffers schema. Claude Code should:
1. Edit the `.fbs` schema file
2. Run `flatc` to regenerate bindings
3. Update the validator to handle the new node type
4. Update the compiler to emit output for the new node type
5. Write tests at each layer

**Pattern: Red-Green-Refactor**
For each validation rule or compiler feature:
1. Write a test that fails (red)
2. Implement the minimum code to pass (green)
3. Refactor for clarity
4. Run full suite: `cargo test --workspace`

**Pattern: Golden File Testing**
For the compiler, use snapshot testing:
1. Create a known IR input (JSON → binary)
2. Compile it
3. Save the expected HTML output as a golden file
4. Future test runs compare compiled output against the golden file
5. If output changes intentionally, update the golden file with review

**Pattern: Incremental Complexity**
Don't implement the full spec of any node type at once. Example for Container:
- First pass: only `stack(vertical)` layout with fixed-size children
- Second pass: add `stack(horizontal)`
- Third pass: add `flex` with proportional sizing
- Fourth pass: add `grid` with explicit rows/cols
- Each pass has its own tests and golden files

### 2.3 Session Structure

Each Claude Code session should follow this structure:

1. **Start:** Review current state — what was last completed, what's next on the task list
2. **Scope:** Pick one focused task (e.g., "implement Container → flexbox emission")
3. **Execute:** Schema → validator → compiler → tests, in that order
4. **Verify:** `cargo test --workspace && cargo clippy --workspace`
5. **Commit:** Atomic commit with clear message referencing the task
6. **Update:** Mark task complete in the tracking issue or task list

## 3. Technical Architecture Decisions

### 3.1 Why FlatBuffers (Not Protobuf, Not Cap'n Proto)

- **Zero-copy deserialization** — IR can be memory-mapped and read without parsing. Critical for compiler startup time.
- **Schema evolution** — fields can be added without breaking existing IR blobs. Deprecated fields can be ignored.
- **Multi-language bindings** — generates Rust, TypeScript, Python, C++. Needed for AI bridge (TS) and compiler (Rust).
- **No runtime dependency** — unlike Protobuf, FlatBuffers doesn't require a runtime library in the compiled output.

Trade-offs: FlatBuffers has worse documentation than Protobuf and a smaller community. Cap'n Proto has better RPC support but weaker TypeScript story.

**Decision: FlatBuffers.** Confirmed by deep research (see `docs/research/DEEP_RESEARCH.md`). Zero-copy access patterns align with read-heavy UI workloads. Cap'n Proto's weaker TypeScript support is a dealbreaker for the AI bridge.

**Important limitation:** FlatBuffers are immutable — you cannot mutate a buffer in place. This means runtime mutable state (signal values, animation progress, fetched data) must live in a separate reactive layer outside the FlatBuffer. The FlatBuffer is the initial state + structure; runtime state is signals/observables that reference FlatBuffer nodes by ID.

**JSON canonical representation:** FlatBuffers supports JSON schema export. The AI bridge generates JSON (which LLMs handle well); the bridge encodes it to binary. The JSON format also serves as: debugging tool, version control diffing format, escape hatch for manual inspection, and AI training data.

### 3.2 Why Rust (Not Zig, Not Go, Not TypeScript)

- **Performance** — the compiler needs to be fast. Rust's zero-cost abstractions and no-GC model are ideal.
- **Correctness** — Rust's type system and ownership model catch categories of bugs at compile time that would be runtime errors in Go or TypeScript.
- **FlatBuffers support** — first-class Rust code generation from FlatBuffers schemas.
- **WASM target** — Rust compiles to WASM natively, which matters for Phase 4 (WASM compile target) and for potentially running the compiler in-browser.
- **Community** — strong ecosystem for parsers (nom, pest), code generation (quote, proc_macro), and CLI tools (clap).

Trade-offs: Slower development velocity than TypeScript for the AI bridge layer. Steeper learning curve for contributors.

**Decision: Rust for schema, validator, and compiler. TypeScript for AI bridge and preview server.**

### 3.3 Compiler Architecture

The compiler follows a standard multi-pass pipeline:

```
Validated IR (FlatBuffers binary)
  │
  ▼
[Ingest] — Deserialize IR into in-memory graph
  │
  ▼
[Optimize] — Dead state elimination, data fusion,
  │           layout pre-computation, tree shaking
  │
  ▼
[Lower] — Target-specific lowering:
  │        IR nodes → target-specific operations
  │        (DOM: nodes → HTML elements + JS statements)
  │        (WebGPU: nodes → draw calls + shaders)
  │
  ▼
[Emit] — Generate output:
  │      (DOM: concatenate HTML + inline CSS + JS into single file)
  │      (WebGPU: bundle shaders + JS init + WASM module)
  │
  ▼
Output file(s)
```

Each pass is a separate module. Passes are composable — new optimization passes can be added without modifying existing ones.

This pipeline mirrors the **SPIR-V architecture** (binary IR + validator + multi-target compilation) which is the closest existing precedent. See `docs/research/DEEP_RESEARCH.md` Section 1.4 for the full SPIR-V analogy.

**Reference implementations to study:**
- **SolidJS compiled output** — for surgical DOM mutation patterns (the DOM compiler should emit similar direct DOM calls)
- **Svelte compiled output** — for static vs dynamic node separation (only generate update code for data-bound nodes)
- **Compose's slot table** — for efficient flat binary representation of composition state

### 3.3a Layout Engine

**Decision: Taffy** (pure Rust flexbox + CSS grid engine, successor to Yoga).

Use Taffy for:
- **Compile-time layout resolution** — for IR nodes with known dimensions, pre-compute positions per breakpoint
- **Native compile targets** — where CSS layout isn't available
- **Validator** — verify layout constraints are satisfiable

For the DOM target, Taffy pre-computes what it can; the browser handles responsive/dynamic content via media queries and flexbox/grid.

### 3.4 DOM Emission Strategy

The DOM compiler emits a single HTML file containing:

1. **HTML structure** — minimal DOM nodes, exactly what's needed. No wrapper divs, no framework artifacts.
2. **Inline styles** — all styles computed at compile time and applied as `style=""` attributes. No `<style>` block, no CSS file, no cascade to resolve at runtime.
3. **JavaScript** — embedded `<script>` block containing state machine runtime, event handlers, and DOM mutation functions. Minified. No external dependencies.
4. **ARIA attributes** — applied directly to DOM nodes during HTML generation. Not injected by JS.

Why a single file: simplest deployment model (drop one file anywhere), zero network requests for resources, and easy to verify correctness.

### 3.5 State Machine Runtime

The compiled state machine is a simple switch-based implementation:

```javascript
// Compiled from StateMachine IR node
const state = { current: "idle" };
const transitions = {
  idle: { click: { target: "loading", guard: null, effect: "startFetch" } },
  loading: { resolve: { target: "loaded", guard: null, effect: "renderData" }, reject: { target: "error", guard: null, effect: "showError" } },
  loaded: { click: { target: "loading", guard: null, effect: "startFetch" } },
  error: { retry: { target: "loading", guard: null, effect: "startFetch" } }
};

function transition(event) {
  const t = transitions[state.current]?.[event];
  if (!t) return;
  if (t.guard && !guards[t.guard]()) return;
  effects[t.effect]?.();
  state.current = t.target;
  render();
}
```

This compiles to ~20 lines of JS per state machine. No framework. No library. Pure state transitions with direct DOM updates in the `render()` function.

## 4. Testing Strategy

### 4.1 Test Pyramid

```
           /  End-to-End  \        Intent → IR → compile → validate → render
          / (5-10 tests)    \      Slow, run in CI only
         /-------------------\
        /  Integration Tests  \    Validator + compiler together
       / (50-100 tests)        \   Medium speed, run on commit
      /-------------------------\
     /      Unit Tests           \  Individual passes, individual codegen
    / (200+ tests)                \ Fast, run constantly
   /-------------------------------\
```

### 4.2 Test Categories

**Schema Tests** (`tests/schema/`)
- Valid IR blobs that must pass validation → `tests/schema/valid/*.json`
- Invalid IR blobs with expected error types → `tests/schema/invalid/*.json`
- Each invalid test includes metadata: expected error code, expected node path

**Compiler Snapshot Tests** (`tests/compiler/snapshots/`)
- Known IR input → expected HTML output
- Use Rust's `insta` crate for snapshot management
- Update snapshots intentionally: `cargo insta review`

**Accessibility Tests** (`tests/compiler/a11y/`)
- Compiled HTML run through axe-core (via Playwright or similar)
- Tests verify: ARIA attributes present, focus order correct, reduced motion working

**Integration Tests** (`tests/integration/`)
- Full pipeline: JSON IR → validate → compile → HTML output → verify
- Includes "negative" integration tests: invalid IR → expected compiler rejection

### 4.3 CI Pipeline

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo test --workspace
      - run: cargo build --release --workspace
```

## 5. Open Source Strategy

### 5.1 License: Apache 2.0

Chosen because:
- Permissive: anyone can use, modify, distribute
- Patent grant: protects users from patent claims on the IR format
- Compatible with commercial use: doesn't prevent building proprietary tooling on top
- Standard for infrastructure projects (Kubernetes, Rust, Arrow all use Apache 2.0)

### 5.2 What's Open, What's Not

| Component | Open Source | Rationale |
|-----------|-----------|-----------|
| IR specification (`.fbs` schemas) | Yes | The format must be open to become a standard |
| Reference validator | Yes | Anyone should be able to verify IR correctness |
| DOM compiler | Yes | Core value proposition, attracts contributors |
| WebGPU compiler | Yes | Community-contributed targets expand the ecosystem |
| AI bridge (structured prompting) | Yes | Reference implementation, shows how to generate IR |
| Fine-tuned AI model weights | TBD | May be commercial if significant training investment |
| Visual inspector | TBD | Could be open core (basic open, advanced commercial) |
| Hosted compilation service | No | Commercial offering: upload IR, get compiled output |

### 5.3 Community Readiness Checklist (Pre-Launch)

Before making the repo public:

- [ ] README with clear project description and getting started
- [ ] CONTRIBUTING.md with code standards and PR process
- [ ] CODE_OF_CONDUCT.md
- [ ] Issue templates (bug report, feature request, new compile target proposal)
- [ ] At least 3 working examples in examples/
- [ ] CI passing on all PRs
- [ ] Published demo video/blog post explaining the project
- [ ] Discord or GitHub Discussions enabled for community Q&A

## 6. Risk Mitigations

### 6.1 "IR Can't Express X" Risk

**Mitigation: Escape Hatch Node**

Define a `RawTargetNode` in the IR that contains opaque target-specific code:

```flatbuffers
table RawTargetNode {
  target: CompileTarget;   // DOM, WebGPU, WASM, Native
  code: string;            // Raw target code (HTML, JS, WGSL, etc.)
  inputs: [DataBinding];   // Data that flows into the raw code
  outputs: [DataBinding];  // Data that flows out
  semantic: SemanticNode;  // Still required — accessibility is non-negotiable
}
```

This sacrifices portability (the raw code only works on one target) but prevents the IR from being a dead end when it encounters an edge case. Track `RawTargetNode` usage as a metric — high usage means the IR spec needs expansion.

### 6.2 "AI Generates Invalid IR" Risk

**Mitigation: Repair Loop**

```
Intent → AI generates IR → Validator checks
                              │
                     Valid? ──┤
                     Yes: ────► Compile
                     No: ─────► Extract errors
                                   │
                                   ▼
                              Feed errors back to AI as follow-up:
                              "The IR you generated has these errors: [...]
                               Please fix them and regenerate."
                                   │
                                   ▼
                              AI generates corrected IR → Validator checks
                              (max 3 repair attempts, then surface to human)
```

### 6.3 "Solo Builder Bottleneck" Risk

**Mitigation: Extreme Modularity**

Every package is independent with a clear API boundary. If the project attracts contributors:
- New compile targets are entirely new packages — no need to touch existing code
- New validation passes are single files in `packages/validator/src/passes/`
- New IR node types are schema additions + corresponding validator + compiler support

The architecture is designed so that a contributor can add a new compile target without understanding the validator, and vice versa.

---

*This plan is designed for execution with Claude Code as the primary development tool. Update the CLAUDE.md file at the start of each phase to reflect current context and priorities.*
