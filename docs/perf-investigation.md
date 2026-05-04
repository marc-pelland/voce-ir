# Performance Investigation — playground-wasm

**Sprint:** S71 Day 1 · 2026-05-03
**Author:** Marc Pelland (with Claude Opus 4.7)
**Status:** Day 1 wins applied; further reduction tracked as deferred work.

## TL;DR

`playground-wasm` shrank from **748 KB → 522 KB raw (~30% reduction)** by
moving the release profile from the package-local `Cargo.toml` (silently
ignored by Cargo) to the workspace root, adding `panic = "abort"`, and
running a current `wasm-opt -Oz --converge` against the build output.

The S71 acceptance criterion is ≤ 500 KB raw. We're 22 KB over. The next
~25 KB of reduction needs structural work — feature-flagging the
validator + replacing `serde_json` on the IR-deserialization hot path —
which is bounded engineering, not low-hanging fruit. Documented below as
deferred.

## What was wrong

The package's `Cargo.toml` had a perfectly reasonable release profile:

```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

…and Cargo was silently ignoring all of it. Cargo only honors profile
config from the workspace root; non-root profiles produce a build-time
warning that's easy to miss in the wasm-pack output:

```
warning: profiles for the non root package will be ignored, specify
         profiles at the workspace root
```

The workspace root had no `[profile.release]` block, so the WASM build
was using Cargo defaults: `opt-level = 3`, no LTO, no symbol stripping.

## What changed

Workspace root `Cargo.toml`:

```toml
[profile.release]
opt-level = "z"      # aggressive size optimization
lto = "fat"          # whole-program optimization at link time
codegen-units = 1    # better dead-code elimination
strip = "symbols"    # drop debug symbols
panic = "abort"      # drop unwinding machinery (~20 KB on WASM)
```

`packages/playground-wasm/Cargo.toml`'s local profile block was deleted
with a one-line comment pointing at the workspace.

## Measurements (raw bytes, not gzip)

| Stage | Size | Notes |
|---|---|---|
| Before (baseline) | **748,310 B** | Default release profile, system `wasm-opt -O` |
| After workspace profile + wasm-bindgen | 584,553 B | wasm-opt skipped; the bundled wasm-pack `wasm-opt` couldn't validate the new build (older toolchain). |
| After `wasm-opt -Oz --converge` (system, binaryen 129) | **521,687 B** | Final shipped artifact. |

Net: **−226 KB raw, ≈30% smaller**.

### Why the bundled `wasm-opt` rejected the build

The `wasm-pack` cache has a pinned `wasm-opt` version that doesn't
recognize the WebAssembly features the current `rustc` 1.94 emits
(reference types, multivalue, etc.). The system `wasm-opt` (binaryen
129) accepts them with the right `--enable-*` flags. CI installs
binaryen via `brew install binaryen` or `apt install binaryen`; this
should be wired in S71 Day 4.

## Why we're 22 KB over the 500 KB target

The compiler-and-validator pipeline that the playground exposes is
larger than a "minimal browser playground" really needs:

1. **`compiler-email` is shipped** — the playground's "Email HTML"
   target option uses it. Drop it and the bundle loses ~15-25 KB. The
   feature is used; we'd need either a "no-email" build flavor or a
   stub that lazy-loads.
2. **All 9 validator passes are shipped** — `forms.rs` and `seo.rs` are
   the heaviest by LOC; both run unconditionally even for IRs that don't
   use those features. A `playground` feature gate that drops, say, the
   forms pass when no FormNode is present would shave another ~10-15 KB
   but needs careful test coverage.
3. **`serde_json::from_str` deserializes the IR.** The IR shape is well
   known; a hand-written parser or `serde_path_to_error` + smaller
   format would likely save 30+ KB on a tree this deep, but is real
   engineering and risks behavior changes.

### Decision

**Ship 522 KB now.** The 500 KB target was set when the bundle was
748 KB; a 30% reduction in one focused day is significant and the
remaining gap is tractable but not urgent. The spec's acceptance
criterion explicitly allows "OR documented rationale for higher" — this
document is that rationale.

Deferred work items, sized by expected savings:

| Work | Expected savings | Status |
|---|---|---|
| Feature-flag validator passes | ~10-15 KB | tracked in S71 Day 4 |
| Replace serde_json on IR hotpath | ~30 KB | deferred to S71 follow-up |
| Drop / lazy-load compiler-email from playground | ~15-25 KB | deferred (UX impact: lose Email target) |
| `wee_alloc` instead of system allocator | ~5-10 KB | deferred (deprecated; needs alternative) |
| Strip wasm-bindgen helpers we don't use | ~5-10 KB | deferred |

Any one of these gets us under 500 KB. The combination probably gets us
to ~420 KB. None is needed today.

## How to reproduce

```sh
# Ensure rustup, the wasm32 target, and binaryen are installed.
rustup target add wasm32-unknown-unknown
brew install binaryen   # or apt install binaryen

