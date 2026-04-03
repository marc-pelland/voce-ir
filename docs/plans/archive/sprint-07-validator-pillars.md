# Sprint 07 — Validator: Pillar Passes

**Status:** Planned
**Goal:** Implement the 6 pillar validation passes: accessibility, security, SEO, forms, i18n, and motion. After this sprint, the validator enforces all three non-negotiable pillars (Stability, Experience, Accessibility) and the total pass count reaches 9. Accessibility violations are compile errors, not warnings.
**Depends on:** Sprint 06 (validation engine, IR model, NodeIndex, core passes)

---

## Deliverables

1. Accessibility pass (A11Y001-A11Y009)
2. Security pass (SEC001-SEC006)
3. SEO pass (SEO001-SEO008)
4. Forms pass (FRM001-FRM008)
5. i18n pass (I18N001-I18N005)
6. Motion pass (MOT001-MOT006)
7. 12 new invalid IR fixtures
8. 18+ new tests (3 per pass)
9. Total: 9 validation passes operational

---

## Tasks

### 1. Accessibility Pass (`a11y.rs`)

Accessibility is a compile error in Voce IR. Missing semantic information blocks compilation.

| Code | Rule | Severity |
|------|------|----------|
| A11Y001 | Every interactive node (GestureHandler, FormField) must have a SemanticNode | Error |
| A11Y002 | SemanticNode `role` must be a valid ARIA role | Error |
| A11Y003 | Images (MediaNode with type=image) must have `alt` text or `aria-hidden` | Error |
| A11Y004 | GestureHandler must have a keyboard equivalent (`key_binding` or `focus_action`) | Error |
| A11Y005 | Heading hierarchy must not skip levels (h1 -> h3 without h2) | Error |
| A11Y006 | FocusTrap must have an escape mechanism (`dismiss_action` or `close_ref`) | Error |
| A11Y007 | LiveRegion must specify `politeness` (polite or assertive) | Error |
| A11Y008 | Color contrast ratio must meet WCAG AA (4.5:1 normal text, 3:1 large text) when colors are statically known | Warning |
| A11Y009 | Form fields must have associated labels (via SemanticNode `label` or `labelled_by`) | Error |

### 2. Security Pass (`security.rs`)

OWASP-informed checks for common web security issues.

| Code | Rule | Severity |
|------|------|----------|
| SEC001 | ActionNode with `auth_required: true` must specify `auth_guard` or `redirect_url` | Error |
| SEC002 | ActionNode performing mutations must have `csrf: true` or `csrf_token_ref` | Error |
| SEC003 | MediaNode `src` must not use `http://` (enforce HTTPS) | Warning |
| SEC004 | FormField of type `password` must have `autocomplete: "new-password"` or `"current-password"` | Warning |
| SEC005 | ContentSlot with `source: "user"` must have `sanitize: true` | Error |
| SEC006 | AuthContextNode must specify `session_strategy` (cookie, token, or both) | Error |

### 3. SEO Pass (`seo.rs`)

Ensures IR produces search-engine-friendly output.

| Code | Rule | Severity |
|------|------|----------|
| SEO001 | ViewRoot must have `metadata` with non-empty `title` | Error |
| SEO002 | ViewRoot `metadata.description` should be 50-160 characters | Warning |
| SEO003 | Exactly one `h1` heading per ViewRoot | Warning |
| SEO004 | All MediaNode images should have descriptive `alt` (not just "image") | Warning |
| SEO005 | ViewRoot should have `canonical` URL | Warning |
| SEO006 | If multiple languages, `metadata.alternates` must have hreflang entries | Warning |
| SEO007 | OpenGraphData should include `og:title`, `og:description`, `og:image` | Warning |
| SEO008 | StructuredData `schema_type` must be a valid schema.org type | Warning |

### 4. Forms Pass (`forms.rs`)

Validates form structure and UX patterns.

| Code | Rule | Severity |
|------|------|----------|
| FRM001 | FormNode must have at least one FormField | Error |
| FRM002 | FormNode must have a `submission` config referencing a valid ActionNode | Error |
| FRM003 | FormField `name` must be unique within its FormNode | Error |
| FRM004 | FormField of type `email` should have email ValidationRule | Warning |
| FRM005 | FormField of type `password` should have min_length ValidationRule | Warning |
| FRM006 | FormField with `required: true` must have a required ValidationRule | Error |
| FRM007 | CrossFieldValidation `field_refs` must all reference fields in the same FormNode | Error |
| FRM008 | FormField validation error messages should use LocalizedString if i18n is enabled | Warning |

### 5. i18n Pass (`i18n.rs`)

Validates internationalization completeness.

| Code | Rule | Severity |
|------|------|----------|
| I18N001 | If any TextNode uses `localized_content`, all TextNodes must use it | Warning |
| I18N002 | LocalizedString must have a non-empty `key` | Error |
| I18N003 | LocalizedString must have a non-empty `default_value` (for fallback) | Error |
| I18N004 | MessageParameter `name` must match a placeholder in the default value | Warning |
| I18N005 | If MessageCatalog is present, all LocalizedString keys must appear in it | Warning |

