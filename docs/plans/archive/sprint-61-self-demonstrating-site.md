# Sprint 61 — Live Pipeline Hero (voce-ir.xyz)

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Replace the static hero on `voce-ir.xyz` with a live, in-browser demonstration of the Voce IR pipeline. Visitors pick a starter prompt and watch IR generate, validate, and compile into a rendered DOM result — entirely client-side via `playground-wasm`. Cached responses only; no API key required, no proxy infrastructure. This is the smallest possible "the site is the demo" payload.

**Depends on:** S53 (playground-wasm), S58 (current production site), S59 (compilation cache)

---

## Motivation

The current `voce-ir.xyz` is a static landing page that *describes* the system. Every claim ("AI generates IR from conversation," "production landing page: 7.6KB") is asserted but not shown. This sprint replaces the hero with a single interactive demo where the pipeline runs on-screen, on user input, in under a few seconds. Smaller surface area than a full multi-target showcase or gallery overhaul — those are S62/S63. Get one piece of indisputable proof on the homepage first.

---

## Deliverables

### 1. `packages/site-hero/` — TypeScript package

A new mountable hero component, embedded as an island into the existing static landing page. Wraps `playground-wasm` and animates pipeline progress.

- **6 starter prompts** with pre-cached IR responses bundled as JSON next to the component (no API call at runtime):
  - "A pricing page for a developer tool"
  - "A contact form with autosave"
  - "A blog index with a featured post"
  - "A SaaS dashboard with a sparkline"
  - "A signup form with password rules"
  - "An FAQ accordion"
- **Three-column layout that animates as the pipeline runs:**
  1. IR JSON tree (collapsible, syntax-highlighted, byte counter)
  2. Validation pass list — 9 passes light up green in sequence with rule counts
  3. Compiled DOM rendered in a sandboxed `<iframe srcdoc>` with byte size + compile time
- **Lazy WASM load:** the 646KB `playground-wasm` blob does not download on page load. It loads on first user interaction (prompt click). A skeleton placeholder shows in the meantime so layout doesn't shift.
- **Cached starter data path** does not invoke the SDK or any external API. The "AI generation" stage is a deterministic playback of the cached IR with a short artificial delay so the visualization feels truthful (200ms streaming feel).

### 2. Wired into existing landing page

- The current `examples/production/landing.voce.json` is regenerated through the pipeline so the hero section uses a new `Container` with an `external_island` slot (or the simplest equivalent — see Risk 3 below).
- The rest of the page (pipeline section, features, CTA, footer) stays IR-compiled.
- A `<script type="module" src="/site-hero.js">` tag is the only added markup; CSS and HTML for the static parts remain compiler-emitted.

### 3. Build journal

`docs/site-v2-build-journal.md` (new file) documenting:
- WASM bundle size at first paint vs. post-interaction
- Bugs found in `playground-wasm`, the validator, or the DOM compiler while building this
- Anything the pipeline could not produce that required a workaround (becomes follow-up tickets)
- At least 5 findings with resolutions

---

## Acceptance Criteria

- [ ] `voce-ir.xyz` homepage shows a live prompt → IR → validate → compile → render demo above the fold
- [ ] All 6 starter prompts complete the full pipeline animation in under 3 seconds on a Macbook M1 (Chrome stable)
- [ ] No API key is required for any starter prompt
- [ ] No outbound network requests fire on page load (verified via DevTools Network panel — only the static page assets)
- [ ] The WASM blob lazy-loads only after the first prompt click
- [ ] Lighthouse Performance ≥ 90 on **first paint** (before any user interaction)
- [ ] Lighthouse Accessibility ≥ 95
- [ ] Lighthouse Performance budget **post-interaction** is documented in the build journal (no hard pass/fail — the WASM cost is acknowledged)
- [ ] Initial HTML payload remains under 30KB
- [ ] The static parts of the page are still IR-compiled (no hand-edited HTML except the single `<script>` tag for the island)
- [ ] Hero is keyboard-navigable (tab through prompts, Enter to run, Escape closes the result)
- [ ] Sandboxed iframe blocks all script execution from compiled output (CSP enforced)
- [ ] Build journal contains at least 5 findings with resolutions
- [ ] All blocking bugs found in playground-wasm or the DOM compiler are fixed in-sprint

---

## Risks

1. **WASM bundle size (646KB).** Loading it tanks Lighthouse Performance even when lazy-loaded. Mitigation: only count Performance pre-interaction; document post-interaction cost in the journal. Future sprint may pursue WASM size reduction (likely tree-shaking unused validator passes when only the demo subset runs).
2. **iOS/Android compile targets are not part of this sprint.** Visitors will not see multi-target output until S62. Set expectations clearly in copy ("Watch the DOM target compile live — see iOS, Android, and Email outputs in the gallery below").
3. **The hero is an interactive island, not pure IR-compiled output.** The IR has no "live pipeline visualization" node type, and adding one to the schema for a single demo would be over-engineering. Honest framing in the build journal and any blog post: "the static parts of voce-ir.xyz are IR-compiled; the hero is a hand-built island that wraps `playground-wasm`." This is acceptable — the demo proves the pipeline works without claiming the visualization itself was generated.
4. **Cached IR drift.** If the schema or compiler changes, the bundled cached IRs may no longer validate or compile. Add a CI check that re-runs each cached IR through `voce validate` and `voce compile dom` on every PR. Failing the check forces regeneration before merge.
5. **Visual polish risk.** A clunky-looking hero is worse than no hero. Reserve the last day of the sprint for visual review and copy revision; do not declare done until the demo feels good to use.

---

## Out of Scope (Defer)

- "Try your own prompt" with user-supplied API key → defer (probably never; revisit only if conversion data demands it)
- Multi-target split-screen (DOM/iOS/Android/Email side-by-side) → **S62**
- Per-page "this is X bytes / view IR / view prompt" footer → **S63**
- Expanded intent gallery (`examples/intents/` 2 → ~7) → **S63**
- `/examples` gallery page → **S63**
- WebGPU pane → dropped entirely (no marketing value, adds bundle weight)
- Voice input on the hero
- Permalinks to generated IRs / public sharing

---

## Sequencing Inside the Sprint

1. **Day 1 — Cache the 6 starter prompts.** Run each through the existing SDK once, save IR + validation report + compiled HTML as JSON fixtures. No UI work yet.
2. **Day 2 — Wire `playground-wasm` into a new `site-hero` package.** Plain HTML scaffold, no styling. Verify each cached prompt round-trips through `validate` and `compile_dom` in the browser.
3. **Day 3 — Three-column layout, animation timing, sandboxed iframe.** Get the pipeline-runs-on-screen feel right.
4. **Day 4 — Embed into landing page IR, deploy to staging, fix bugs, write journal.**
5. **Day 5 — Visual polish, copy review, accessibility audit, ship.**
