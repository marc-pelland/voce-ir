# Sprint 15 — Theming, SEO & i18n Emission

**Status:** Planned
**Goal:** Compile theming, SEO metadata, and internationalization to HTML output. ThemeNode → CSS custom properties, PageMetadata → complete `<head>`, static i18n → per-locale HTML files. After this sprint, compiled pages have correct meta tags, structured data, theme support, and can be built for multiple locales.
**Depends on:** Sprint 14 (animation compilation — animations may reference theme values)

---

## Deliverables

1. ThemeNode → CSS custom properties on `:root`
2. Dark/light mode → class toggle + `prefers-color-scheme` media query
3. PageMetadata → complete `<head>` with title, meta, OG, Twitter, canonical, robots
4. StructuredData → `<script type="application/ld+json">`
5. Static i18n → compile separate HTML per locale with resolved strings
6. Sitemap.xml and robots.txt generation from RouteMap
7. `<html lang="xx" dir="ltr|rtl">` from ViewRoot

---

## Tasks

### 1. Theme Compilation (`lower/theme.rs`)

ThemeNode compiles to CSS custom properties scoped to `:root`:
