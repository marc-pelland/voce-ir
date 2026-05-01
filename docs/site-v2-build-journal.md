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

## Day 4 — Integration, CI hooks, drift bug found

### F-011 · Pass-list drift bug found by writing the lint check
**Found:** Day 4, while implementing `verify-pass-list.mjs`.
**Description:** `types.ts::VALIDATION_PASSES` contained 3 names that did not match the engine's canonical names, plus a different ordering. Specifically:

| Front-end (wrong) | Engine (right) |
| --- | --- |
| structural-completeness | structural |
| reference-resolution | references |
| motion-safety | motion |

Order also disagreed: front-end had `motion-safety` 5th, engine has `motion` last (9th). The front-end's `groupByPass()` logic compares incoming diagnostic `pass` strings (which come from the engine) against this constant — so for any IR with diagnostics, all rows would have been silently mis-bucketed. Both currently-cached fixtures are clean (no diagnostics), so this never manifested in Days 1–3.
**Resolution:** Updated `types.ts::VALIDATION_PASSES` to match the engine exactly. Added `verify-pass-list.mjs` which parses `passes/mod.rs::all_passes()` execution order and each pass file's `fn name()` literal, then asserts equality. Wired into CI in the new `frontend-verify` job.
**Probe:** ran the WASM `validate()` against an intentionally broken IR; diagnostics returned `pass: "structural"` and `pass: "seo"`, both now in the front-end's `VALIDATION_PASSES`. No "unknown" passes after the fix. Verified.

### F-012 · Fixture determinism for CI diff check
**Found:** Day 4, designing the CI verification step.
**Description:** Original fixture format included `generatedAt` timestamps and per-build wall-clock `timings` (validateMs, compileMs). The CI plan to fail-on-drift via `git diff --quiet` after fixture regeneration would have produced a false positive on every run.
**Resolution:** Dropped `generatedAt` and `timings` from both `Fixture` and `FixtureIndexEntry`. The display values they fed (cached vs. live timing comparison) were already removed in the Day 3 UI overhaul, so no functional loss. Verified determinism: `build-fixtures.mjs` produces identical output across consecutive runs.

### F-013 · Landing page integration: marker TextNode + post-compile splice
**Found:** Day 4, designing the integration approach.
**Description:** The schema has no "external island" or "raw HTML" node type. Adding one for a single demo would be over-engineering (S61 Risk 3 anticipated this). The compiler also doesn't emit `id` or `class` attributes from `node_id`, so a known IR node can't be located in compiled HTML by selector.
**Resolution:** Replace the hero `Container` in `landing.voce.json` with a single `TextNode` whose content is `__VOCE_HERO_SLOT__`. The compiler emits this verbatim as `<p>__VOCE_HERO_SLOT__</p>`. The build script (`build-landing.mjs`) regex-finds this element in the IR-compiled body, splits the body innerHTML into "above marker" (nav) and "below marker" (pipeline + features + CTA + footer), then splices both halves into `<!-- VOCE_LANDING_TOP -->` and `<!-- VOCE_LANDING_BOTTOM -->` markers in site-hero's built `dist/index.html`.

**Resulting page structure:**
```
<body>
  <!-- IR-compiled: nav (role="navigation") -->
  <div class="wrap">
    <!-- site-hero: header, status, prompts, columns (role="main"), skip btn -->
  </div>
  <!-- IR-compiled: pipeline section, features, CTA, footer (role="contentinfo") -->
  <script type="module" src="/assets/index-*.js"></script>
</body>
```

`role="main"` was moved from IR (`sem-main` on the deleted hero) to site-hero's `.columns` div, since the live demo is now the main content. The validator emits an SEO003 warning (page has no h1 heading) because the IR no longer contains an h1 — the live hero's h1 is HTML-side, invisible to the validator. Acceptable warning; not blocking.
**Output:** `packages/site-hero/dist-integrated/index.html` (12,961 bytes). Smoke-tested with `vite preview` — homepage serves 200 text/html, WASM asset serves 200. End-to-end integration works at the HTTP level. Visual review pending.
**Follow-up:** None blocking. Future schema sprint may consider an `ExternalIsland` or `RawHtml` node if more islands appear.

