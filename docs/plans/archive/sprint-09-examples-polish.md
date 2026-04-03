# Sprint 09 — Examples & Polish

**Status:** Planned
**Goal:** Create the reference landing page IR using 15+ node types, build 5 intent-IR training pairs for Phase 3 RAG, expand test fixtures to 20+ invalid and 5+ valid, ensure JSON round-trip fidelity, reach 30+ total tests, and polish diagnostic messages for clarity. After this sprint, the validator is battle-tested and the example IR demonstrates the full expressive range of the schema.
**Depends on:** Sprint 08 (working CLI with validate, inspect, json2bin, bin2json)

---

## Deliverables

1. Reference landing page IR (`.voce.json`) — 15+ node types, realistic structure
2. 5 intent-IR training pairs in `examples/intents/`
3. 20+ invalid IR fixtures (cumulative across all sprints)
4. 5+ valid IR fixtures (cumulative)
5. JSON round-trip verification for all valid fixtures
6. 30+ total tests passing
7. Diagnostic message polish — clear, actionable, with fix suggestions
8. `examples/` README explaining each example

---

## Tasks

### 1. Reference Landing Page IR

Create `examples/landing-page/landing-page.voce.json` — a realistic SaaS landing page that exercises the full schema.

Required node types (15+ minimum):
- **ViewRoot** — with PageMetadata (title, description, OG, canonical)
- **Container** — header, hero section, features grid, testimonials, footer
- **Surface** — hero background, feature cards
- **TextNode** — headings (h1-h3), body text, button labels
- **MediaNode** — hero image, logo, feature icons
- **GestureHandler** — CTA button, nav links, mobile menu toggle
- **SemanticNode** — on every interactive element and landmark
- **StateMachine** — mobile nav toggle (open/closed), FAQ accordion
- **DataNode** — testimonial data source
- **ComputeNode** — derived display values
- **Transition** — state transitions with easing
- **ReducedMotion** — alternatives for all animations
- **ThemeNode** — light/dark theme support
- **ResponsiveRule** — mobile/tablet/desktop breakpoints
- **FormNode** — newsletter signup with email field and submit
- **FormField** — email input with validation
- **ActionNode** — form submission endpoint
- **LocalizedString** — at least 3 strings with localization keys

Structure: ~40-60 nodes, 4-6 levels deep, realistic for a production landing page.

### 2. Intent-IR Training Pairs

Create 5 pairs in `examples/intents/`, each with an intent file and corresponding IR:

| File | Intent | Complexity |
|------|--------|------------|
| `01-hero-section.intent.md` + `.voce.json` | "A hero section with headline, subtext, and CTA button" | Simple (5-8 nodes) |
| `02-feature-grid.intent.md` + `.voce.json` | "A 3-column feature grid with icons, titles, and descriptions" | Medium (12-15 nodes) |
| `03-contact-form.intent.md` + `.voce.json` | "A contact form with name, email, message, and submit" | Medium (10-12 nodes) |
| `04-pricing-table.intent.md` + `.voce.json` | "A pricing table with 3 tiers, feature lists, and CTA buttons" | Complex (20-25 nodes) |
| `05-auth-flow.intent.md` + `.voce.json` | "A login form with email/password, social login buttons, and forgot password link" | Complex (15-20 nodes) |

Each `.intent.md` file contains the natural language description an AI would receive. Each `.voce.json` is the expected IR output. These pairs will be used for RAG training and few-shot prompting in Phase 3.

### 3. Additional Invalid IR Fixtures

Add fixtures to reach 20+ total in `tests/schema/invalid/`. New fixtures beyond Sprints 06-07:

- `form-submission-no-action.voce.json` — triggers FRM002
- `password-no-autocomplete.voce.json` — triggers SEC004
- `user-content-no-sanitize.voce.json` — triggers SEC005
- `no-h1-heading.voce.json` — triggers SEO003
- `image-generic-alt.voce.json` — triggers SEO004
- `focus-trap-no-escape.voce.json` — triggers A11Y006
- `form-label-missing.voce.json` — triggers A11Y009
- `sequence-no-reduced-motion.voce.json` — triggers MOT002

### 4. Additional Valid IR Fixtures

Create in `tests/schema/valid/`:

- `minimal-valid.voce.json` — smallest possible valid IR (ViewRoot + one TextNode + metadata)
- `full-a11y.voce.json` — valid IR with comprehensive accessibility annotations
- `stateful-form.voce.json` — form with state machine, validation, submission
- `multi-route.voce.json` — multi-page IR with RouteMap and route transitions
- `i18n-complete.voce.json` — fully localized IR with MessageCatalog

### 5. JSON Round-Trip Verification

