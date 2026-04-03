---
name: 3D Compilation Strategy Decision
description: Three.wasm is a joke. Real options for 3D: custom WebGPU (simple scenes), Three.js (complex scenes), hybrid approach recommended.
type: project
---

Marc found three.wasm (mrdoob/three.wasm) — turns out it's an April Fools' joke (April 1, 2026). Fake FPS counter, single hardcoded cube, not a real project.

**Why:** The idea of leveraging an existing 3D library for compilation is sound but three.wasm isn't it.

**How to apply:** For Voce IR's 3D compilation:
- **Simple scenes** (primitives, basic lighting, particles): use our custom WebGPU renderer (packages/compiler-webgpu). Smallest output, full control.
- **Complex scenes** (glTF models, PBR, environment maps): consider compiling to Three.js API calls as an optional target. ~175KB runtime but battle-tested.
- **Alternative:** Babylon.js has stronger TypeScript and WebGPU-native support.
- The compiler should choose based on scene complexity: simple → custom WebGPU, complex → Three.js runtime.
- This is a Phase 4 decision (S36-S38 scope).
