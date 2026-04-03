# Sprint 41 — A11y & Performance Inspector

**Status:** Planned
**Phase:** 5 (Visual Inspector & Tooling)
**Depends on:** S40 (state/animation inspector)

---

## Goal

Add live accessibility tree visualization, focus order debugging, and frame timing profiler to the inspector.

---

## Deliverables

- Live accessibility tree: mirrors browser a11y tree, shows ARIA roles/labels/states
- Focus order visualization: numbered overlay showing tab order across the page
- Tab-through simulator: step through focus order without actually tabbing
- Missing semantic node detection: real-time warnings for uncovered interactive elements
- Screen reader preview: text-only rendering of what a screen reader would announce
- Frame timing profiler: per-frame breakdown (layout, paint, composite, JS)
- GPU utilization meter (for WebGPU targets): draw calls, memory, shader time
- Performance warnings: jank detection, long task identification
- Export profiler data as JSON for external analysis

---

## Acceptance Criteria

- [ ] A11y tree displays correct ARIA hierarchy for compiled output
- [ ] Focus order overlay shows numbered tab stops
- [ ] Missing semantic nodes flagged with warning icons
- [ ] Screen reader preview produces accurate text representation
- [ ] Frame profiler shows per-frame timing breakdown
- [ ] GPU meter reports draw calls and memory for WebGPU output
- [ ] Jank frames (>16ms) highlighted in profiler
