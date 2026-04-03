# Sprint 34 — Hybrid Compilation

**Status:** Planned
**Phase:** 4 (Multi-Target Compilation)
**Depends on:** S31 (WebGPU), S33 (WASM)

---

## Goal

Implement per-component target analysis so the compiler automatically selects DOM, WebGPU, or WASM for each IR node based on its characteristics and the device profile.

---

## Deliverables

- Target analysis pass: classify each node as DOM, WebGPU, or WASM candidate
- Decision heuristics: Scene3D/MeshNode → WebGPU, heavy StateMachine → WASM, layout → DOM
- DeviceProfile IR integration: GPU capabilities, memory limits, CPU cores
- Device profile presets: mobile-low, mobile-high, desktop, desktop-gpu
- Unified output bundling: single HTML with embedded WASM + WebGPU init
- Shared state bridge: DOM and WASM components communicate via shared memory
- WebGPU canvas embedded within DOM layout (mixed 2D/3D pages)
- Compilation report shows target selection rationale per node
- 10+ tests covering target selection logic and hybrid output

---

## Acceptance Criteria

- [ ] IR with mixed 2D layout and 3D scene compiles to hybrid DOM+WebGPU output
- [ ] Heavy state machine automatically routes to WASM in hybrid mode
- [ ] Device profile "mobile-low" produces DOM-only output (no WebGPU)
- [ ] Device profile "desktop-gpu" produces DOM+WebGPU+WASM hybrid
- [ ] State changes in WASM reflect in DOM elements
- [ ] 3D canvas integrates cleanly within flexbox/grid DOM layout
- [ ] `cargo test` and `cargo clippy` pass
