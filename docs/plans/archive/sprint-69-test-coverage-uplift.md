# Sprint 69 — Test Coverage Uplift

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Take the test suite from 172 unit-and-snapshot tests to a comprehensive coverage posture: property-based tests for the validator, snapshot tests across every compile target, integration tests for the full pipeline (prompt → IR → validate → compile → deploy), and a per-crate coverage report wired into CI with a hard floor that PRs cannot regress.

**Depends on:** the Rust workspace (S01–S20). Independent of S65–S68; runs in parallel.

---

## Motivation

172 tests sounds like a lot but the math doesn't reassure: the schema has 27 union variants, 12 .fbs files, ~100 FlatBuffers types, 9 validation passes, 46 error codes, 7 compile targets, 5 deployment adapters. Coverage almost certainly has gaps — the S61 form-styling regression and the F-011 pass-list drift bug both passed every existing test. This sprint introduces structural ways to find what's untested and raises the floor.

---

## Deliverables

### 1. Property-based testing for the validator

Adopt `proptest` for the validator. Add property tests that:

- Generate arbitrary IRs (small fuzzed shapes) and assert validate() never panics
- Generate IRs satisfying schema invariants (using custom proptest strategies) and assert specific properties — e.g., "an IR with all required SemanticNode references resolves cleanly through the references pass"
- Round-trip property: `for any IR, JSON ↔ binary ↔ JSON is identity`
- Determinism property: `for any IR, validate(ir) is deterministic across runs`

Initial target: 15 property tests covering structural, references, a11y, security passes. Scale to all 9 over time.

### 2. Snapshot coverage for every target

Today snapshot tests cover DOM and partial coverage of email, swiftui, compose. Add:

- `compiler-webgpu` snapshots for at least 5 fixtures
- `compiler-wasm` snapshots for at least 5 fixtures  
- `compiler-hybrid` snapshots for at least 5 fixtures
- Existing target snapshots refreshed to use the cross-target fixtures from S68 (single source of truth)

### 3. Full-pipeline integration tests

A new `tests/pipeline/` directory with end-to-end scenarios that exercise the *entire* pipeline:

- `pipeline_simple_landing.rs` — fixture IR → validate → compile DOM → assert HTML structure → run minimal HTML parser, check semantic correctness
- `pipeline_form_with_action.rs` — IR with FormNode + ActionNode → compile → check generated JS handler → invoke a stub server and assert action posts correctly
- `pipeline_state_machine.rs` — IR with StateMachine → compile → simulate state transitions in a JS sandbox → assert reactive updates

These are slower than unit tests but catch class-of-bugs unit tests cannot.

### 4. Coverage tooling

Wire `cargo-llvm-cov` into the workspace:

- Local: `cargo llvm-cov --workspace --html` produces a coverage report at `target/llvm-cov/html/`
- CI: a new `coverage` job runs on every PR, uploads the report as an artifact, and fails if line coverage drops more than 2 points relative to `main`
- Floor: set initial floor at the *current* measurement (let's say it's around 60% — measure first), and grow it by ~2 points per quarter

Per-crate breakdown so contributors can see exactly which file regressed.

### 5. Mutation testing pilot

Run `cargo-mutants` against `voce-validator` and `voce-compiler-dom`. Mutation testing flags places where a code change wouldn't break any test — those are the gaps. One-time investigation in this sprint:

- Pick the 5 most concerning surviving mutations
- Write tests that catch them
- Document remaining surviving mutations in `docs/test-coverage-gaps.md` with reasoning

Don't make mutation testing a CI gate (too slow), but document a quarterly review cadence.

### 6. Test naming + organization audit

Inconsistent test naming across crates today (some have `test_X`, some `X_test`, some inline). Pick one convention (`X` plain or `it_does_X`) and migrate. Add a `tests/CONTRIBUTING.md` describing where to put a new test depending on what it exercises.

### 7. Flaky test sentinel

Add a CI step that runs the test suite 3 times on a randomly sampled PR (1 in 10) and flags any test that fails non-deterministically. Reports go to a tracking issue.

---

## Acceptance Criteria

- [ ] `proptest` integrated; ≥ 15 property tests across the validator
- [ ] Every compile target has ≥ 5 snapshot tests
- [ ] ≥ 3 full-pipeline integration tests passing
- [ ] `cargo llvm-cov` runs locally and in CI
- [ ] Coverage report uploaded as CI artifact
- [ ] Coverage floor enforced with -2pt regression threshold
- [ ] `cargo-mutants` run once on validator + compiler-dom; top 5 surviving mutations addressed
- [ ] `tests/CONTRIBUTING.md` documents the testing strategy
- [ ] Test naming convention picked and applied
- [ ] Flaky-test sentinel running on sampled PRs

---

## Risks

1. **CI time blowup.** Property tests + full-pipeline tests + coverage instrumentation can each add minutes. Budget: full CI time stays ≤ 8 minutes on standard runner. Move expensive tests to a nightly-only schedule if needed.
2. **Coverage floor may be embarrassing.** If actual coverage is 40%, the headline number is uncomfortable. Better to know than not.
3. **Property tests can find existing bugs.** Good. Budget half the sprint for fixing what they surface.
4. **`cargo-llvm-cov` requires nightly toolchain on some setups.** Use the stable variant or pin a known-good toolchain version.

---

## Out of Scope

- TypeScript test coverage (cli-chat, mcp-server, ai-bridge) — separate effort, less critical for "system solidity"
- Front-end visual regression (pixel diffs)
- Load testing / fuzzing the WASM module
- Test parallelization beyond what `cargo test` does natively
