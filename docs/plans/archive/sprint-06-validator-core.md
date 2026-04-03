# Sprint 06 — Validator: Core Passes

**Status:** Planned
**Goal:** Build the validation engine and implement the 3 foundational passes: structural completeness, reference resolution, and state machine validation. After this sprint, the validator can load IR from JSON, index all nodes, and report typed diagnostics for the most fundamental correctness rules.
**Depends on:** Sprint 05 (complete schema with all node types)

---

## Architecture Decision: Serde IR Model

FlatBuffers zero-copy access is excellent for runtime performance but painful for multi-pass validation. Navigating nested tables requires verbose accessor chains, optional unwrapping at every level, and no ability to collect or cross-reference nodes ergonomically.

**Decision:** Create a parallel serde-deserializable IR model (`ir.rs`) that mirrors the FlatBuffers schema as plain Rust structs with `#[derive(Deserialize)]`. The validator loads IR from the JSON canonical format via serde_json, builds a `NodeIndex` for fast lookups, then runs passes over the serde model.

This means the validator works on `.voce.json` files directly. The `voce json2bin` / `voce bin2json` commands (Sprint 08) handle conversion. The FlatBuffers binary format remains the production wire format; JSON is the development and validation format.

---

## Deliverables

1. `ValidationPass` trait and pass-runner engine
2. Serde-deserializable IR model covering all node types
3. `NodeIndex` — O(1) lookup by node ID, parent traversal, type filtering
4. Structural completeness pass (STR001-STR008)
5. Reference resolution pass (REF001-REF010)
6. State machine validation pass (STA001-STA005)
7. 8 invalid IR fixtures exercising the new passes
8. 12+ unit and integration tests

---

## Tasks

### 1. Validation Engine (`engine.rs`)

Define the `ValidationPass` trait and the runner that executes passes in order.

```rust
pub trait ValidationPass {
    /// Unique name for this pass (e.g., "structural", "references")
    fn name(&self) -> &'static str;

    /// Run the pass, appending diagnostics to the result.
    fn run(&self, ir: &VoceIr, index: &NodeIndex, result: &mut ValidationResult);
}
```

The engine:
- Accepts a `VoceIr` (deserialized root)
- Builds a `NodeIndex`
- Runs each registered pass in sequence
- Returns aggregated `ValidationResult`

Passes run in dependency order: structural -> references -> state_machine.

### 2. Serde IR Model (`ir.rs`)

Create Rust structs mirroring every FlatBuffers table. Key types:

- `VoceIr` — root, contains `ViewRoot`
- `ViewRoot`, `Container`, `Surface`, `TextNode`, `MediaNode`
- `StateMachine`, `DataNode`, `ComputeNode`, `EffectNode`, `ContextNode`
- `Transition`, `Sequence`, `GestureHandler`, `ScrollBinding`, `PhysicsBody`
- `RouteMap`, `RouteTransition`
- `SemanticNode`, `LiveRegion`, `FocusTrap`, `ReducedMotion`
- `ThemeNode`, `PersonalizationSlot`, `ResponsiveRule`
- `ActionNode`, `FormNode`, `PageMetadata`, `LocalizedString`
- `ChildUnion` — serde enum with `#[serde(tag = "type")]`

All structs derive `Deserialize`, `Debug`, `Clone`. Fields that are optional in the schema use `Option<T>`.

### 3. Node Index (`index.rs`)

Build a fast lookup structure from the deserialized IR tree.

```rust
pub struct NodeIndex {
    /// node_id -> (node_path, node_type, parent_id)
    nodes: HashMap<String, NodeEntry>,
    /// node_type -> [node_id]
    by_type: HashMap<String, Vec<String>>,
}
```

Capabilities:
- `get(id) -> Option<&NodeEntry>` — O(1) by ID
- `by_type(type_name) -> &[String]` — all nodes of a given type
- `parent(id) -> Option<&str>` — parent node ID
- `path(id) -> &str` — JSON-pointer-style path for diagnostics
- `children(id) -> &[String]` — direct children

Built by a single recursive walk of the IR tree after deserialization.

### 4. Structural Completeness Pass (`structural.rs`)

Checks that required fields are present and node structure is valid.

| Code | Rule | Severity |
|------|------|----------|
| STR001 | ViewRoot must exist at document root | Error |
| STR002 | Every node must have a unique `id` field | Error |
| STR003 | Container must have at least one child | Warning |
| STR004 | TextNode must have non-empty `content` or `localized_content` | Error |
| STR005 | MediaNode must have `src` or `source` | Error |
| STR006 | Surface must have `width` and `height` or layout constraints | Error |
| STR007 | No orphan nodes (every non-root node has a reachable parent) | Error |
| STR008 | ChildUnion discriminator matches actual node type | Error |

