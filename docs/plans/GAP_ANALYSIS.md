# Voce IR — Gap Analysis & Remediation Plan

**Date:** 2026-04-06
**Status:** The schema is ambitious (27 node types, 12 .fbs files) but the DOM compiler only emits a fraction of what's defined. Critical web primitives are missing entirely.

---

## Production Readiness: 25%

Can handle: simple landing pages with flexbox layouts, text, images, basic forms.
Cannot handle: content sites, navigation, rich text, tables, video, interactive buttons, responsive design.

---

## Critical Gaps (Blocks any real website)

### Gap 1: No Links
- **Impact:** Cannot build any navigable website
- **What's needed:** `<a href>` support — either a new LinkNode or `href` field on TextNode/Surface
- **Effort:** Small — schema change + compiler emit

### Gap 2: No Semantic HTML Elements
- **Impact:** Only emits `<div>`, losing SEO and a11y value
- **What's needed:** Map SemanticNode roles to actual HTML elements: `role="navigation"` → `<nav>`, `role="main"` → `<main>`, `role="contentinfo"` → `<footer>`, etc.
- **Effort:** Small — emit logic change only, no schema change

### Gap 3: No Interactive States (hover, focus, active)
- **Impact:** Links/buttons have no visual feedback, looks broken
- **What's needed:** Emit `:hover`, `:focus`, `:active` styles. GestureHandler already has keyboard_equivalent — extend with visual states.
- **Effort:** Medium — need to generate CSS classes instead of pure inline styles

### Gap 4: No Video/Audio
- **Impact:** MediaNode claims to support video/audio but only emits `<img>`
- **What's needed:** Check `media_type` field, emit `<video>` or `<audio>` with controls
- **Effort:** Small — conditional in emit

### Gap 5: RichTextNode is Dead Code
- **Impact:** Cannot render CMS content, markdown, lists, tables, inline links
- **What's needed:** Implement RichTextNode → HTML: paragraphs, headings, lists (ul/ol/li), tables, links, bold/italic/code spans
- **Effort:** Medium — new emit function, recursive block/span rendering

### Gap 6: Missing Form Inputs
- **Impact:** Cannot build real forms (select, checkbox, radio, date, file)
- **What's needed:** Extend the form emitter to handle all FormFieldType variants
- **Effort:** Small — add cases to existing match

### Gap 7: No Responsive Design
- **Impact:** Sites look wrong on mobile
- **What's needed:** ResponsiveRule is in the schema but not compiled. Emit `@media` queries from breakpoint definitions.
- **Effort:** Medium — collect rules, generate media query CSS blocks

### Gap 8: Schema Fields Not Compiled
- **Impact:** IR authors set properties that silently do nothing
- **What's needed:** Compile text-decoration, border styling, text-overflow, max-lines, font-family, grid-rows, margins, viewport units
- **Effort:** Small per field — mostly adding lines to ingest.rs

---

## Important Gaps (Needed for real use)

### Gap 9: No Buttons
- Surfaces with tap gestures should render as `<button>` or `<a>` depending on context
- Need a way to distinguish decorative surfaces from interactive ones

### Gap 10: No CSS Classes (Everything Inline)
- Duplicate styles across identical nodes
- Cannot do pseudo-selectors (:hover, :focus) with inline styles
- Need to extract shared styles into `<style>` block classes

### Gap 11: Select/Checkbox/Radio Inputs
- FormFieldType enum has them but emitter only handles text-like inputs

### Gap 12: Margin Support
- Only padding exists — no way to add spacing between sibling elements except gap

---

## Remediation Sprints

### Sprint R1: Links, Semantic HTML, Buttons (Priority: Critical)

**Schema changes:**
- Add `href: string` field to TextNode (optional — when present, emit `<a>` instead of `<p>`)
- Add `href: string` field to Surface (when present, emit `<a>` wrapper)
- Add `target: string` to control `_blank` etc.

