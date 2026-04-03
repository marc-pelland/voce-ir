# Sprint 44 — Inspector Polish & v0.5.0

**Status:** Planned
**Phase:** 5 (Visual Inspector & Tooling)
**Depends on:** S43 (conversational debugging)

---

## Goal

Polish the inspector for public release: extension API for community plugins, comprehensive documentation, keyboard-driven workflow, and v0.5.0 release.

---

## Deliverables

- Extension API: register custom inspector panels, custom node renderers, custom profilers
- Extension documentation: how to build and publish inspector plugins
- Keyboard shortcuts for all inspector actions (documented in help panel)
- Inspector settings: panel positions, theme (light/dark), default panels
- Performance optimization: inspector overhead < 5% of page frame budget
- Comprehensive documentation: user guide, API reference, tutorial videos
- CHANGELOG.md for v0.5.0
- Updated ROADMAP.md with Phase 5 completion status
- npm publish: @voce-ir/inspector package
- Tag v0.5.0 release

---

## Acceptance Criteria

- [ ] Extension API allows third-party panels to register and render
- [ ] Sample extension plugin documented and working
- [ ] All inspector actions have keyboard shortcuts
- [ ] Inspector overhead < 5% frame time (measured on reference landing page)
- [ ] User guide covers all inspector features with screenshots
- [ ] v0.5.0 tagged and published
- [ ] All tests pass: workspace + inspector package
