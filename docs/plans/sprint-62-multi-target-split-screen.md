# Sprint 62 — Multi-Target Split-Screen

**Phase:** 7 — Production Readiness
**Status:** Planned (depends on S61 + spike outcome)
**Goal:** Add a second below-the-fold section on `voce-ir.xyz` that takes one canonical IR and shows it compiled to DOM, iOS SwiftUI, Android Compose, and Email HTML simultaneously. This is the strongest possible demonstration of the project's "one IR, every platform" pitch.

**Depends on:** S61 (live hero proves `playground-wasm` works in the homepage), wasm32 compatibility spike (see Risks)

---

## Pre-Sprint Spike (1 day, blocking)

Before committing to this sprint plan, run a 1-day investigation to answer:

- Does `compiler-ios` build for `wasm32-unknown-unknown` with the current dependency tree?
- Does `compiler-android` build for `wasm32-unknown-unknown`?
- Does `compiler-email` build for `wasm32-unknown-unknown`? (likely yes — already used by `playground-wasm` for `compile_email`)
- Identify any non-wasm-compatible dependencies (filesystem, threads, native crypto) and the cost of replacing them.

**Spike output:** a 1–2 page memo in `docs/plans/s62-spike-findings.md`. If iOS or Android cannot reasonably reach wasm32, this sprint is rescoped to "source-only output rendered server-side at build time" — same UI, different data path.

---

## Deliverables (assuming spike succeeds)

### 1. New WASM exports in `packages/playground-wasm/src/lib.rs`

- `compile_ios(ir_json: &str) -> String` — returns SwiftUI source as a string
- `compile_android(ir_json: &str) -> String` — returns Compose source as a string
- (`compile_email` and `compile_dom` already exist from S53/S61)

### 2. `packages/site-targets/` — TypeScript package

- Tab strip: **DOM** (live render) · **iOS SwiftUI** (syntax-highlighted source) · **Android Compose** (syntax-highlighted source) · **Email HTML** (live render in iframe)
- Toggle between **3 reference IRs** chosen for cross-target interest:
  - hero section (text + image + CTA — exercises typography, image pipeline, button)
  - contact form (FormNode — exercises validation, semantics)
  - pricing table (Container with grid — exercises layout primitives)
- All four panes update within 500ms when switching IRs (cached compilation results acceptable; live compilation preferred)

### 3. Wired into landing page

- New section between the existing pipeline diagram and the features grid.
- Section copy: short tagline ("One IR. Four targets. Same source of truth.") + the split-screen.

### 4. Build journal addendum

Append to `docs/site-v2-build-journal.md`:
- Findings from the wasm32 spike
- Each non-wasm-compatible dependency replaced and how
- Bundle size delta after adding iOS + Android compilers
- At least 3 new findings with resolutions

---

## Acceptance Criteria

- [ ] Spike memo `docs/plans/s62-spike-findings.md` exists and was reviewed before sprint started
- [ ] `playground-wasm` exports `compile_ios` and `compile_android`; both round-trip the 3 reference IRs without panicking
- [ ] Multi-target section is live on `voce-ir.xyz` below the fold
- [ ] All 4 panes show output for all 3 reference IRs
- [ ] Switching between IRs updates all 4 panes in under 500ms (cache-warm)
- [ ] Combined post-interaction WASM size (S61 hero + S62 split-screen) is documented; if it exceeds 1.5MB, document the mitigation plan
- [ ] Lighthouse Performance ≥ 90 on **first paint** unchanged from S61
- [ ] Source panes are syntax-highlighted, scrollable, and copy-to-clipboard works
- [ ] Live-render panes are sandboxed (CSP, no script execution from compiled output)
- [ ] All blocking bugs found in iOS/Android compilers during this sprint are fixed in-sprint

---

## Risks

1. **iOS/Android compilers may not compile to wasm32.** Mitigated by the pre-sprint spike. If the spike fails, the sprint is rescoped or paused.
2. **Bundle size growth.** Adding two more compilers to the lazy-loaded WASM blob may push it past 1MB. Mitigation: investigate splitting into multiple WASM modules (one per target) loaded on tab activation. Decide based on spike findings.
3. **Source-pane visual quality.** A flat unstyled SwiftUI source dump is worse than no demo. Reserve time for syntax highlighting (Shiki or Prism) and scrollbar polish.
4. **Reference IRs must compile cleanly on all 4 targets.** Some IR features (e.g., certain MediaNode loading strategies) may not have full parity across targets. Pick the 3 reference IRs to favor portability — keep them simple, validate cross-target before committing.

---

## Out of Scope (Defer)

- WebGPU pane (no marketing value, dropped permanently from this plan series)
- Per-page footer / sidecar metadata → S63
- Gallery expansion → S63
- Live device-frame previews around the iOS/Android source (cosmetic; nice but not core)
