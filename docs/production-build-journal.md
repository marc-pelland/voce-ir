# Production Build Journal

**Date:** 2026-04-03
**Target:** voce-ir.xyz landing page
**Pipeline:** Hand-authored IR → validate → compile → deploy (dry-run)

---

## Pipeline Execution

| Stage | Time | Result |
|-------|------|--------|
| IR authoring | ~10 min | 380-line JSON IR, 30+ nodes |
| Validation | <0.1s | 1 warning (missing OG image), 0 errors |
| Compilation | 0.4s | 7,628 bytes HTML output |
| Deploy dry-run | <0.1s | Cloudflare Pages bundle ready |

## Findings

### 1. Heading Hierarchy Validation Works (Finding: Positive)

Initial IR used h1 (hero) and h3 (features) with no h2 in between. Validator caught this as A11Y005. Fixed by adding an h2 "Why Voce IR" section header before the feature cards.

**Resolution:** Added heading. This is exactly the kind of thing the validator should catch.

### 2. OG Image Warning (Finding: Expected)

SEO007 warned about missing og:image in OpenGraph data. For a text-only design this is intentional — no image to provide. The warning severity is correct (not an error).

**Resolution:** Acceptable warning. Future: could add a generated OG image from the page content.

### 3. Output Size Excellent (Finding: Positive)

7,628 bytes for a full landing page with:
- 7 sections (nav, hero, pipeline steps, features grid, install CTA, footer)
- 30+ nodes
- Complete CSS (theme vars, reset, flexbox/grid layouts, colors, typography)
- Security headers (CSP, X-Frame-Options, X-Content-Type-Options)
- SEO meta tags (title, description, canonical, OG)
- ARIA attributes on semantic sections

For comparison, a minimal Next.js page ships ~80KB of JS alone.

### 4. Compilation Speed (Finding: Positive)

0.4 seconds from IR to deployable HTML, including validation. Well under the 5-second target.

### 5. Theme System Works End-to-End (Finding: Positive)

Theme colors defined in the IR are compiled to CSS custom properties (`:root` vars) and applied throughout. Dark theme renders correctly with proper contrast.

### 6. Grid Layout Requires Explicit grid_columns (Finding: Friction)

The features grid needed `grid_columns` with explicit `Fr` unit objects rather than just `columns: 3`. The existing compile tests already covered this, but a new user might expect simpler syntax.

**Resolution:** Documented in schema reference. Future: could add a `columns` shorthand that expands to `grid_columns`.

### 7. Corner Radius Requires Length Objects (Finding: Friction)

Corner radius values need `{ "value": 8, "unit": "Px" }` objects, not plain numbers. Consistent with the rest of the schema but verbose.

**Resolution:** This is by design — Length is a typed struct in FlatBuffers. The AI generation layer produces these automatically.

## Quality Audit

| Metric | Target | Actual |
|--------|--------|--------|
| Page weight (excl. images) | < 100KB | 7.6KB |
| Compilation time | < 5s | 0.4s |
| Validation errors | 0 | 0 |
| HTML structure | Valid | DOCTYPE, lang, viewport, CSP |
| Accessibility | ARIA landmarks | nav, main, contentinfo roles |
| SEO | Meta complete | title, description, canonical, OG |
| Security headers | Present | CSP, X-Frame-Options, X-Content-Type-Options |

## Conclusion

The full pipeline works end-to-end for a production landing page. Output quality is excellent — tiny file size, complete security headers, proper accessibility, full SEO meta. The main friction points (grid_columns syntax, Length objects) are schema design decisions that the AI bridge handles automatically.
