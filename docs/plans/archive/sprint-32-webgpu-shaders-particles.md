# Sprint 32 — WebGPU Shaders & Particles

**Status:** Planned
**Phase:** 4 (Multi-Target Compilation)
**Depends on:** S31 (WebGPU renderer foundation)

---

## Goal

Compile ShaderNode to WGSL shaders and ParticleSystem to GPU compute shaders. Build a material system that maps IR shader descriptions to efficient WGSL code.

---

## Deliverables

- ShaderNode IR → WGSL transpilation pipeline
- Material system: PBR (physically-based rendering) default material
- Texture sampling and UV mapping compilation
- ParticleSystem IR → compute shader pipeline (spawn, update, render)
- Particle emitter types: point, sphere, cone, mesh surface
- Particle forces: gravity, wind, turbulence from IR properties
- Mesh instancing for particle rendering (single draw call)
- LOD (level of detail) selection based on camera distance
- 15+ tests covering shader compilation and particle simulation

---

## Acceptance Criteria

- [ ] ShaderNode with custom properties compiles to valid WGSL
- [ ] PBR material renders with metallic/roughness workflow
- [ ] ParticleSystem runs on GPU compute shader (1000+ particles at 60fps)
- [ ] Particle emitters spawn from configured geometry
- [ ] LOD switching works based on camera distance thresholds
- [ ] All WGSL output passes validation (no shader compilation errors)
- [ ] `cargo test` and `cargo clippy` pass
