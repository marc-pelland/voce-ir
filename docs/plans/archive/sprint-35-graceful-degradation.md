# Sprint 35 — Graceful Degradation

**Status:** Planned
**Phase:** 4 (Multi-Target Compilation)
**Depends on:** S34 (hybrid compilation)

---

## Goal

Implement the fallback chain: WebGPU → Canvas 2D → static image, so 3D content degrades gracefully on devices without GPU support.

---

## Deliverables

- Runtime capability detection: WebGPU availability check at load time
- Canvas 2D fallback renderer for Scene3D (simplified 3D → 2D projection)
- Static image fallback: pre-rendered snapshot for no-JS/no-canvas environments
- Fallback selection logic embedded in compiled output
- Progressive enhancement: DOM content loads first, 3D initializes async
- Loading states: skeleton/placeholder while WebGPU initializes
- WASM fallback: pure JS equivalent when WASM unavailable
- User-facing capability report (what features are active on this device)
- 10+ tests covering each fallback path

---

## Acceptance Criteria

- [ ] WebGPU scene renders on supported browsers (Chrome, Edge)
- [ ] Canvas 2D fallback activates on Firefox/Safari without WebGPU
- [ ] Static image fallback works with JavaScript disabled
- [ ] Page remains functional (DOM content, navigation) regardless of GPU support
- [ ] No console errors on any fallback path
- [ ] Load time penalty for fallback detection < 50ms
- [ ] `cargo test` and `cargo clippy` pass
