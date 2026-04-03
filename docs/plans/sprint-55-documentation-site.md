# Sprint 55 — Documentation Site (mdBook)

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Build a proper documentation website using mdBook with getting started guide, CLI reference, full schema reference for every node type, architecture guide, and contributor docs. Deploy to docs.voce-ir.xyz.

**Depends on:** Sprint 54 (tests validate all node types, confirming schema is stable)

---

## Deliverables

- mdBook project in `docs/site/` with `book.toml` configuration
- **Getting Started** chapter:
  - Installation (cargo install, npm, pre-built binaries)
  - First IR file (hand-authored hello world)
  - First compilation (`voce compile --target dom`)
  - First AI generation (using SDK with API key)
- **CLI Reference** chapter:
  - `voce compile` — all flags, targets, examples
  - `voce validate` — validation passes, error codes
  - `voce deploy` — adapter options, configuration
  - `voce inspect` — IR inspection and debugging
- **Schema Reference** chapter (one section per node type):
  - Layout: ViewRoot, Container, Surface, TextNode, MediaNode
  - State: StateMachine, DataNode, ComputeNode, EffectNode, ContextNode
  - Motion: Transition, Sequence, GestureHandler, ScrollBinding, PhysicsBody
  - Navigation: RouteMap, RouteTransition
  - Accessibility: SemanticNode, LiveRegion, FocusTrap, ReducedMotion
  - Theming: ThemeNode, PersonalizationSlot, ResponsiveRule
  - Each section: field table, valid values, example IR fragment, common patterns
- **Architecture Guide** chapter:
  - Pipeline overview (generate -> validate -> compile -> deploy)
  - FlatBuffers IR format rationale
  - Compiler architecture (shared trait, per-target codegen)
  - Validation passes explained
- **Style Pack Guide** chapter: creating custom style packs, built-in packs reference
- **Contributing Guide** chapter: dev setup, code conventions, PR process, adding a new compiler target
- Custom CSS theme matching Voce IR brand (dark mode default, clean typography)
- GitHub Actions workflow: build mdBook on push to `docs/`, deploy to Cloudflare Pages
- `mdbook-mermaid` preprocessor for architecture diagrams

## Acceptance Criteria

- [ ] `mdbook build` in `docs/site/` produces a valid static site with no broken links
- [ ] Every node type in the FlatBuffers schema has a corresponding reference section
- [ ] Getting started guide is followable by a new user with only Rust and cargo installed
- [ ] CLI reference documents every subcommand and flag currently implemented
- [ ] At least 3 architecture diagrams rendered via Mermaid (pipeline, compiler, validation)
- [ ] Search works (mdBook built-in search indexes all content)
- [ ] Custom theme renders correctly in Chrome, Firefox, Safari
- [ ] Dark mode is default, light mode toggle works
- [ ] Site deploys to Cloudflare Pages and is accessible at target URL
- [ ] All code examples in docs are tested (extracted and compiled as part of `cargo test`)
- [ ] Page load time under 1 second on broadband
