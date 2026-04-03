# Sprint 56 — Font Subsetting and Optimization

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Implement real font subsetting using Rust font tooling. Analyze IR text content for used glyphs, subset fonts to minimum required characters, convert to WOFF2, and emit optimized `@font-face` declarations with `font-display:swap` and preload hints.

**Depends on:** Sprint 51 (asset pipeline infrastructure), TextNode compilation in compiler-dom

---

## Deliverables

- Add `allsorts` (font parsing/subsetting) and `brotli` (WOFF2 compression) crates to compiler-dom
- `FontPipeline` struct that:
  - Collects all text content from TextNode instances in the IR tree
  - Determines unique Unicode codepoints used per font family
  - Subsets source font files (TTF/OTF) to only required glyphs plus a safety set (punctuation, digits, common symbols)
  - Converts subsetted fonts to WOFF2 format
  - Generates content-hashed filenames (e.g., `inter-subset-a1b2c3.woff2`)
- `@font-face` emission with:
  - `font-display: swap` (prevents invisible text during load)
  - `unicode-range` descriptor for multi-file strategies
  - Local font fallback: `local('Inter'), url(...)` pattern
- `<link rel="preload" as="font">` for critical above-the-fold fonts (fonts used in first ViewRoot)
- Font fallback stack generation: for each specified font, emit a system font fallback stack with matching metrics using `size-adjust`, `ascent-override`, `descent-override`
- Style pack integration: style packs can declare font sources (Google Fonts URL, local path, or bundled)
- `--skip-fonts` flag to bypass font processing during development
- Support for variable fonts: detect axes, subset without losing variable capabilities
- Unit tests for subsetting (verify output contains only requested glyphs), WOFF2 conversion, and `@font-face` generation

## Acceptance Criteria

- [ ] Given an IR using Inter font with 50 unique characters, subsetted WOFF2 is under 15KB (vs ~300KB full)
- [ ] Generated `@font-face` includes `font-display: swap` on every declaration
- [ ] Preload `<link>` emitted for fonts used in the first screen of content
- [ ] Fallback font stack includes metric-adjusted system fonts (`size-adjust`, `ascent-override`)
- [ ] Variable font subsetting preserves weight axis (tested with Inter Variable)
- [ ] `unicode-range` in `@font-face` matches actual glyphs in subset
- [ ] Font files use content-hash filenames for cache busting
- [ ] `--skip-fonts` flag produces output with standard `font-family` CSS only (no files, no preloads)
- [ ] End-to-end: reference landing page compiled with font pipeline produces zero layout shift on load (CLS = 0)
- [ ] All existing tests continue to pass
- [ ] `cargo clippy --workspace -- -D warnings` passes clean
