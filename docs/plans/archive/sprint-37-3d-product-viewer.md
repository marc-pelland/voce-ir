# Sprint 37 — 3D Product Viewer Demo

**Status:** Planned
**Phase:** 4 (Multi-Target Compilation)
**Depends on:** S36 (AI bridge for 3D)

---

## Goal

Build an interactive 3D product viewer entirely through conversation, demonstrating the full multi-target pipeline from natural language to WebGPU output.

---

## Deliverables

- Complete 3D product viewer built via conversational AI (no manual IR editing)
- Features: orbit camera, environment lighting, product annotations, color variants
- Mixed 2D/3D page: product info (DOM) + 3D viewer (WebGPU) + add-to-cart (DOM)
- Responsive: 3D viewer scales to viewport, touch controls on mobile
- Accessibility: keyboard orbit controls, alt text for 3D content, reduced motion
- Graceful degradation: Canvas fallback with 2D product images
- Screen recording of the conversational build session
- Published in examples/3d-product-viewer/

---

## Acceptance Criteria

- [ ] Product viewer renders in WebGPU with orbit camera controls
- [ ] Color variant switching updates the 3D model material in real-time
- [ ] Annotations overlay on 3D scene (click hotspot → info panel)
- [ ] DOM layout (product info, CTA) integrates seamlessly with 3D canvas
- [ ] Canvas 2D fallback provides usable product viewing experience
- [ ] Keyboard controls: arrow keys orbit, +/- zoom, Tab to annotations
- [ ] Entire project built through conversation (no manual edits)
