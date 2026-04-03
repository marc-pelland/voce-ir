# Sprint 12 — Layout Compilation

**Status:** Planned
**Goal:** Compile all layout IR nodes to HTML elements with pre-computed inline styles. Container → flexbox/grid, Surface → styled div, TextNode → semantic heading/paragraph/span, MediaNode → responsive picture element. All static dimensions are pre-computed at compile time using Taffy. After this sprint, a layout-only page compiles to pixel-accurate HTML with zero JS.
**Depends on:** Sprint 11 (compiler pipeline, ingestion, emission)

---

## Deliverables

1. Container lowering → `<div>` with flexbox or grid inline styles
2. Surface lowering → `<div>` with background, border-radius, shadow, border
3. TextNode lowering → `<span>`, `<p>`, or `<h1>`-`<h6>` with typography styles
4. MediaNode lowering → `<picture>` with `<source>` + `<img>` fallback
5. ResponsiveRule lowering → `@media` query blocks in `<style>` tag
6. Taffy integration for compile-time layout pre-computation
7. Integration test: landing page layout section → valid styled HTML

---

## Tasks

### 1. Container Lowering (`lower/container.rs`)

Map Container layout modes to CSS:

| IR Layout | CSS Output |
|-----------|------------|
| `Stack` | `display: flex; flex-direction: column` |
| `Flex` | `display: flex` + direction, wrap, justify, align from IR fields |
| `Grid` | `display: grid` + template-columns, template-rows, gap |
| `Absolute` | `position: relative` on container, `position: absolute` on children |

Emit all layout properties as inline `style` attributes. Map IR spacing units to CSS:
- `Px(n)` → `{n}px`
- `Rem(n)` → `{n}rem`
- `Percent(n)` → `{n}%`
- `Vw(n)` / `Vh(n)` → `{n}vw` / `{n}vh`

Gap, padding, margin from `EdgeInsets` → `padding: {top}px {right}px {bottom}px {left}px`.

### 2. Surface Lowering (`lower/surface.rs`)

Surface → `<div>` with decorative styles:

- `background_color` → `background-color: {color}`
- `corner_radii` → `border-radius: {tl}px {tr}px {br}px {bl}px`
- `shadow` → `box-shadow: {x}px {y}px {blur}px {spread}px {color}`
- `border` → `border: {width}px solid {color}` (per-side if needed)
- `overflow` → `overflow: hidden|scroll|auto`

Color values from IR `Color` struct (r, g, b, a) → `rgba({r},{g},{b},{a})`.

### 3. TextNode Lowering (`lower/text.rs`)

Choose HTML tag based on semantic meaning:

| IR Config | HTML Tag |
|-----------|----------|
| `heading_level: 1-6` | `<h1>` - `<h6>` |
| No heading, block context | `<p>` |
| No heading, inline context | `<span>` |

Typography styles as inline CSS:
- `font_family` → `font-family`
- `font_size` → `font-size`
- `font_weight` → `font-weight`
- `line_height` → `line-height`
- `letter_spacing` → `letter-spacing`
- `text_align` → `text-align`
- `text_decoration` → `text-decoration`
- `color` → `color`
- `text_transform` → `text-transform`

Content: use `content` field directly, or `localized_content.default_value` for the default locale build.

### 4. MediaNode Lowering (`lower/media.rs`)

Emit responsive image markup:
