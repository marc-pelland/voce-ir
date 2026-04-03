# Phase 4 Benchmarks — Multi-Target Compilation

**Date:** 2026-04-02
**Status:** Template (populate with real GPU benchmark data when running on hardware)

---

## WebGPU Renderer vs Three.js

| Metric | Voce IR WebGPU | Three.js (est.) |
|--------|---------------|----------------|
| Bundle size (single cube) | ~8 KB | ~175 KB |
| Bundle size (PBR scene) | ~12 KB | ~200 KB |
| FPS (100 objects) | TBD | ~60 fps |
| FPS (1000 particles) | TBD | ~60 fps |
| GPU memory (basic scene) | TBD | ~50 MB |
| Load time (first paint) | TBD | ~200ms |
| Shader compilation | Compile-time WGSL | Runtime GLSL→WebGL |

**Voce IR advantage:** Output size 10-20x smaller (no framework runtime). Shaders compiled at build time, not runtime.

## WASM State Machine vs JavaScript

| Metric | WASM | JS (compiled by DOM compiler) |
|--------|------|-------------------------------|
| Module size (5-state SM) | ~500B WAT | ~200B JS |
| Module size (15-state SM) | ~1.2 KB WAT | ~600B JS |
| Transition latency | <0.01ms | <0.05ms |
| Memory (per SM) | 4 bytes (linear memory) | ~100 bytes (JS object) |

**Note:** For simple state machines (<10 states), JS is faster due to no WASM instantiation overhead. WASM wins for complex machines with frequent transitions (>100/sec).

## Hybrid Compilation Target Selection

| IR Content | Target | Rationale |
|-----------|--------|-----------|
| Container, TextNode, Surface | DOM | Layout/text — browser engine is optimal |
| Scene3D, MeshNode | WebGPU | 3D rendering — GPU required |
| StateMachine (>10 states) | WASM | Complex state logic — predictable performance |
| ComputeNode (complex) | WASM | Math-heavy — no GC pauses |
| ParticleSystem | WebGPU compute | GPU parallelism essential |
| FormNode, ActionNode | DOM | HTML forms — progressive enhancement |
| AnimationTransition | CSS | Compositor thread — zero JS |

## Device Profile Impact

| Profile | Output Composition | Total Size |
|---------|-------------------|-----------|
| desktop (WebGPU) | DOM + WebGPU + WASM | ~15 KB |
| mobile-high (WebGPU) | DOM + WebGPU | ~12 KB |
| mobile-low (no GPU) | DOM + Canvas 2D fallback | ~8 KB |

## 3D Product Viewer Demo

| Metric | Value |
|--------|-------|
| IR nodes | 18 (8 DOM, 1 Scene3D, 1 MeshNode, 3 Semantic) |
| DOM output | 2.7 KB |
| WebGPU output (est.) | ~8 KB |
| Total hybrid (est.) | ~12 KB |
| Three.js equivalent (est.) | ~200 KB |
| Fallback (Canvas 2D) | +0.8 KB |
| Accessibility | Keyboard orbit, alt text, reduced motion |

---

*Populate with measured data when running on hardware with WebGPU support.*
