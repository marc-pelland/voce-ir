# Site v2 Build Journal — S61 (Live Pipeline Hero)

A running log of findings, bugs, and design decisions encountered while building the self-demonstrating site (S61–S63). Append entries; do not delete. Each finding has an ID, the day it was found, a short description, the resolution applied (if any), and a follow-up ticket reference.

---

## Day 1 — Cache the 6 starter prompts

### F-001 · Validator output is summary-only — ~~partial~~ revisited (resolved)
**Found:** Day 1, while wiring `voce validate --format json` into the fixture build script.
**Initial reading:** The validator's JSON output is summary-only and S61's "9 passes light up in sequence" design needs per-pass timing/rule counts → motivated a planned `voce validate --verbose-passes` Rust change.
**Day 3 reality check:** The WASM `validate()` output already includes the originating pass on every diagnostic via the `pass` field (verified — `STR002 → "structural"`, `SEO001 → "seo"`, etc.). The canonical list of 9 passes is hardcoded in two places (`packages/validator/src/passes/mod.rs::all_passes` and `packages/site-hero/src/types.ts::VALIDATION_PASSES`); they match. So the front-end can already derive per-pass status (clean / N errors / M warnings) without any backend change.
**What `--verbose-passes` would have added:** per-pass timing in microseconds (visually irrelevant at 1s cinematic pacing) and an authoritative pass list (cheap CI lint covers drift instead).
**Resolution:** Skip the Rust change for S61. Day 3 visualization derives per-pass status from the existing diagnostic stream. Add a CI lint (Day 4) that asserts `types.ts::VALIDATION_PASSES` matches `passes/mod.rs::all_passes()` to catch any drift. **Cost saved: ~1 day of Rust + WASM rebuild.**
**Follow-up:** If a future demo (S62? S63?) needs real per-pass timing, revisit then. Not before.

### F-002 · Validator JSON leaks absolute filesystem paths
**Found:** Day 1, fixture build script.
**Description:** `voce validate --format json` includes a `file` field with the absolute path of the input. Bundling fixtures with a build host's path leaks that path to the CDN.
**Resolution:** `build-fixtures.mjs` strips the `file` field before writing the fixture.
**Follow-up:** Validator should emit relative paths, accept stdin (no path), or expose `--strip-path`. Low priority.

