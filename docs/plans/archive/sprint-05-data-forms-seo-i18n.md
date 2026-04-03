# Sprint 05 — Schema: Data, Forms, SEO, i18n

**Status:** Outlined (will be detailed before starting)
**Goal:** Complete the IR schema with the remaining domains. After this sprint, the schema covers everything needed for a production web application: data fetching/mutations, forms with validation, SEO metadata, and internationalization. The ChildUnion is complete.
**Depends on:** Sprint 04 (a11y.fbs, theming.fbs)

---

## Deliverables

1. `packages/schema/schemas/data.fbs` — ActionNode, SubscriptionNode, AuthContextNode, ContentSlot, ContentSource, RichTextNode, ContentModel
2. `packages/schema/schemas/forms.fbs` — FormNode, FormField, ValidationRule, CrossFieldValidation, FormSubmission
3. `packages/schema/schemas/seo.fbs` — PageMetadata, OpenGraphData, TwitterCardData, StructuredData (on ViewRoot)
4. `packages/schema/schemas/i18n.fbs` — LocalizedString, MessageParameter, FormatOptions, MessageCatalog
5. Final ChildUnion with all node types (~25+ types)
6. Tests for ActionNode, FormNode, PageMetadata, LocalizedString

---

## Key Design Notes

### data.fbs

References: `docs/research/DATA_INTEGRATION.md`

- **ActionNode** — the server action equivalent. Declares endpoint, input/output types, optimistic strategy, cache invalidation targets, error handling, CSRF/auth annotations. The compiler emits TanStack Query mutation calls.
- **SubscriptionNode** — real-time data (WebSocket/SSE/polling). Target DataNode, connection config, update strategy.
- **AuthContextNode** — auth provider config, session strategy, user schema reference. Global ContextNode for auth state.
- **ContentSlot** — CMS content injection. Cache strategy (static/isr/dynamic) determines compiler behavior.
- **RichTextNode** — structured rich text (paragraph, heading, list, image, code, quote blocks with inline spans and marks).

### forms.fbs

References: `docs/research/FORMS_SEO_I18N.md`

- **FormNode** — form coordinator with fields, validation mode (on_submit/on_blur/on_change/on_blur_then_change), submission config, autosave.
- **FormField** — field type (text/email/password/number/select/checkbox/radio/textarea/file/date/hidden), label, placeholder, validations, async validations, visibility condition, autocomplete hint.
- **ValidationRule** — required/min_length/max_length/pattern/min/max/email/url/custom with localized error message.
- **FormSubmission** — references ActionNode + encoding (url_encoded/multipart/json) + progressive enhancement flag.

### seo.fbs

References: `docs/research/FORMS_SEO_I18N.md`

- **PageMetadata** — added as field on ViewRoot. Title, description, canonical, robots, OG, Twitter card, alternates (hreflang), custom meta.
- **StructuredData** — JSON-LD blocks (schema type + properties DataNode reference).

### i18n.fbs

References: `docs/research/FORMS_SEO_I18N.md`

- **LocalizedString** — message key + default value + parameters + description for translators. Used as alternative to raw string in TextNode content.
- **MessageParameter** — typed interpolation (string/number/date/currency/plural/select).
- **FormatOptions** — locale-specific number/date/currency formatting config.
- **MessageCatalog** — companion file format (one per locale) with messages and fallback locale.

### Schema integration

- TextNode gains `localized_content: LocalizedString` field (alternative to `content: string`)
- ViewRoot gains `metadata: PageMetadata`, `structured_data: [StructuredData]`
- FormField validation messages use LocalizedString
- ChildUnion gets: ActionNode, SubscriptionNode, AuthContextNode, ContentSlot, RichTextNode, FormNode, PersonalizationSlot (if not already added in S04)

---

## Acceptance Criteria

- [ ] All 4 new `.fbs` files compile via regeneration script
- [ ] ChildUnion includes all node types (25+)
- [ ] ViewRoot has metadata and structured_data fields
- [ ] TextNode has localized_content field
- [ ] Generated Rust bindings compile
- [ ] 15+ total tests passing
- [ ] JSON round-trip for ActionNode, FormNode, LocalizedString
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] Schema is feature-complete for Phase 1

---

## After Sprint 05

The schema phase is complete. All IR node types are defined. The remaining Phase 1 sprints focus on:
- **S06-S07:** Validator passes (using the complete schema)
- **S08:** CLI tooling (working subcommands)
- **S09:** Example IRs (reference landing page)
- **S10:** Documentation and v0.1.0 release