### F-014 · Existing `examples/production/dist/index.html` is hand-styled, not IR-compiled
**Found:** Day 4, comparing fresh `voce compile` output against the shipped dist.
**Description:** The shipped `examples/production/dist/index.html` (from S58) uses semantic CSS classes like `<section class="hero">`, `.feat`, `.bottom-cta` — none of which the current `voce compile` produces. Fresh compilation of `landing.voce.json` emits inline `<div style="...">` elements. The two outputs diverge significantly. Either S58's "no hand-edited HTML in shipped output" claim was inaccurate, or there's a different compilation path that's been removed since.
**Resolution:** Not in scope to fix S58's existing dist. The S61 integration writes to `dist-integrated/`, leaving the historical dist untouched. The shipped `voce-ir.xyz` will swap to `dist-integrated/` when this work deploys.
**Follow-up:** Consider a follow-up sprint to either (a) extend the compiler to emit semantic classes from `node_id`, or (b) update S58's claim to acknowledge the static dist was hand-finished. Out of scope for S61–S63.

### CI wiring

`.github/workflows/ci.yml` got a new `frontend-verify` job, depends on `check`. Steps:
1. Build voce CLI (release)
2. `npm install` site-hero
3. `node verify-pass-list.mjs` — pass-list drift check
4. `node build-fixtures.mjs` — regenerate against current toolchain
5. `node verify-roundtrip.mjs` — WASM ↔ cached HTML byte-for-byte
6. `git diff --quiet packages/site-hero/fixtures/` — fail if regeneration produced changes (catches schema/compiler/validator drift)

### Bundle measurements (Day 4 integrated build)

| Asset | Size |
| --- | --- |
| `dist-integrated/index.html` | 12,961 B |
| └─ above-slot (nav) | 409 B |
| └─ site-hero scaffold + CSS | ~7,770 B |
| └─ below-slot (pipeline + features + CTA + footer) | 4,778 B |
| `assets/index-*.js` (main) | 8.56 kB / 3.54 kB gzip |
| `assets/voce_playground_wasm_bg-*.wasm` (lazy) | 661 kB / 232 kB gzip |
| Per-fixture chunks (lazy) | ~5 kB each / ~1.9 kB gzip |

Initial paint cost (HTML + main bundle, gzipped): ~6.5 kB. WASM blob loads on first prompt click. Within S61 acceptance budget.

### Out of scope, deferred

- Deploy to staging — no hosting credentials configured, would need user action
- Fresh visual review — needs a browser; deferred to user
- IR validator emitting per-pass JSON (`--verbose-passes`) — defer indefinitely (F-001 closure)

---

## Day 5 — A11y baseline, copy refinement, reduced-motion

Visual polish that requires browser feedback is deferred until the user reviews Day 4's integrated build. This day's work covers the changes that are objectively improvements regardless of how the page currently looks.

### F-015 · prefers-reduced-motion honored
**Found:** Day 5, accessibility pass.
**Description:** Animation timing was hardcoded `setTimeout` constants. Visitors with `prefers-reduced-motion: reduce` would still get the 10-second cinematic sequence even though their OS asked for less motion.
**Resolution:** `main.ts` reads `window.matchMedia("(prefers-reduced-motion: reduce)").matches` once at module load and uses a 10ms-per-stage timing config when set. Effectively snaps to final state without skipping the pipeline run. CSS @media rule also collapses transitions and the running-pulse animation to ~0ms in that case.

### F-016 · Real `<main>` element instead of `<div role="main">`
**Found:** Day 5, landmark audit.
**Description:** Used `role="main"` on a `<div>` for the visualization columns. Native `<main>` is semantically richer and ARIA-validators prefer it.
**Resolution:** Promoted `<div class="columns" role="main">` → `<main class="columns" aria-label="Voce IR live pipeline">`. No CSS changes needed (class selector still hits).

### F-017 · Skip-link for keyboard users
**Found:** Day 5, keyboard nav check.
**Description:** Visitors tabbing into the page would have to tab through every prompt button to reach the demo content. No way to bypass.
**Resolution:** Added `<a class="skip-link" href="#columns">Skip to live demo</a>` as the first focusable element. Hidden off-screen via `transform: translateY(-200%)` until focused; slides into view on `:focus`. Standard skip-link pattern.