### F-003 · Compilation cache (S59) skews timing measurements
**Found:** Day 1, fixture build script — observed "✓ Cache hit:" lines on stderr from `voce compile`.
**Description:** Both fixture compilations measured 3ms, which is cache-warm time, not cold compilation. If the hero visualization shows "compile took 3ms" it implies live compilation runs that fast; in practice cold compilation is materially slower (S59 benchmarks: 209μs for landing-page IR, but those are also internal — end-to-end CLI cold path includes process spawn, FS, etc.).
**Resolution:** The hero must measure live in-browser compilation independently (Day 2 does — WASM `compile_dom` is called fresh in the visitor's browser, no cache involved). The cached-CLI timings stored in the fixture are kept as a comparison number but labeled "cached" in the UI to avoid confusion.
**Follow-up:** Optional: add `voce compile --no-cache` to make CLI-side cold-compilation measurement easier.

---

## Day 2 — Wire playground-wasm into site-hero, verify round-trip

### F-004 · WASM and CLI output shapes diverge
**Found:** Day 2, while wiring the WASM into `site-hero/src/main.ts`.
**Description:** `playground-wasm` and the `voce` CLI return different shapes for the same conceptual operations:

| Operation | CLI | WASM |
| --- | --- | --- |
| Validate | `{valid, errors:N, warnings:N, diagnostics[]}` (counts) | `{valid, errors[], warnings[]}` (arrays of diagnostics) |
| Compile  | raw HTML written to file | `{ok, html, sizeBytes, error?}` envelope |

This was not documented in any single place — discovered by reading the wasm-bindgen-generated `.d.ts` file.

**Resolution:** Site-hero defines `WasmCompileResult` and `WasmValidateResult` types in `src/types.ts` and unwraps the envelopes before comparing to cached output. The cached fixtures use the CLI shape; the live runtime uses the WASM shape; the round-trip verifier (`verify-roundtrip.mjs`) handles both.
**Follow-up:** Long-term, the two surfaces should converge on a single canonical shape. Documenting this divergence in the schema crate's error/result types is the right place to start. Not in scope for S61.

### F-005 · Cross-package WASM artifact duplication
**Found:** Day 2, package setup.
**Description:** `packages/site-hero/wasm/` is a 660 kB copy of `packages/playground/wasm/`. Both packages ship the same WASM blob.
**Resolution:** Accepted for S61. Disk is cheap; build correctness matters more than DRY here.
**Follow-up:** Promote `playground-wasm` build output to a workspace-shared location (e.g. a `packages/playground-wasm-pkg/` published artifact) and have both consumers import it. Worth doing before S62 adds two more compile targets to the WASM blob.

### F-006 · WASM ↔ CLI compilation is byte-for-byte identical
**Found:** Day 2, `verify-roundtrip.mjs` against both available fixtures.
**Description:** Live in-browser `compile_dom` produced HTML byte-for-byte identical to the CLI's `voce compile` output for both `hero-section` and `contact-form` fixtures. No codegen divergence between native and WASM builds of the `voce-compiler-dom` crate.
**Resolution:** None needed — this is the desired property. Recorded so we know this is an intentional invariant if it ever breaks later.
**Follow-up:** Wire `verify-roundtrip.mjs` into CI so any future divergence is caught at PR time. Day 4 work.

### F-007 · Vite emits a misleading "dynamically imported / statically imported" warning
**Found:** Day 2, first `vite build`.
**Description:** Vite warns that `fixtures/index.json` is both dynamically and statically imported. In fact `main.ts` imports `index.json` only statically; the dynamic-import template `await import(`../fixtures/${id}.json`)` causes Vite's scanner to include `index.json` in the dynamic-import set incidentally.
**Resolution:** Bundle output is correct (index.json is part of the main chunk; per-fixture files are split). Warning is cosmetic. Not suppressing it — doing so would mask real warnings of the same shape later.
**Follow-up:** None.

### Bundle measurements (S61 acceptance reference)

From `vite build` output (production, gzipped):

| Asset | Raw | Gzip | Loaded when |
| --- | --- | --- | --- |
| index.html | 2.85 kB | 1.21 kB | first paint |
| main bundle (`index-*.js`) | 6.44 kB | 2.79 kB | first paint |
| WASM glue (`voce_playground_wasm-*.js`) | 3.60 kB | 1.39 kB | first prompt click |
| WASM blob (`voce_playground_wasm_bg-*.wasm`) | 661 kB | **232 kB** | first prompt click |
| `hero-section` fixture | 4.35 kB | 1.93 kB | when prompt selected |
| `contact-form` fixture | 5.36 kB | 1.98 kB | when prompt selected |

**Initial paint cost: ~5.4 kB gzip** (index.html + main bundle + glue if eagerly loaded; current setup defers the glue too).
**Post-interaction cost: ~234 kB gzip** (mostly WASM, one-time).

Both budgets are met for S61's acceptance criteria. Re-measure on Day 4 after embedding into the landing page IR.

---

## Day 3 — Three-column layout, cinematic animation

### F-008 · Cinematic pacing decision
**Found:** Day 3, designing animation timing.
**Description:** User specified "around 1 second per pass" — deliberate but engaged. Designed the full sequence to total ~10.3s:

| Stage | Duration | What |
| --- | --- | --- |
| A — IR fade-in | 800ms | Cached IR JSON appears in column 1 |
| B — pre-pass pause | 200ms | Lets the eye settle before validation begins |
| C — per-pass tick | 9 × 900ms | Each pass: pulse-accent → result class. Total 8.1s |
| D — post-pass pause | 300ms | Verdict reveal |
| E — render | 600ms | Compiled DOM iframe fades in |

**Key honesty point:** the WASM pipeline runs **once at the start of `onPromptClick`** (~few ms total). The cinematic sequence paces the *display* of pre-computed results, not the execution. This is documented in `main.ts` and the build journal. The pipeline is real; the pacing is intentional.
**Skip controls:** dedicated "skip animation →" button + click-anywhere-on-the-columns area both abort the animation and snap to the final state.
**Follow-up:** Day 5 may want to compress this further (replays after first view could be instant; or motion-reduce setting could honor `prefers-reduced-motion`).

### F-009 · Skip-state recovery without controller state
**Found:** Day 3, while wiring the skip button.
**Description:** First implementation lost final pass result classes for passes that hadn't yet animated through their `running → ok/warn/err` transition when skip fired. The DOM was being treated as the source of truth, but un-animated passes had no class set yet, so they defaulted to "ok" — wrong for an invalid IR.
**Resolution:** Stash the final result class (`data-final`) and detail text (`data-detail`) on each `<li>` at run start, before any animation begins. Skip reads from there; no separate controller state needed. Animation transitions still happen on visible classes; skip just promotes everything to its final state in one tick.
**Follow-up:** None.

### F-010 · Inline CSS bundle delta
**Found:** Day 3, post-build.
**Description:** Inlining the visualization CSS into `index.html` pushed it from 2.85 kB raw → 7.72 kB raw (1.21 kB → 2.44 kB gzip). Vite does not currently extract critical CSS for a single-page bundle of this size; an external stylesheet would add a separate roundtrip without saving bytes.
**Resolution:** Keep CSS inline for now. Initial paint is 7.4 kB gzip total — still well under the 30 kB budget. Revisit if the site-hero starts having sub-pages.
**Follow-up:** None.

### Bundle measurements (Day 3 production build, gzipped)

| Asset | Day 2 | Day 3 | Δ |
| --- | --- | --- | --- |
| index.html | 1.21 kB | 2.44 kB | +1.23 kB |
| main bundle | 2.79 kB | 3.61 kB | +0.82 kB |
| WASM glue | 1.39 kB | 1.39 kB | 0 |
| WASM blob (lazy) | 232 kB | 232 kB | 0 |
| per-fixture chunks | ~1.95 kB | ~1.95 kB | 0 |
| **Initial paint** | **5.4 kB** | **7.4 kB** | **+2 kB** |

Within budget. Round-trip verifier (`verify-roundtrip.mjs`) re-run after Day 3 changes — both fixtures still match byte-for-byte.

---

## Day 4 — (pending)

_Pending: integrate into landing page IR · CI hooks for round-trip + pass-list drift._

