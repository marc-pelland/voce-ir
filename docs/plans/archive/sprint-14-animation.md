# Sprint 14 — Animation Compilation

**Status:** Planned
**Goal:** Implement the tiered animation strategy from the research. CSS for zero-JS motion, WAAPI for choreographed sequences, minimal rAF JS for interruptible springs. Compile-time spring curve solving via ODE in Rust emitting CSS `linear()` points. After this sprint, animations compile to the lightest possible technique per use case with full reduced-motion support.
**Depends on:** Sprint 13 (state machines, event binding, JS emission)

---

## Deliverables

1. Tier 1 (CSS): Transition on hover/focus → CSS `transition` properties
2. Tier 1 (CSS): Spring easing → pre-computed `linear()` CSS via Rust ODE solver
3. Tier 1 (CSS): ScrollBinding → CSS `animation-timeline: view()/scroll()`
4. Tier 2 (WAAPI): Sequence → `element.animate()` with `.finished` chaining
5. Tier 3 (rAF): PhysicsBody with `interruptible: true` → rAF spring stepper
6. ReducedMotion → `@media (prefers-reduced-motion: reduce)` overrides
7. View Transitions API for RouteTransition
8. Spring ODE solver module

---

## Tasks

### 1. Animation Tier Selection (`animation/tier.rs`)

Analyze each motion node and select the compilation tier:
