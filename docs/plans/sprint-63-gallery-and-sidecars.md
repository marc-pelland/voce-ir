# Sprint 63 — Gallery & Per-Page Sidecars

**Phase:** 7 — Production Readiness
**Status:** Planned (depends on S61 + S62)
**Goal:** Make every page on `voce-ir.xyz` self-revealing — visible byte size, view-source-IR link, view-prompt link in the footer — and ship a curated `/examples` gallery with five new conversation-driven intent examples. The site moves from "trust us, the system works" to "click any page, see how it was built."

**Depends on:** S61 (homepage hero), S62 (multi-target shipped — gallery cards link there), S58 (current production deploy pipeline)

---

## Decision Required Before Sprint Start

The intent examples must be **genuinely conversation-driven**, not hand-authored IR retrofitted with a synthetic prompt. The "anti-vibe-coding" pillar (CLAUDE.md, `docs/research/CONVERSATIONAL_DESIGN.md`) is the project's differentiator — if the gallery is hand-authored, it doesn't demonstrate the differentiator and the gallery is honest only as "look at IR shapes," not "look at how the AI builds context and pushes back."

**Authoring rules for each new intent:**
- Real conversation log saved (`conversation.txt`), not reconstructed
- Prompt and AI follow-up questions captured
- AI pushback moments preserved (the points where the model challenged the request)
- Final IR validates cleanly through the standard 9 passes
- No manual editing of the IR after generation — if the IR has issues, fix the upstream conversation or the SDK, not the output

If this rule cannot be met, the sprint is rescoped to "polish existing intents and ship gallery without new entries."

---

## Deliverables

### 1. Five new intent examples in `examples/intents/`

Curated for variety across the IR surface area, not breadth for its own sake:

- `03-pricing-table` — exercises grid layout + theming
- `04-blog-post` — long-form RichTextNode + heading hierarchy + reading-time compute
- `05-saas-dashboard` — DataNode + sparkline + table + auth context
- `06-checkout-flow` — multi-step FormNode + validation + state machine
- `07-faq-accordion` — disclosure pattern + a11y semantics

Each entry contains:
- `intent.md` — the natural-language prompt
- `conversation.txt` — full conversation log (questions asked, pushback, decisions)
- `ir.voce.json` — the validated output
- `dist/index.html` — compiled DOM for the gallery thumbnail

Combined with the existing 2 (hero, contact form), the gallery has **7 entries**. Not 15.

### 2. `/examples` gallery page

- Grid view of all 7 intents
- Each card: thumbnail (sandboxed iframe with the compiled DOM), title, prompt excerpt, byte size
- Card click → full-page detail view: prompt + collapsible conversation log + collapsible IR + live render + multi-target tab strip (DOM/iOS/Android/Email — reuses S62 component)
- Generated through the IR pipeline like the rest of the site

### 3. Per-page sidecar metadata

Build-time post-processor in `packages/cli` (or a new `packages/site-meta` if it grows beyond a script) that:
- For each compiled page, emits `<page>.voce.meta.json` containing: byte size, source IR path, source prompt path, compile time, validation rule count
- Inlines a small (~1KB) script into every page's footer that reads its own meta and renders:
  - "This page is **X** bytes"
  - "View IR" → modal with source `.voce.json`, copy button
  - "View prompt" → modal with source prompt, copy button
- The sidecar feature applies to: landing, `/docs/*`, `/playground`, `/examples`, `/examples/*`

### 4. Build journal addendum

Append to `docs/site-v2-build-journal.md`:
- Conversation observations: where the AI asked good questions, where it failed to push back, where it generated invalid IR that required SDK fixes
- Intent generation friction points (these become AI bridge tickets)
- Sidecar pipeline edge cases (pages that don't have a clean IR source, e.g. mdBook-generated docs pages)
- At least 5 new findings with resolutions

---

## Acceptance Criteria

- [ ] 5 new intent directories exist in `examples/intents/`, each containing `intent.md`, `conversation.txt`, `ir.voce.json`, `dist/index.html`
- [ ] All 7 intents (2 existing + 5 new) validate cleanly with `voce validate` (zero errors, warnings documented)
- [ ] Each new intent's `conversation.txt` is a real log, not a reconstruction (verify by including timestamps and at least one pushback moment per conversation)
- [ ] `/examples` gallery page is live, lists all 7, all thumbnails render
- [ ] Detail page for each intent shows prompt + conversation + IR + live render + multi-target tabs
- [ ] Every page on `voce-ir.xyz` shows byte size + "View IR" + "View prompt" in the footer (or "Source not available" for docs pages that don't have an IR source)
- [ ] Sidecar `<page>.voce.meta.json` files are generated automatically by the build, not hand-written
- [ ] Sidecar inline script is under 1KB, runs without external dependencies, gracefully handles missing meta
- [ ] Build journal contains at least 5 new findings with resolutions
- [ ] Lighthouse Performance ≥ 90 on the gallery page (first paint, before any thumbnail iframe loads its iframe — defer iframe loads via `loading="lazy"`)

---

## Risks

1. **Conversation-driven authoring is slow.** Five real conversations (with question/pushback flow) may take a full day each if the AI bridge surfaces bugs. Acceptable — those bugs are the point. But if a single intent reveals a major SDK bug, pause the gallery work and fix the bug before continuing.
2. **Docs pages have no IR source.** mdBook generates `/docs/*` from Markdown. Either: (a) skip sidecars on docs pages and show "Source: mdBook" instead, or (b) build a parallel pipeline that generates `.voce.json` from the same Markdown source. Option (a) is the right call for this sprint; option (b) is its own future sprint if it ever matters.
3. **Modal UX for "View IR" / "View prompt".** A 500-line IR JSON dropped into a modal is unreadable. Use the same syntax-highlighted, collapsible tree component from the S61 hero. Reuse, do not rebuild.
4. **The "byte size" claim must be the actual rendered HTML, not the source IR.** Easy to measure wrong. Test on at least 3 pages that the displayed number matches `wc -c` on the deployed file.

---

## Out of Scope (Defer)

- Permalink to a generated IR (anyone-can-share URLs) — interesting follow-up sprint, not this one
- AI-bridge-driven gallery regeneration on every release — frozen with explicit `voce examples regenerate` opt-in
- 3D / WebGPU / interactive demo intents — surface area too large for this sprint, defer
- Search / filter on the gallery — 7 entries don't need it
