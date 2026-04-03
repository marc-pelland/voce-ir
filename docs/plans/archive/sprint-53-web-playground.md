# Sprint 53 — Web Playground

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Build a browser-based "try Voce IR" playground where users can paste a prompt or IR JSON, see compiled output live, and explore validation results. No local install required. Deploy to Cloudflare Pages.

**Depends on:** WASM compiler target (Phase 4), SDK package, Inspector package

---

## Deliverables

- New package: `packages/playground/` — Vite + vanilla TypeScript (no framework dependency)
- WASM build of validator and compiler-dom: compile core Rust crates to `wasm32-unknown-unknown` via `wasm-pack`, expose `validate(ir_bytes) -> ValidationResult` and `compile_dom(ir_bytes) -> string`
- Three-panel layout:
  - **Input panel** — tabbed: "Prompt" (natural language textarea) and "IR JSON" (code editor with syntax highlighting via CodeMirror or Monaco)
  - **Preview panel** — live-rendered iframe showing compiled DOM output
  - **Details panel** — tabbed: "Validation" (pass/fail per rule with messages), "Compiled Output" (syntax-highlighted HTML), "IR Inspector" (tree view of nodes)
- Compile target selector dropdown: DOM, Email (others show "coming to playground soon")
- Shareable URLs: encode IR as base64 in URL hash for link sharing
- Example gallery: 5 pre-built IR examples (landing page, card, form, nav bar, dashboard widget) loadable from buttons
- Prompt mode: calls MCP server or shows placeholder explaining AI generation requires API key
- `voce-playground` Cloudflare Pages deployment config (`wrangler.toml` for Pages)
- Responsive design — usable on tablet, gracefully degrades on mobile (stacked panels)

## Acceptance Criteria

- [ ] WASM validator and compiler-dom build successfully with `wasm-pack build --target web`
- [ ] Pasting valid IR JSON into the editor shows compiled HTML in the preview iframe within 500ms
- [ ] Pasting invalid IR shows validation errors in the details panel with specific node paths and messages
- [ ] Compile target dropdown switches between DOM and Email output
- [ ] Each of the 5 example gallery items loads and renders correctly
- [ ] Shareable URL round-trips: copy URL, open in new tab, same IR and output appear
- [ ] Page loads in under 2 seconds on 3G throttle (WASM loaded lazily)
- [ ] Total bundle size under 500KB gzipped (excluding WASM)
- [ ] WASM module size under 2MB
- [ ] Deploys to Cloudflare Pages via `wrangler pages deploy`
- [ ] No runtime errors in Chrome, Firefox, Safari latest
- [ ] Lighthouse accessibility score >= 90