### 5. Reference Resolution Pass (`references.rs`)

Validates that all cross-node references (`ref_id` fields) point to existing nodes of the correct type.

| Code | Rule | Severity |
|------|------|----------|
| REF001 | DataNode `source_ref` must reference a valid DataNode or ActionNode | Error |
| REF002 | ComputeNode `input_refs` must all resolve | Error |
| REF003 | EffectNode `trigger_ref` must reference a StateMachine or DataNode | Error |
| REF004 | Transition `target_state` must be a state in the same StateMachine | Error |
| REF005 | GestureHandler `target_ref` must reference an existing node | Error |
| REF006 | ScrollBinding `source_ref` and `target_ref` must resolve | Error |
| REF007 | RouteTransition `target_route` must exist in RouteMap | Error |
| REF008 | FormField `visibility_condition` refs must resolve | Error |
| REF009 | No circular references in ComputeNode dependency chains | Error |
| REF010 | ContextNode `provider_ref` must reference a valid ContextNode | Error |

### 6. State Machine Validation Pass (`state_machine.rs`)

Validates StateMachine semantics.

| Code | Rule | Severity |
|------|------|----------|
| STA001 | StateMachine must have at least one state | Error |
| STA002 | StateMachine must declare an `initial_state` that exists in its states | Error |
| STA003 | Every state must be reachable from `initial_state` via transitions | Warning |
| STA004 | No duplicate transition triggers within the same state | Error |
| STA005 | Guard conditions must reference valid DataNode or ComputeNode | Error |

### 7. Invalid IR Fixtures

Create in `tests/schema/invalid/`:

- `missing-viewroot.voce.json` — triggers STR001
- `duplicate-ids.voce.json` — triggers STR002
- `empty-textnode.voce.json` — triggers STR004
- `broken-datanode-ref.voce.json` — triggers REF001
- `circular-compute.voce.json` — triggers REF009
- `missing-initial-state.voce.json` — triggers STA002
- `unreachable-state.voce.json` — triggers STA003
- `duplicate-transitions.voce.json` — triggers STA004

### 8. Tests

Unit tests in each pass module:
- 3+ tests per pass (valid IR produces no diagnostics, invalid IR produces expected codes)
- Edge cases: empty IR, single-node IR, deeply nested trees

Integration tests in `tests/`:
- Load each invalid fixture, run full validation, assert specific diagnostic codes
- Load valid fixture, run full validation, assert zero errors

---

## Files to Create / Modify

### Create
- `packages/validator/src/engine.rs` — ValidationPass trait + runner
- `packages/validator/src/ir.rs` — Serde IR model
- `packages/validator/src/index.rs` — NodeIndex
- `packages/validator/src/passes/structural.rs` — STR001-STR008
- `packages/validator/src/passes/references.rs` — REF001-REF010
- `packages/validator/src/passes/state_machine.rs` — STA001-STA005
- `tests/schema/invalid/missing-viewroot.voce.json`
- `tests/schema/invalid/duplicate-ids.voce.json`
- `tests/schema/invalid/empty-textnode.voce.json`
- `tests/schema/invalid/broken-datanode-ref.voce.json`
- `tests/schema/invalid/circular-compute.voce.json`
- `tests/schema/invalid/missing-initial-state.voce.json`
- `tests/schema/invalid/unreachable-state.voce.json`
- `tests/schema/invalid/duplicate-transitions.voce.json`

### Modify
- `packages/validator/src/lib.rs` — add `pub mod engine; pub mod ir; pub mod index;`
- `packages/validator/src/passes/mod.rs` — register structural, references, state_machine passes
- `packages/validator/Cargo.toml` — add `serde`, `serde_json` dependencies if not already present

---

## Acceptance Criteria

- [ ] `ValidationPass` trait defined with `name()` and `run()` methods
- [ ] Serde IR model can deserialize a valid `.voce.json` fixture
- [ ] `NodeIndex` supports O(1) lookup by ID and type filtering
- [ ] Structural pass catches all 8 STR codes on invalid input
- [ ] Reference pass catches all 10 REF codes on invalid input
- [ ] State machine pass catches all 5 STA codes on invalid input
- [ ] 8 invalid IR fixture files exist in `tests/schema/invalid/`
- [ ] 12+ tests passing
- [ ] Valid IR produces zero diagnostics across all 3 passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] Every public type and function has `///` doc comments

---

## Notes

- The serde IR model will diverge from FlatBuffers generated code intentionally. The generated FlatBuffers bindings remain the source of truth for the binary wire format. The serde model is the source of truth for validation logic.
- Circular reference detection (REF009) uses iterative DFS with a visited set — not recursion — to avoid stack overflow on pathological inputs.
- The `NodeIndex` build step is O(n) where n = total nodes. Validation passes should avoid re-walking the full tree when the index provides what they need.