For every valid fixture:
1. `voce json2bin fixture.voce.json -o /tmp/fixture.voce`
2. `voce bin2json /tmp/fixture.voce -o /tmp/fixture.roundtrip.voce.json`
3. Compare original and round-tripped JSON for semantic equality

Create a test that automates this for all valid fixtures using `glob` to discover them.

### 6. Diagnostic Message Polish

Review and improve all 42+ diagnostic messages across 9 passes:

- Every message should state **what** is wrong and **how** to fix it
- Include the expected value when applicable ("expected at least 1 child, found 0")
- Use consistent terminology (always "node" not "element", always "field" not "property")
- Add `hint` field to Diagnostic struct for optional fix suggestions

Examples of polished messages:
- Before: `"TextNode must have non-empty content"`
- After: `"TextNode must have non-empty content or localized_content. Add a content string or a localized_content with key and default_value."`

- Before: `"Missing SemanticNode"`
- After: `"Interactive node (GestureHandler) requires a SemanticNode for accessibility. Add a SemanticNode with role and label as a sibling or parent."`

### 7. Test Expansion

Reach 30+ total tests:
- Round-trip tests for each valid fixture (5 tests)
- New invalid fixture tests (8 tests)
- Landing page validates cleanly (1 test)
- Intent-IR pairs validate cleanly (5 tests)
- Inspect output snapshot tests for landing page (1 test)
- Diagnostic message content assertions (3+ tests)

---

## Files to Create / Modify

### Create
- `examples/landing-page/landing-page.voce.json`
- `examples/intents/01-hero-section.intent.md`
- `examples/intents/01-hero-section.voce.json`
- `examples/intents/02-feature-grid.intent.md`
- `examples/intents/02-feature-grid.voce.json`
- `examples/intents/03-contact-form.intent.md`
- `examples/intents/03-contact-form.voce.json`
- `examples/intents/04-pricing-table.intent.md`
- `examples/intents/04-pricing-table.voce.json`
- `examples/intents/05-auth-flow.intent.md`
- `examples/intents/05-auth-flow.voce.json`
- `tests/schema/invalid/form-submission-no-action.voce.json`
- `tests/schema/invalid/password-no-autocomplete.voce.json`
- `tests/schema/invalid/user-content-no-sanitize.voce.json`
- `tests/schema/invalid/no-h1-heading.voce.json`
- `tests/schema/invalid/image-generic-alt.voce.json`
- `tests/schema/invalid/focus-trap-no-escape.voce.json`
- `tests/schema/invalid/form-label-missing.voce.json`
- `tests/schema/invalid/sequence-no-reduced-motion.voce.json`
- `tests/schema/valid/minimal-valid.voce.json`
- `tests/schema/valid/full-a11y.voce.json`
- `tests/schema/valid/stateful-form.voce.json`
- `tests/schema/valid/multi-route.voce.json`
- `tests/schema/valid/i18n-complete.voce.json`
- `tests/integration/roundtrip.rs`
- `tests/integration/examples.rs`
- `examples/README.md`

### Modify
- `packages/validator/src/errors.rs` — add `hint: Option<String>` to Diagnostic
- `packages/validator/src/passes/*.rs` — polish all diagnostic messages, add hints
- `packages/validator/src/formatter.rs` — render hints in terminal and JSON output

---

## Acceptance Criteria

- [ ] Landing page IR uses 15+ distinct node types
- [ ] Landing page IR validates with zero errors across all 9 passes
- [ ] `voce inspect` on landing page shows ~40-60 nodes, 4-6 depth levels
- [ ] 5 intent-IR pairs exist, each with `.intent.md` and `.voce.json`
- [ ] All intent-IR `.voce.json` files validate cleanly
- [ ] 20+ invalid IR fixtures exist in `tests/schema/invalid/`
- [ ] 5+ valid IR fixtures exist in `tests/schema/valid/`
- [ ] JSON round-trip produces semantically identical output for all valid fixtures
- [ ] 30+ total tests passing
- [ ] Every diagnostic message includes what is wrong and how to fix it
- [ ] Diagnostic `hint` field populated for at least 20 error codes
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes

---

## Notes

- The landing page IR is the flagship example. It should feel like a real SaaS landing page, not a toy. Use realistic text content, plausible data sources, and proper accessibility throughout.
- Intent-IR pairs are the foundation for Phase 3 RAG training. The intents should be written as a non-technical user would describe what they want. The IR should be the ideal output a well-trained AI would produce.
- Round-trip fidelity is critical for the serialization bridge. Field ordering differences are acceptable, but values, types, and structure must be identical.
- The `hint` field is optional and should only be added where a concrete fix can be suggested. Generic hints like "fix this" are worse than no hint.