### 6. Motion Pass (`motion.rs`)

Validates animations and transitions are safe and accessible.

| Code | Rule | Severity |
|------|------|----------|
| MOT001 | Every Transition must have a corresponding ReducedMotion alternative | Error |
| MOT002 | Every Sequence must have a corresponding ReducedMotion alternative | Error |
| MOT003 | PhysicsBody must have `damping` > 0 (prevent infinite oscillation) | Error |
| MOT004 | Transition `duration_ms` should not exceed 5000ms | Warning |
| MOT005 | ScrollBinding with `parallax: true` must have ReducedMotion fallback | Error |
| MOT006 | GestureHandler animation responses must have ReducedMotion variants | Error |

### 7. Invalid IR Fixtures

Create in `tests/schema/invalid/`:

- `missing-semantic-interactive.voce.json` — triggers A11Y001
- `gesture-no-keyboard.voce.json` — triggers A11Y004
- `heading-skip.voce.json` — triggers A11Y005
- `auth-no-redirect.voce.json` — triggers SEC001
- `mutation-no-csrf.voce.json` — triggers SEC002
- `missing-page-title.voce.json` — triggers SEO001
- `form-no-fields.voce.json` — triggers FRM001
- `form-duplicate-names.voce.json` — triggers FRM003
- `inconsistent-i18n.voce.json` — triggers I18N001
- `empty-localized-key.voce.json` — triggers I18N002
- `transition-no-reduced-motion.voce.json` — triggers MOT001
- `physics-zero-damping.voce.json` — triggers MOT003

### 8. Tests

Unit tests in each pass module (3+ per pass, 18+ total):
- Valid IR with proper accessibility annotations passes A11Y
- Interactive node without SemanticNode triggers A11Y001
- ActionNode with auth but no guard triggers SEC001
- Mutation without CSRF triggers SEC002
- ViewRoot without title triggers SEO001
- FormNode without fields triggers FRM001
- Mixed i18n usage triggers I18N001
- Transition without ReducedMotion triggers MOT001
- Valid motion IR with all ReducedMotion alternatives passes MOT

---

## Files to Create / Modify

### Create
- `packages/validator/src/passes/a11y.rs`
- `packages/validator/src/passes/security.rs`
- `packages/validator/src/passes/seo.rs`
- `packages/validator/src/passes/forms.rs`
- `packages/validator/src/passes/i18n.rs`
- `packages/validator/src/passes/motion.rs`
- `tests/schema/invalid/missing-semantic-interactive.voce.json`
- `tests/schema/invalid/gesture-no-keyboard.voce.json`
- `tests/schema/invalid/heading-skip.voce.json`
- `tests/schema/invalid/auth-no-redirect.voce.json`
- `tests/schema/invalid/mutation-no-csrf.voce.json`
- `tests/schema/invalid/missing-page-title.voce.json`
- `tests/schema/invalid/form-no-fields.voce.json`
- `tests/schema/invalid/form-duplicate-names.voce.json`
- `tests/schema/invalid/inconsistent-i18n.voce.json`
- `tests/schema/invalid/empty-localized-key.voce.json`
- `tests/schema/invalid/transition-no-reduced-motion.voce.json`
- `tests/schema/invalid/physics-zero-damping.voce.json`

### Modify
- `packages/validator/src/passes/mod.rs` — register all 6 new passes
- `packages/validator/src/engine.rs` — update default pass ordering (structural -> references -> state_machine -> a11y -> security -> seo -> forms -> i18n -> motion)

---

## Acceptance Criteria

- [ ] Accessibility pass catches all 9 A11Y codes on invalid input
- [ ] Security pass catches all 6 SEC codes on invalid input
- [ ] SEO pass catches all 8 SEO codes on invalid input
- [ ] Forms pass catches all 8 FRM codes on invalid input
- [ ] i18n pass catches all 5 I18N codes on invalid input
- [ ] Motion pass catches all 6 MOT codes on invalid input
- [ ] 12 new invalid IR fixture files in `tests/schema/invalid/`
- [ ] 18+ new tests passing (30+ total across Sprints 06-07)
- [ ] Valid reference IR produces zero errors across all 9 passes
- [ ] A11Y violations produce `Severity::Error`, not `Warning`
- [ ] Motion violations (missing ReducedMotion) produce `Severity::Error`
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] Every public type and function has `///` doc comments

---

## Notes

- Pass ordering matters: structural and reference passes run first because pillar passes assume valid structure. If structural fails, later passes may panic on missing fields. The engine should short-circuit if structural errors are fatal.
- A11Y008 (color contrast) only applies when both foreground and background colors are statically known in the IR. Dynamic/themed colors get a pass at validation time and must be checked at runtime.
- SEC002 (CSRF) applies to ActionNodes where `method` is POST/PUT/DELETE/PATCH. GET actions are exempt.
- The 42 total error codes across 9 passes represent the minimum rule set. Additional codes can be added in later sprints without architectural changes.