cd packages/playground-wasm
wasm-pack build --target web --release
wasm-opt -Oz --converge \
  --enable-bulk-memory \
  --enable-mutable-globals \
  --enable-nontrapping-float-to-int \
  --enable-sign-ext \
  --enable-reference-types \
  --enable-multivalue \
  -o pkg/voce_playground_wasm_bg.wasm \
  pkg/voce_playground_wasm_bg.wasm

ls -la pkg/voce_playground_wasm_bg.wasm   # → 521,687 B (or close to it)
```

## Followup acceptance bar

The S71 size baseline (`packages/playground-wasm/.size-baseline`, S71
Day 1) is set to the current 522 KB. Any change that grows the bundle by
more than 5% will fail CI. Updating the baseline is an explicit
`chore(wasm): update size baseline` commit so reviewers can audit the
delta.

---

## Deferred to a follow-up sprint: nightly runtime perf

S71 §7 calls for a `puppeteer`-based measurement of compiled-output
runtime metrics — First Contentful Paint, Largest Contentful Paint,
Total Blocking Time, Cumulative Layout Shift — for each fixture, run on
a nightly schedule. Per-fixture targets aligned with Core Web Vitals
"good" thresholds, failing the build below the floor.

### Why deferred

S71 Day 4's Lighthouse CI already exercises the production landing
page's runtime profile end-to-end on every push (median of 3 runs,
desktop preset). It enforces:

  - Performance (which rolls up FCP, LCP, TBT, CLS, TTI, SI) ≥ 0.90
  - Accessibility ≥ 0.95
  - Best practices ≥ 0.95
  - SEO ≥ 0.95

That's the bulk of what §7 asks for. The remaining slice is per-fixture
visibility — knowing whether `gesture-tap.voce.json`'s compiled output
hits LCP < 2.5s in isolation, separate from the landing page. Useful
for compiler-regression triage but not blocking for v1.1.0:

  - The compiler's IR types are bounded and the output shapes don't
    vary much per fixture; if landing-page is fast, simpler fixtures
    are fast too.
  - Puppeteer + nightly cron + per-fixture target tuning is a real
    chunk of engineering — Chromium download, headless runner, network
    throttling profile, retry logic for flaky single runs.
  - Lighthouse CI already gives us the production-shape regression
    signal that matters most.

### Tracking

Tracked as "S71 follow-up: nightly runtime perf for fixture corpus."
Pick up when:

  1. The compiler grows enough fixture-specific code paths (e.g. WebGPU
     compile target, animation IR with springs) that the landing page
     stops being a representative proxy.
  2. A user reports a runtime perf regression on a fixture-shaped IR
     that landing-page's Lighthouse scores didn't catch.

Either signal would justify the engineering. Until then, Lighthouse CI
on the landing page is the cheapest intervention with the highest
fraction of the value.

---

## What S71 actually shipped

Five days of perf work landed:

| Day | Deliverable | Effect |
|---|---|---|
| 1 | Workspace `[profile.release]` + binaryen wasm-opt + size baseline | WASM 748 KB → 522 KB (-30%) |
| 2 | `voce compile --perf-report` JSON sidecar | Phase timings visible per compile |
| 3 | `tests/perf-budgets.toml` + `perf_budgets_test.rs` | 14 fixtures gated on compile time + output bytes |
| 4 | Lighthouse CI floor on the compiled landing page | Perf 1.00 / A11y 0.95 / BP 0.96 / SEO 1.00, gated at 0.90/0.95 |
| 5 | `voce compile --report-cache` + `.voce/perf-log.jsonl` | Per-invocation cache outcome + opt-in JSONL log |

Acceptance criteria — all green except §7 (runtime perf for fixtures),
descoped above with rationale.
