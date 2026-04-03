# Sprint 18 — Security, A11y & Output Quality

**Status:** Planned
**Goal:** Emit security headers, ARIA attributes, focus management, and keyboard navigation. Measure output quality against Lighthouse and axe-core targets. Tune output size and TTI to meet the <10KB / <50ms targets. After this sprint, compiled output is production-quality: secure, accessible, performant.
**Depends on:** Sprint 17 (asset pipeline — image sizes affect total output size and TTI)

---

## Deliverables

1. Security headers: CSP meta tag, X-Frame-Options, X-Content-Type-Options
2. Output safety: no eval(), innerHTML, or document.write() in emitted code
3. SemanticNode → ARIA attributes on corresponding HTML elements
4. FocusTrap → focus management JS
5. LiveRegion → aria-live attributes
6. Keyboard navigation: tabindex, keydown handlers
7. Performance tuning: <10KB landing page, <50ms TTI
8. Audit integration: Lighthouse >95, axe-core 0 violations

---

## Tasks

### 1. Security Header Emission (`security/headers.rs`)

Emit Content Security Policy as a `<meta>` tag:
