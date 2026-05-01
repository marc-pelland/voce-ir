# Sprint 64 — Compiler-Emits-Rich-Defaults

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Make "the compiler emits rich, accessible, responsive output by default" a load-bearing assumption of the system. The IR specifies semantics and structure; the compiler fills the visual gap with intelligent defaults so a minimal IR produces a presentable result. The S61 contact-form regression (browser-default unstyled form) is the canonical failure mode this sprint prevents from recurring across the rest of the output surface.

**Depends on:** S61 (form defaults shipped — establishes the pattern)

---

## Motivation

S61 surfaced a real principle violation: `FormNode`/`FormField` carry no style fields, and the DOM compiler emits no defaults for `input`/`textarea`/`button`/`form`. The compiled contact-form looked like a plain 1995 web form. We patched the form gap in S61 (F-022). Several other surfaces have the same shape: tables, lists, code blocks, blockquotes, headings without explicit styles, and IR-level Containers that don't carry responsive padding.

The user-stated principle: **"creating and outputting rich and visual experiences rather than plain with no visual styles. Aim to not overdo the visuals, but at least intelligently laying things out, adding some styling, and ensuring that it is responsive and accessible should all be core to our functionality."**

This sprint extends that principle from forms to the rest of the output.

---

## Deliverables

### 1. Default base stylesheet expansion (`packages/compiler-dom/src/emit/html.rs`)

Add baseline CSS rules to the compiler's emitted `<style>` block, in the same shape as the form defaults from S61. Each addition should use `var(--voce-{name}, fallback)` so theme-defined IRs override gracefully.

- **Typography rhythm** — `h1`/`h2`/`h3`/`h4` margin-bottom values, `p` line-height, paragraph spacing. (Inline `font-size`/`font-weight` already come from IR but spacing is missing.)
- **Lists** — `ul`/`ol` indentation and bullet style; `li` line-height.
- **Code** — `code` inline (mono font, subtle background, 2px padding); `pre code` block (mono, padded, scrollable, theme-aware bg).
- **Blockquote** — left border in accent color, italic, padded.
- **Tables** — minimal padding, alternating row tint, header row weight.
- **Images** — already `max-width:100%; display:block`; add `border-radius` from theme if defined.
- **Hr** — subtle horizontal rule with theme-aware color.

### 2. Responsive defaults

- Container nodes already accept fixed pixel padding/gap. Add a `@media(max-width:640px)` block that scales padding values down by ~50% on small screens. Achievable by emitting `--voce-pad-mobile` vars and using `clamp()` in container inline styles, OR by emitting per-container `@media` rules at compile time. Pick whichever is cheaper.
- Image `srcset`/`sizes` for `MediaNode` if the IR specifies multiple sources (not in scope of S64; flag for a future sprint).

### 3. Theme-aware semantic colors

Currently the IR's `theme.colors` map produces `--voce-{slot}` CSS vars only when the IR defines them. Add **fallback theme defaults** to the compiler's stylesheet so an IR with no theme still gets reasonable colors:

```css
:root{
  --voce-fg:#111;
  --voce-bg:#fff;
  --voce-muted-fg:#666;
  --voce-border:rgba(127,127,127,.25);
  --voce-surface:rgba(127,127,127,.04);
  --voce-primary:#6366f1;
  --voce-error:#ef4444;
  --voce-warning:#f59e0b;
  --voce-success:#10b981;
}
@media (prefers-color-scheme: dark){
  :root{
    --voce-fg:#e8e8ec;
    --voce-bg:#0a0a0c;
    --voce-muted-fg:#8b8b94;
    --voce-border:rgba(255,255,255,.1);
    --voce-surface:rgba(255,255,255,.04);
  }
}
```

IR-defined theme vars override these (current order is correct).

### 4. Snapshot test refresh

All ~10 DOM snapshot tests in `packages/validator/tests/snapshots/` will re-diff. Each diff should be additive (new CSS rules) — review and accept.

### 5. Compiler change ripple verification

After changes:
- `cargo test --workspace` clean
- `wasm-pack build --target web` for `playground-wasm`
- WASM artifacts copied to `packages/site-hero/wasm/` and `packages/playground/wasm/`
- `node packages/site-hero/build-fixtures.mjs` — fixtures regenerate, pass round-trip
- CI `frontend-verify` job clean

### 6. Visual regression sweep

The 2 site-hero fixtures (hero-section, contact-form) get visually re-reviewed in the browser. Any other compiled examples in the repo (`examples/intents/*`, `examples/demos/*` if applicable) get a quick visual check too.

---

## Acceptance Criteria

- [ ] Compiler emits typography spacing for headings, paragraphs, and lists
- [ ] Compiler emits inline-`<code>` and block-`<pre><code>` defaults
- [ ] Compiler emits blockquote, hr, table baseline styles
- [ ] Compiler emits a complete `:root` fallback palette for light + dark schemes
- [ ] Container padding scales down on small screens (mechanism to be decided in spike)
- [ ] All snapshot tests refreshed and accepted
- [ ] WASM rebuilt and distributed; round-trip verifier clean
- [ ] Site-hero fixtures regenerated and visually reviewed
- [ ] CI `frontend-verify` job passes on a fresh PR
- [ ] No new schema fields added (everything is compiler-side)
- [ ] Total stylesheet addition ≤ 2 KB raw (≤ 600 B gzipped) — keep the budget tight

---

## Risks

1. **Hidden snapshot consumers.** Some downstream test or doc may compare exact compiled output. Audit before changing.
2. **Theme conflict.** IRs that define partial theme vars may rely on browser defaults for the missing slots. The new `:root` fallback fills those, which could change appearance. Document and ship; offer a `--minimal-defaults` compile flag if regression complaints arrive.
3. **WASM size growth.** Currently 661 kB → 704 kB after S61's form CSS. Each addition grows it. Stay under 800 kB or call out the regression.
4. **Touch targets and `prefers-reduced-motion`** — already partially honored. Audit during this sprint.

---

## Out of Scope (Defer)

- New IR node types (e.g., `ExternalIsland`, `RawHtml`)
- Schema changes (no new style fields on FormField, etc.)
- Per-target custom defaults (this sprint is DOM only; iOS/Android compilers have their own rich-defaults effort if needed)
- A dark-mode toggle in compiled output (relies on `prefers-color-scheme` only)
- Image `srcset` / responsive image sourcing
