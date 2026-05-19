# Sprint 68 — Cross-Target Parity

**Phase:** 7 — Production Readiness
**Status:** In progress

## Implementation Status

- ✅ **Slice 1 (oracle):** `SemanticSummary` promoted into the validator
  library (`packages/validator/src/semantic_summary.rs`) with
  `from_ir` (the contract) + `from_html` (observed) + unit tests.
  `tests/cross_target_parity.rs` verifies the **DOM oracle** preserves
  IR semantics (heading order, form fields, named media, interactive
  floor, landmark superset) across the 13-fixture corpus reused from
  `tests/fixtures/` (no duplicate `tests/cross-target/` dir — DRY).
  All green on first run; the reference compiler is faithful.
- ✅ **Slice 2 (HTML family):** Hybrid asserted at the full oracle
  contract (verified identical to DOM across the corpus). Email
  asserted on its required contract (heading order + named media);
  forms/landmarks classified as medium-degradation (◐). WebGPU
  classified ✗-via-this-lens (GPU-painted behind a fixed shell — needs
  an a11y-tree extractor, not `from_html`). `docs/compatibility-matrix.md`
  (Deliverable 4) generated from the real run, with a profile model so
  divergence is classified, never silent. A diagnostic dump test aids
  inspection. **Deliverable-5 finding logged:** Email emits zero
  anchors for `links-and-nav` (4 links) — flagged ⚠ as a likely
  flattening bug (email *can* do links), not asserted as a gate yet.
- ✅ **Deliverable 5 (Email-anchor ⚠ resolved):** the Email compiler
  silently dropped `href` on TextNode and Surface. Both now emit
  email-safe anchors (`target`/`rel` matching DOM; Surface wraps the
  block in a `display:block` anchor). `links-and-nav` 0→4 anchors.
  This also surfaced a model flaw: `interactive_count` conflated links
  and gestures, which would have hidden that email legitimately can't
  do JS gestures. Split into `link_count` + `gesture_count`; Email
  profile now asserts link parity (✓) while gesture is documented ◐.
  Matrix updated (Interactive row → Links + Gestures).
- ✅ **Deliverable 6 (CI gate):** dedicated `cross-target-parity` job
  in `.github/workflows/ci.yml` (`needs: check`) runs the verifier in
  isolation on every PR and uploads the divergence dump as the
  `cross-target-divergence` artifact. A regression in a ✓/required-◐
  cell fails this job loudly instead of being buried in the 39-suite
  workspace run.
- ⏳ **Remaining:** Slice 3 only — SwiftUI / Compose language-specific
  extractors; WebGPU a11y-tree extractor; WASM (logic-only — partial).
  The HTML-family foundation (DOM/Hybrid/Email) is complete and gated.
- This slice also unblocks **S82 D7/D9** (axe-core CI runs on this
  corpus) and the **S91** conformance kit (which promotes this exact
  extractor to a normative, portable algorithm).

---
**Goal:** Verify that all 7 compile targets (DOM, WebGPU, WASM, Hybrid, iOS SwiftUI, Android Compose, Email HTML) produce semantically equivalent output for the same IR. Today the iOS/Android/WebGPU compilers exist but their output is rarely cross-checked against DOM. Silent divergence is a real risk for the project's "one IR, every platform" pitch — this sprint closes it with a parity test matrix and a documented compatibility table.

**Depends on:** all 7 compilers (S20, S31–S34, S45–S46, S49). Independent of S65–S67 / S69; can run in parallel.

---

## Motivation

The pitch is "one IR, every platform." The project ships 7 compilers. Today there's no test that takes a single IR and verifies the output across all 7 produces the same semantic page. Each compiler has its own snapshot tests, but those are isolated — a feature can render correctly on DOM and silently mis-render on iOS without any test catching it. The S62 multi-target split-screen sprint will need this guarantee to be load-bearing; building the parity matrix now is also the foundation S62 stands on.

---

## Deliverables

### 1. Cross-target test fixture set

A new `tests/cross-target/` directory with 12-15 IR fixtures chosen to exercise the surface area each compiler covers:

- `01-text-only.voce.json` — single TextNode with heading hierarchy
- `02-flex-row.voce.json` — Container with Row direction
- `03-flex-column.voce.json` — Container with Column direction
- `04-grid.voce.json` — Container with Grid layout
- `05-image-with-alt.voce.json` — MediaNode + SemanticNode
- `06-form-basic.voce.json` — FormNode with text/email/textarea fields
- `07-form-validation.voce.json` — FormNode with required/email/min-length
- `08-state-machine.voce.json` — StateMachine with 3 states + transitions
- `09-gesture-tap.voce.json` — GestureHandler with keyboard equivalent
- `10-animation.voce.json` — AnimationTransition with ReducedMotion
- `11-theme-applied.voce.json` — ThemeNode with palette + typography
- `12-i18n.voce.json` — LocalizedString + MessageCatalog
- `13-nested-scroll.voce.json` — ScrollBinding + nested Containers
- `14-data-fetch.voce.json` — DataNode + ContentSlot
- `15-full-landing.voce.json` — composite (covers most of the above)

Each fixture validates cleanly and covers at least one feature. The set should be the minimum that exercises every node type union variant.

### 2. Per-target expected-output specs

For each fixture, an `expected/<target>/` directory containing:

- The compiled artifact (HTML / SwiftUI / Compose / etc.)
- A `semantic-summary.json` describing the *semantic* structure (heading hierarchy, interactive elements, form fields, ARIA roles, color tokens applied) — independent of representation

The semantic summaries must match across targets. The compiled artifacts will not match (different languages); the summaries must.

### 3. Cross-target verifier (`tests/cross_target_test.rs`)

A Rust integration test that:

1. Loads each fixture
2. Compiles via every available target compiler
3. For each compiled artifact, derives a `SemanticSummary` (counts, structure, semantic landmarks)
4. Asserts all 7 summaries are equivalent (same number of headings, same heading levels in same order, same number of interactive elements, same form field count and types, same number of images with alt text, etc.)

When summaries diverge, the test prints a diff explaining which target dropped/gained which feature.

### 4. Compatibility matrix doc

`docs/compatibility-matrix.md` — generated from the test run. For each fixture × target cell:

- ✓ Full parity
- ◐ Degraded (target supports semantically but with limitations — e.g., Email HTML has no JS, so animations become CSS-only)
- ✗ Not supported (e.g., 3D scenes on Email)
- ⚠ Silent gap (compiler emits something but parity check fails — bug)

Every ⚠ becomes a fix-in-sprint ticket. Every ✗ becomes a documented limitation.

### 5. Bug fixes for divergences

Whatever the parity matrix surfaces. Budget half the sprint for fixing real divergences. Examples likely to surface:

- iOS/Android compilers may not handle certain Container alignment values
- Email HTML can't represent state machines (warn/skip is correct; silent skip is not)
- WebGPU may render text without the typography scale applied
- Animation timings may use different curves on iOS vs DOM

For each: either fix the compiler, or document the limitation in the matrix and emit a compile-time warning when the affected feature is encountered.

### 6. CI integration

A new job `cross-target-parity` runs the full test matrix on every PR. Fails on any new ⚠. Existing ◐ and ✗ are baseline, recorded in `docs/compatibility-matrix.md` and only updated by explicit PR.

---

## Acceptance Criteria

- [ ] 12+ cross-target fixtures committed to `tests/cross-target/`
- [ ] Every fixture compiles cleanly on every applicable target
- [ ] `tests/cross_target_test.rs` runs and passes (with documented baseline gaps)
- [ ] `docs/compatibility-matrix.md` exists and is generated from the test run
- [ ] At least 5 silent divergences (⚠) found and fixed in-sprint, OR documented as ✗ with rationale
- [ ] CI `cross-target-parity` job present and gating on regressions
- [ ] Compatibility matrix linked from README and `docs/`

---

## Risks

1. **iOS/Android compilers may not even compile to wasm32.** If the S62 spike fails this, those targets can still be tested via native binaries — slower in CI but workable.
2. **Semantic summary is itself a compiler.** Risk of bugs in the summary extractor masking real divergences. Mitigation: cross-check against a hand-written summary for each fixture.
3. **Fixes may require schema changes.** If a node type is missing fields needed for parity, that's an S72 (schema completeness audit) finding, not a quick fix here. Document and defer.
4. **Email HTML genuinely cannot do state machines.** Some divergences are physical, not bugs. The matrix needs to distinguish "limitation of the medium" from "compiler bug."

---

## Out of Scope

- Visual regression (pixel-diffing) across targets — semantic parity only here
- Performance parity (a slow Compose output is still parity if semantics match)
- New target compilers
- Iframe-based live preview of all 7 targets — that's S62