**Compiler changes:**
- TextNode with `href` → emit `<a href="...">` instead of `<p>`/`<h*>`
- Surface with `href` → wrap in `<a href="...">`
- SemanticNode role → map to HTML elements:
  - `navigation` → `<nav>`
  - `main` → `<main>`
  - `contentinfo` → `<footer>`
  - `banner` → `<header>`
  - `complementary` → `<aside>`
  - `region` → `<section>`
  - `article` → `<article>`
- GestureHandler target with tap → emit `<button>` wrapper (or `role="button"` + tabindex)

**Validation:**
- Add rule: links must have accessible text (either content or aria-label)

**Tests:** Fixtures for links, semantic elements, buttons. Snapshot tests.

### Sprint R2: Interactive States & CSS Classes (Priority: Critical)

**Compiler changes:**
- Move from pure inline styles to a hybrid approach:
  - Generate unique CSS class names per style combination
  - Emit a `<style>` block with classes
  - Nodes reference classes via `class="..."` 
- Add `:hover`, `:focus`, `:active` state support:
  - TextNode with `href` gets hover color change
  - Surface with gesture handler gets cursor:pointer + hover effect
  - Form inputs get focus ring
- Add `:focus-visible` for keyboard focus indicators

**Tests:** Verify CSS classes generated, hover states present.

### Sprint R3: Video/Audio, Missing CSS Properties (Priority: High)

**Compiler changes:**
- MediaNode: check `media_type`, emit `<video controls>` or `<audio controls>`
- Add to ingest.rs:
  - `text-decoration` from TextNode.text_decoration
  - `border` from Surface.border (width/style/color)
  - `text-overflow` + `-webkit-line-clamp` from max_lines
  - `font-family` from TextNode.font_family
  - `grid-template-rows` from Container.grid_rows
  - `margin` support (add to schema if needed)
  - Viewport units: map Vh→vh, Vw→vw, Dvh→dvh, Svh→svh

**Tests:** Video/audio emission, border styling, text truncation.

### Sprint R4: RichTextNode Implementation (Priority: High)

**Compiler changes:**
- New `emit_rich_text()` function in html.rs
- Block types → HTML:
  - Paragraph → `<p>`
  - Heading → `<h1>`-`<h6>`
  - UnorderedList → `<ul><li>...</li></ul>`
  - OrderedList → `<ol><li>...</li></ol>`
  - Table → `<table><tr><td>...</td></tr></table>`
  - CodeBlock → `<pre><code>...</code></pre>`
  - Blockquote → `<blockquote>`
  - HorizontalRule → `<hr>`
- Span marks → HTML:
  - Bold → `<strong>`
  - Italic → `<em>`
  - Code → `<code>`
  - Link → `<a href="...">`
  - Strikethrough → `<s>`

**Tests:** Rich text with all block types, nested lists, tables.

### Sprint R5: Responsive Design (Priority: High)

**Compiler changes:**
- Collect ResponsiveRule nodes from IR
- Generate `@media` queries:
  - Breakpoint widths → `@media (max-width: ...)`
  - Property overrides → CSS rules targeting node IDs/classes
- Common patterns:
  - Stack columns on mobile (flex-direction: row → column)
  - Reduce font sizes
  - Adjust padding/margins
  - Hide/show elements

**Tests:** Responsive rules generate correct media queries.

### Sprint R6: Complete Form Inputs & Polish (Priority: Medium)

**Compiler changes:**
- `<select>` with `<option>` elements
- `<input type="checkbox">` and `<input type="radio">`
- `<input type="date">`, `type="time"`, `type="color"`, `type="range"`, `type="file">`
- Form reset button
- Conditional field visibility

**Tests:** All input types render correctly.

---

## Sprint Order

```
R1: Links, Semantic HTML, Buttons     <- unblocks real websites
R2: Interactive States, CSS Classes   <- makes them usable
R3: Video/Audio, Missing Properties   <- completeness
R4: RichTextNode                      <- content sites
R5: Responsive Design                 <- mobile
R6: Complete Forms                    <- full form support
```

**Estimated: 6 sprints to reach genuine production readiness for web.**

After these, the compiler can produce a real website that you wouldn't be embarrassed to ship.
