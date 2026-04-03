# Sprint 11 — Compiler Pipeline Foundation

**Status:** Planned
**Goal:** Define the compiler pipeline architecture (ingest, optimize, lower, emit), create the compiler's in-memory IR representation optimized for code generation, build the IR ingestion layer, and emit a minimal valid HTML5 document from a ViewRoot. After this sprint, a minimal IR JSON file compiles to a valid HTML5 page end-to-end.
**Depends on:** Sprint 10 (Phase 1 complete, v0.1.0 tagged)

---

## Architecture Decision: Compiler IR vs Validator IR

The validator's serde IR model (`voce-validator::ir`) is optimized for traversal and rule checking — fields are `Option<T>`, values stay as `serde_json::Value` where validation doesn't inspect them. The compiler needs a different representation:

- **Resolved types:** No `Option` for required fields (validation already passed). No `serde_json::Value` — every field is a concrete Rust type.
- **Graph structure:** Nodes stored in an arena with typed handles. Parent/child/sibling traversal via indices, not string lookups.
- **Annotation slots:** Each node carries mutable annotation space for optimization passes (e.g., "dead state", "above fold", "computed layout rect").
- **Lowering targets:** Each node maps to one or more `HtmlElement` or `JsStatement` in the lowering phase.

**Decision:** Create a new `compiler_ir` module in `voce-compiler-dom`. Ingestion loads from the validator's serde model (which loads from JSON), then converts to the compiler IR. This isolates the compiler from the validator's internal types while reusing its JSON parsing.

---

## Deliverables

1. `CompilerPipeline` struct orchestrating ingest → optimize → lower → emit
2. Compiler IR types: `CompilerIr`, `NodeArena`, `NodeHandle`, typed node variants
3. IR ingestion: `voce-validator` JSON → compiler IR conversion
4. HTML emitter: minimal HTML5 document skeleton from ViewRoot
5. `CompileOptions` configuration (output path, minify, source maps, etc.)
6. `CompileResult` with output HTML string + diagnostics + metadata
7. First end-to-end test: `minimal-page.voce.json` → `index.html`

---

## Tasks

### 1. Compiler Pipeline (`pipeline.rs`)

Define the four-phase pipeline as a struct with methods:
