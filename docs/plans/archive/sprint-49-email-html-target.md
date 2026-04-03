# Sprint 49 — Email HTML Compile Target

**Status:** Planned
**Phase:** 6 (Ecosystem & Community)
**Depends on:** S20 (compiler-dom as reference)

---

## Goal

Build an email-specific compile target that produces HTML emails compatible with major email clients (Outlook, Gmail, Apple Mail, Yahoo).

---

## Deliverables

- `packages/compiler-email/` Rust crate
- Table-based layout: Container → nested table/tr/td (email clients ignore flexbox/grid)
- Inline CSS: all styles inlined on elements (no style tags for Gmail)
- Outlook conditional comments: VML fallbacks for rounded corners, backgrounds
- Gmail-specific hacks: class-based dark mode, image blocking fallbacks
- Apple Mail: webkit media queries for enhanced rendering
- Responsive email: fluid-hybrid approach (max-width + width:100%)
- Image handling: hosted URLs with alt text, retina srcset where supported
- Dark mode support: meta color-scheme + inverted color fallbacks
- Client preview matrix: render preview for top 10 email clients
- `voce compile --target email` CLI flag
- Email-specific validation pass: no unsupported CSS, image sizes, text-to-image ratio

---

## Acceptance Criteria

- [ ] Reference landing page IR compiles to working email HTML
- [ ] Email renders correctly in Outlook 2019+ (table layout, VML)
- [ ] Email renders correctly in Gmail (inline CSS, no style tag)
- [ ] Email renders correctly in Apple Mail (webkit enhancements)
- [ ] Dark mode adaptation works in supporting clients
- [ ] All images have alt text and hosted URL references
- [ ] Email-specific validation catches unsupported CSS properties
- [ ] Output passes Litmus/Email on Acid basic checks
