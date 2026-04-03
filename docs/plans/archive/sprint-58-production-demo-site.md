# Sprint 58 — End-to-End Demo: Real Production Site

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Build a real production website using Voce IR end-to-end. Use the full pipeline (conversation to generation to compilation to deployment) to ship an actual site — either the Voce IR project homepage or a real production landing page. Document every friction point and bug found.

**Depends on:** Sprint 51 (images), Sprint 52 (deployment), Sprint 56 (fonts), Sprint 57 (error handling)

---

## Deliverables

- **Target site:** voce-ir.xyz landing page — hero section, feature grid, code example, getting started CTA, footer
- **Full pipeline execution documented step-by-step:**
  1. Natural language conversation describing the page (saved as `examples/production/prompt.md`)
  2. AI generation via SDK producing IR (saved as `examples/production/landing.voce`)
  3. Validation pass (saved as `examples/production/validation-report.json`)
  4. Compilation to DOM target with real images, subsetted fonts, optimized output (saved as `examples/production/dist/`)
  5. Deployment via adapter-cloudflare to voce-ir.xyz (or adapter-static as fallback)
- **Bug journal:** `docs/production-build-journal.md` documenting:
  - Every error encountered and how it was resolved
  - Every manual intervention required (things the pipeline should have handled)
  - Missing features discovered during real usage
  - Performance measurements at each pipeline stage
- **Quality audit of output:**
  - Lighthouse scores (Performance, Accessibility, Best Practices, SEO — all targets >= 90)
  - Core Web Vitals: LCP < 2.5s, FID < 100ms, CLS < 0.1
  - HTML validation (W3C validator, zero errors)
  - Cross-browser testing: Chrome, Firefox, Safari, Edge
  - Mobile responsiveness: verified at 375px, 768px, 1024px, 1440px
- **Comparison artifacts:** same page built with hand-coded HTML for size/performance comparison
- **Fixes:** all blocking bugs found during this sprint are fixed in-sprint (not deferred)

## Acceptance Criteria

- [ ] voce-ir.xyz (or equivalent) is live and publicly accessible
- [ ] Site was generated entirely through the Voce IR pipeline (no hand-edited HTML in final output)
- [ ] Lighthouse Performance score >= 90
- [ ] Lighthouse Accessibility score >= 95
- [ ] All Core Web Vitals in "Good" range
- [ ] W3C HTML validator reports zero errors
- [ ] Site renders correctly on Chrome, Firefox, Safari, Edge (latest versions)
- [ ] Site is responsive at 375px, 768px, 1024px, 1440px breakpoints
- [ ] Production build journal documents at least 5 findings with resolutions
- [ ] All blocking bugs found are fixed (no known broken functionality in shipped site)
- [ ] Total page weight under 100KB (excluding images), under 500KB including images
- [ ] Time from `voce compile` to deployable output under 5 seconds
