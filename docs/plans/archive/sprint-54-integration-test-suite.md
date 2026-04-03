# Sprint 54 — Integration Test Suite

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Build comprehensive cross-target integration tests that compile the reference landing page to all 7 targets and verify output correctness. Add end-to-end tests covering prompt-to-output. Bring total test count from 73 to 150+.

**Depends on:** All 7 compiler targets operational, Sprint 51 (real image pipeline)

---

## Deliverables

- New test directory: `tests/integration/` with per-target test modules
- **Cross-target matrix tests:** compile the reference landing page IR to each of the 7 targets (DOM, WebGPU, WASM, Hybrid, iOS, Android, Email), verify each output is structurally valid
- **DOM output validation:** parse compiled HTML with `scraper` crate, assert correct element hierarchy, ARIA attributes present, responsive images in `<picture>`, styles inlined
- **Email output validation:** verify inline CSS only (no `<style>` blocks), table-based layout, no unsupported elements per email client matrix
- **iOS/Android output validation:** parse SwiftUI/Jetpack Compose output, verify accessibility modifiers present, navigation structure correct
- **End-to-end pipeline tests:** IR JSON string -> deserialize -> validate -> compile -> verify output structure (5 test cases: landing page, form, dashboard, article, error page)
- **Negative tests:** malformed IR, missing required fields, invalid references, circular state machines — verify validator catches each and compiler rejects gracefully
- **Snapshot tests:** `insta` snapshots for compiled output of 10 canonical IR fragments, one per node type
- **Performance regression tests:** assert compilation of reference landing page completes in under 500ms
- **Test fixtures:** `tests/fixtures/` directory with 20+ IR files covering every node type combination
- CI configuration: `cargo test --workspace` runs all tests, GitHub Actions workflow

## Acceptance Criteria

- [ ] Total test count across workspace is 150 or more (`cargo test --workspace 2>&1 | grep "test result"`)
- [ ] Reference landing page compiles successfully to all 7 targets without errors
- [ ] DOM output contains valid HTML5 (verified by parsing, not string matching)
- [ ] Email output contains no `<div>` elements (table-based layout enforced)
- [ ] Every `SemanticNode` in test IR produces corresponding ARIA attributes in DOM output
- [ ] All 20+ negative test cases produce specific, helpful error messages (not panics)
- [ ] `insta` snapshots exist for 10 canonical IR fragments
- [ ] Compilation performance test asserts < 500ms for reference landing page
- [ ] `cargo test --workspace` completes in under 60 seconds total
- [ ] Zero flaky tests — all pass deterministically on 3 consecutive runs
- [ ] `cargo clippy --workspace -- -D warnings` passes clean
