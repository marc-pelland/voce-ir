# Sprint 38 — Benchmarks & v0.4.0 Release

**Status:** Planned
**Phase:** 4 (Multi-Target Compilation)
**Depends on:** S37 (3D product viewer demo)

---

## Goal

Comprehensive performance benchmarking of multi-target compilation, documentation of Phase 4 work, and v0.4.0 release.

---

## Deliverables

- Benchmark suite: Voce WebGPU output vs Three.js equivalent
  - Metrics: FPS, draw calls, GPU memory, load time, output size
  - Test scenes: 100 objects, 1000 particles, PBR materials
- Benchmark suite: Voce WASM state machine vs equivalent JS
  - Metrics: transition latency, memory usage, module size
- Hybrid compilation analysis: output size breakdown by target
- CHANGELOG.md for v0.4.0
- Updated ARCHITECTURE.md with multi-target compilation details
- Updated ROADMAP.md with Phase 4 completion status
- API documentation for compiler-webgpu and compiler-wasm crates
- Tag v0.4.0 release
- crates.io publish: compiler-webgpu, compiler-wasm

---

## Acceptance Criteria

- [ ] Benchmark results documented with reproducible methodology
- [ ] Voce WebGPU output is within 20% of Three.js FPS for equivalent scene
- [ ] Voce WebGPU output size < 50% of Three.js bundle for equivalent scene
- [ ] WASM state machine transitions < 1ms for 10-state machine
- [ ] CHANGELOG covers all Phase 4 features
- [ ] v0.4.0 tagged and published
- [ ] All Phase 4 tests pass: `cargo test --workspace`