### F-018 · Focus-visible ring
**Found:** Day 5, keyboard nav check.
**Description:** No focus indicator on prompt buttons or skip button, so keyboard users had no visual feedback during navigation.
**Resolution:** Added a global `:focus-visible` rule with the accent color outline. `:focus { outline: none }` strips the default browser ring; `:focus-visible` only fires for keyboard navigation, so mouse users don't see rings on click.

### F-019 · Header copy refined for landing-page voice
**Found:** Day 5, copy review.
**Description:** Header read "Voce IR — live pipeline · site-hero · S61 day 3" — dev-tool framing left over from the Day 2 scaffold. Inappropriate for a marketing landing page.
**Resolution:** Changed to the project's tagline: "The code is gone. The experience remains." with subtitle "live pipeline · runs in your browser." Removed dev-build framing. The h1 now also serves as the page's primary heading for SEO.

### F-020 · Status announcements via `role="status"` + `aria-live`
**Found:** Day 5, screen-reader check.
**Description:** The "WASM ready · 2 of 6 prompts cached" message updates via DOM mutation but had no live-region announcement, so screen-reader users wouldn't hear the state change.
**Resolution:** Added `role="status" aria-live="polite"` to the status element. Also added `role="group" aria-label="Starter prompts"` to the prompt-buttons container.

### A11y self-audit results

| Check | Result |
| --- | --- |
| `<h1>` count | 1 ✓ |
| `<h2>` count | 2 (features section, bottom CTA) ✓ |
| `<h3>` count | 6 (feature cards) ✓ |
| `<main>` count | 1 ✓ |
| `role="navigation"` count | 1 (IR-compiled nav) ✓ |
| `role="contentinfo"` count | 1 (IR-compiled footer) ✓ |
| Skip link present | ✓ |
| Iframe has `title` | ✓ ("Compiled DOM output") |
| All buttons have accessible name | ✓ |
| Status region is announced | ✓ |

Standard browser-side checks (Lighthouse/axe) still need to run — a11y above is a static-source review. Day 5 will close when user runs Lighthouse on the integrated build.

### Bundle delta (Day 5 integrated build)

| Asset | Day 4 | Day 5 |
| --- | --- | --- |
| `dist-integrated/index.html` | 12,961 B | 14,062 B (+1.1 kB CSS + skip-link + ARIA) |
| main bundle (`index-*.js`) | 8.56 kB / 3.54 kB gzip | 8.71 kB / 3.60 kB gzip |

Both verifiers (`verify-pass-list.mjs`, `verify-roundtrip.mjs`) re-run clean.

### Visual polish deferred to user feedback

These changes are gated on browser review of the integrated build:
- Visual integration between site-hero's dark theme and the IR-compiled content below (typography/density mismatch potential)
- Whether the column proportions read well at common breakpoints (1280, 1024, 768, 375)
- Whether the cinematic pacing feels right at production density
- Whether the bottom IR-compiled content feels cohesive with the visualization above
- Lighthouse score (Performance + Accessibility + SEO)

User opens `npm run serve:integrated` → http://localhost:5175 → reports findings → Day 5 reopens for the visual portion.

---

## Day 5 (continued) — Form regression fix + compiler change

User reviewed the integrated build in the browser and reported the contact-form fixture renders as a "plain and boring form" — browser-default unstyled inputs/buttons. Investigation: real compiler quality bug, not a one-off. Decision: fix at compiler level (benefits every future form), not in the one IR.

### F-021 · Compilation cache poisons fixture regeneration after compiler changes
**Found:** Day 5 (continued), while regenerating fixtures after the form-CSS compiler change.
**Description:** `voce compile` writes its output through a content-addressed cache (S59) keyed on input IR JSON. The cache key does NOT include compiler version or compile-options. After changing the compiler (adding form CSS), running `build-fixtures.mjs` produced fixtures with the old (pre-change) HTML because every IR hit the cache. WASM-side compilation produced the new HTML, so round-trip verification immediately diverged: `live=3656B cached=1870B`. Symptoms initially confusing because the divergence offset (741) showed CSS ordering changes that weren't part of my edit.
**Resolution:** Pass `--no-cache` to `voce compile` in `build-fixtures.mjs`. Documented as a code comment so future maintainers don't strip it.
**Follow-up:** Cache key should include compiler version (or git hash). Out of scope for S61; ticket-able for compiler-cache hygiene.

