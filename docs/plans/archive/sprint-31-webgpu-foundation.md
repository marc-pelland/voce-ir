# Sprint 31 — WebGPU Renderer Foundation

**Status:** Planned
**Phase:** 4 (Multi-Target Compilation)
**Depends on:** S20 (compiler-dom complete), S30 (AI bridge complete)

---

## Goal

Build the foundational WebGPU compile target: Scene3D nodes compile to a working WebGPU render pipeline with camera, lighting, and basic mesh rendering.

---

## Deliverables

- `packages/compiler-webgpu/` Rust crate with compiler pipeline
- Scene3D IR node → WebGPU device initialization + render loop
- Camera system: perspective and orthographic projection, orbit controls
- Lighting: directional, point, and ambient light compilation
- Basic mesh rendering: cube, sphere, plane primitives from MeshNode
- Vertex buffer layout and index buffer generation
- Depth buffer and basic render pass configuration
- `voce compile --target webgpu` CLI flag
- Single-file HTML output with embedded WebGPU initialization
- 10+ unit tests for render pipeline stages

---

## Acceptance Criteria

- [ ] Scene3D node with a single MeshNode compiles to working WebGPU output
- [ ] Camera orbits around the scene with mouse drag
- [ ] Directional light produces visible shading on mesh surfaces
- [ ] Output runs in Chrome/Edge with WebGPU enabled
- [ ] Output file is a single self-contained HTML file
- [ ] `cargo test -p compiler-webgpu` passes all tests
- [ ] `cargo clippy -p compiler-webgpu -- -D warnings` passes
