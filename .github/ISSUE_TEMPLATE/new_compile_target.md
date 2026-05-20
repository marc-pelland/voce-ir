---
name: New Compile Target Proposal
about: Propose a new compile target for Voce IR (e.g. Flutter, React Native, TUI, e-paper)
title: "Compile target: <name>"
labels: enhancement, compile-target
---

**Target**
What platform / framework / format? (e.g. Flutter, React Native, terminal UI, e-paper, slide deck)

**Why this target?**
What use case does it unlock that today's 7 targets (DOM / Hybrid / Email / WebGPU / WASM / iOS / Android) don't? Be specific — every target carries maintenance cost.

**Conformance class**
Per `docs/compatibility-matrix.md`, which class best describes it — and why?

- [ ] `OracleFull` — can represent every IR semantic exactly (rare; today only DOM)
- [ ] `OracleFullSuperset` — DOM-equivalent plus extras (Hybrid)
- [ ] `RequiredContract` — preserves a documented subset; medium-degrades the rest (Email)
- [ ] `NonHtmlVisual` — visual-only; semantic parity needs an a11y-tree extractor (WebGPU)
- [ ] `LogicOnly` — logic, not UI (WASM)
- [ ] `Native` — emits native-platform source (iOS / Android)

**Output shape**
What does the compiler emit? (single file, multi-file, source vs. binary, runtime requirements)

**Three-pillar fit**
How does this target preserve:

- **Accessibility** — semantic structure, focus order, AT story
- **Stability** — security posture, zero-runtime-deps stance (or documented exception)
- **Experience** — performance characteristics, animation/motion

**Acceptance**
What would "this target is shipped" look like? (subset of IR supported, conformance level claimed, fixtures it must pass)

**Implementation notes**
Anything you've already thought through — existing libraries, prior art, gotchas.

**Maintainer**
Are you proposing to build this, or proposing it for someone else? Either is fine; the answer changes the discussion.
