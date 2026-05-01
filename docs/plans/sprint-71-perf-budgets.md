# Sprint 71 — Performance + Bundle Budgets

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Convert performance from a "we measured it once" property (S59) into a load-bearing CI gate. Hard size and time budgets per artifact + per fixture, enforced on every PR. The WASM blob trims from 704 kB to a documented target. The compiler emits compile-time perf reports per build. No regression goes unnoticed.

**Depends on:** S59 (criterion benchmarks shipped). Independent of S65–S70.

---

## Motivation

S59 established benchmarks (209 µs landing-page compile, 4.4 µs minimal). Today's WASM blob is 704 kB raw / 244 kB gzip — large for a library people would ship to a homepage. The compiler emits no perf metadata in its output, so a regression that doubles compile time only surfaces if someone reruns benchmarks manually. This sprint moves perf from "watched" to "guarded."

---

## Deliverables

### 1. WASM size reduction pass

Target: `playground-wasm` from 704 kB → ≤ 500 kB raw (≤ 175 kB gzip), or document why we can't.

Investigation steps:
- `wasm-objdump -h` to identify largest sections
- `cargo bloat --target wasm32-unknown-unknown --release -p voce-playground-wasm` to find the heaviest functions
- Likely candidates: `serde_json` deserialization paths, `regex`-style passes in validator, full string formatting in error paths
- Strategies:
  - `wasm-opt -Oz` (already on); review settings
  - Replace `serde_json` with `simd-json` or hand-written deserializer for the IR shape
  - Feature-flag heavy passes: a "playground" feature that drops adapter code, font pipeline, etc.
  - Strip debug symbols even harder (`strip = "symbols"` already set; investigate further)
- Document findings + decisions in `docs/perf-investigation.md`

### 2. Compiler perf reports

`voce compile` gets a `--perf-report` flag that emits a JSON sidecar:

```json
{
  "input_bytes": 7842,
  "output_bytes": 14062,
  "total_ms": 12.4,
  "phases": {
    "parse": 1.2,
    "validate": 2.1,
    "ingest": 0.8,
    "lower": 1.4,
    "emit_html": 6.1,
    "minify": 0.8
  },
  "node_count": 47,
  "compiled_at": "..."
}
```

Useful for power users; required for the next deliverable.

### 3. Per-fixture perf budgets

A new `tests/perf-budgets.toml`:

```toml
[fixture."01-text-only"]
max_compile_ms = 5
max_output_bytes = 4000

[fixture."06-form-basic"]
max_compile_ms = 15
max_output_bytes = 8000

[fixture."15-full-landing"]
max_compile_ms = 50
max_output_bytes = 30000
```

A new test in `tests/perf_budgets_test.rs` runs each fixture, compares to its budget, fails on regression. Budgets sized at ~1.5x current measured to allow normal noise.

### 4. CI bundle-size gate

A new CI step that builds `playground-wasm` and compares the output size to a tracked baseline (`packages/playground-wasm/.size-baseline`). Fails if growth exceeds 5%. Updating the baseline is an explicit PR action with a commit message tag (`chore(wasm): update size baseline`).

### 5. Lighthouse CI for the production demo

Building on S58: every PR rebuilds the production landing site and runs Lighthouse against it (via `@lhci/cli`). Hard floors:
- Performance ≥ 90
- Accessibility ≥ 95
- Best Practices ≥ 95
- SEO ≥ 95

Fails the build below the floor. Configurable per-environment (mobile vs desktop scoring).

### 6. Compile-cache effectiveness telemetry

S59's compilation cache is in place but its hit rate is invisible. Add lightweight metrics:
- `voce compile --report-cache` prints `cache hit / miss / invalidate` counts
- Optional opt-in `.voce/perf-log.jsonl` records every compile with cache outcome
- Use to tune cache eviction strategy if needed

### 7. Runtime perf for compiled output

Beyond compile-time: are the *outputs* fast?

- For each fixture's compiled DOM, run a `puppeteer`-based measurement of:
  - First Contentful Paint
  - Largest Contentful Paint
  - Total Blocking Time
  - Cumulative Layout Shift
- Set per-fixture targets aligned with Core Web Vitals "good" thresholds
- Report failures in CI

This is heavier than the bundle-size gate; runs on a nightly schedule rather than every PR.

---

## Acceptance Criteria

- [ ] `playground-wasm` ≤ 500 kB raw OR documented rationale for higher
- [ ] `voce compile --perf-report` produces structured JSON output
- [ ] `tests/perf-budgets.toml` covers all S68 cross-target fixtures
- [ ] `tests/perf_budgets_test.rs` runs and gates regressions
- [ ] CI bundle-size gate active with 5% threshold
- [ ] Lighthouse CI runs on PR; floors enforced
- [ ] `voce compile --report-cache` available
- [ ] Nightly runtime-perf job runs against compiled outputs
- [ ] `docs/perf-investigation.md` documents findings from the WASM size reduction pass

---

## Risks

1. **WASM size reduction might require sacrificing features.** A feature-flagged "playground" build that drops some passes is a real architectural change. Time-box and pick the lowest-cost wins; document deferred opportunities.
2. **Lighthouse score volatility.** Mobile scores are notoriously noisy. Run 3 times per PR and take the median; allow a small variance band.
3. **Per-fixture budgets bake in current perf as the floor.** If a future change makes everything 30% faster but trips the upper bound somewhere, that's a false positive. Budgets are upper bounds, not equality assertions.
4. **Adding perf instrumentation to the compiler hot path can itself slow it down.** Use feature flags so default builds aren't paying the cost.

---

## Out of Scope

- Real-user monitoring (RUM) — that's an opt-in in the deployed page, not the compiler
- Compiler parallelization (per-file or per-pass parallel)
- WebGPU runtime perf — its own beast, separate sprint
- AI bridge latency optimization (LLM provider's problem)
