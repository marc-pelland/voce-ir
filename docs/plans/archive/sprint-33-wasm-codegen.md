# Sprint 33 — WASM Code Generation

**Status:** Planned
**Phase:** 4 (Multi-Target Compilation)
**Depends on:** S20 (compiler-dom complete)

---

## Goal

Compile StateMachine and ComputeNode IR to WebAssembly for compute-heavy logic, with a JS interop bridge for DOM integration.

---

## Deliverables

- `packages/compiler-wasm/` Rust crate
- StateMachine → WASM function compilation (state transitions as WASM calls)
- ComputeNode → WASM pure function compilation (data transforms, calculations)
- Memory management: linear memory allocation, typed array views
- JS interop bridge: WASM exports callable from DOM-compiled JS
- wasm-bindgen or raw import/export table generation
- Size optimization: wasm-opt pass, dead code elimination
- Performance benchmarks: WASM state machine vs equivalent JS
- `voce compile --target wasm` for WASM-only output
- 10+ tests covering state machine and compute compilation

---

## Acceptance Criteria

- [ ] StateMachine with 5+ states compiles to working WASM module
- [ ] ComputeNode pure functions callable from JS via interop bridge
- [ ] WASM module size < 5KB for a typical state machine
- [ ] State transitions in WASM are measurably faster than JS equivalent
- [ ] Memory is properly managed (no leaks over 1000 state transitions)
- [ ] `cargo test` and `cargo clippy` pass