### F-022 · Homebrew-installed `voce` shadowed local build
**Found:** Day 5 (continued), same investigation.
**Description:** `which voce` resolved to `/opt/homebrew/bin/voce → ../Cellar/voce-ir/1.1.0/bin/voce`, dated April 6 — a previously installed version. Even after `cargo build --release -p voce-validator` produced a fresh binary at `target/release/voce`, build-fixtures kept invoking the stale Homebrew one because it was earlier on PATH. Combined with F-021, this was the second source of stale-output divergence.
**Resolution:** `build-fixtures.mjs` now auto-detects `target/release/voce` and prefers it over PATH-installed `voce`. CI workflow already prepends `target/release` to PATH after building, so it's safe there too.
**Follow-up:** Document this in CONTRIBUTING.md when a maintainer next touches the file. Possibly add a top-level `bin/voce` shim in the repo root that always invokes the local build.

### F-023 · WASM rebuild requires rustup-managed `rustc`, not Homebrew Rust
**Found:** Day 5 (continued).
**Description:** `wasm-pack build --target web` failed with "wasm32-unknown-unknown target not found in sysroot" because `wasm-pack` invoked Homebrew's `rustc` (no wasm32 target) instead of the rustup-managed one (which has the target installed). User had rustup at `~/.cargo/bin/rustup` but homebrew Rust at `/opt/homebrew/bin/rustc` was earlier on PATH.
**Resolution:** Run `wasm-pack` with `PATH="$HOME/.cargo/bin:$PATH"` prepended. Documented in this journal; not yet codified into a script.
**Follow-up:** Add a `scripts/build-wasm.sh` that handles the PATH dance. Document in CONTRIBUTING.md.

### F-024 · Compiler now emits rich form defaults (the actual fix)
**Found:** Day 5 (continued), after addressing F-021 / F-022 / F-023 plumbing.
**Description:** Added 11 CSS rules to the DOM compiler's emitted `<style>` block covering form layout (flex column, gap, max-width 520px), label typography, input/textarea/select base styling (subtle background, border, 6px radius, 10/12 padding, 44px touch target), submit button (filled primary, hover/active/focus), error span color, and a `@media(max-width:520px)` rule that stretches the form full-width on mobile. All values use `var(--voce-{name}, fallback)` to respect IR-defined theme overrides.

**Compiled output growth:** every DOM compilation grows by ~1786 bytes (raw) — a fixed cost regardless of whether the IR contains a form. Acceptable: gzip compresses repeated CSS heavily, and the principle ("compiler emits rich defaults") is now load-bearing.

| Fixture | Before | After |
| --- | --- | --- |
| hero-section | 1,870 B | 3,656 B |
| contact-form | 3,272 B | 5,058 B |

**Resolution:** Shipped. 10 DOM snapshot tests in `packages/validator/tests/snapshots/` were re-diffed (purely additive) and accepted via `cargo insta accept`. WASM rebuilt (661 kB → 704 kB raw, 232 kB → 244 kB gzip). Both verifiers clean. Site-hero `dist-integrated/index.html` rebuilt at 14,198 B.
**Follow-up:** S64 sprint planned to extend the same principle to typography rhythm, lists, code blocks, blockquotes, tables, hr, theme fallbacks, and responsive container padding. Spec at `docs/plans/sprint-64-rich-defaults.md`.

### Day 5 final state

- ✓ Compiler emits rich form defaults; visible visual improvement on contact-form fixture
- ✓ Snapshot tests refreshed
- ✓ WASM rebuilt + distributed
- ✓ Fixture regeneration robust against cache poisoning + Homebrew shadowing
- ✓ Round-trip verifier passes (CLI ↔ WASM byte-for-byte)
- ✓ Pass-list drift verifier passes
- ✓ Integrated dist rebuilt
- → S64 scoped for the broader rich-defaults effort (typography, lists, code, theme fallbacks, responsive containers)

S61 ready to ship pending final visual review.



