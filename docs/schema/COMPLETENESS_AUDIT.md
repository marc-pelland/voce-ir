# Schema Completeness Audit

**Sprint:** S72 §1
**Last updated:** 2026-05-01
**Scope:** Identify gaps in the IR schema that force the compiler to either guess at defaults (S64 territory) or produce unstyled output (the form regression that motivated S64). Per-node analysis based on `packages/schema/schemas/*.fbs`.

---

## Headline finding

**The schema is far more complete than the S61 form regression suggested.** Container, Surface, TextNode, and MediaNode already declare comprehensive style/sizing/visual fields in their `.fbs` definitions. The form regression was caused by exactly one node type — `FormField` — having zero visual customization fields.

The real gaps are now:

1. **`FormField` has no style fields at all** (the original motivation for this audit) — confirmed.
2. **The validator's serde IR struct (`packages/validator/src/ir.rs`) deserializes only a fraction of each schema's declared fields.** Fields not deserialized are not validated, so an IR with invalid values for those fields ships to the compiler without warning.
3. **Compiler emit coverage is unknown for several fields** — needs per-emit-path verification.

---

## Per-node analysis

Legend:
- ✓ Schema declares the capability
- ⚠ Schema declares it; validator does not deserialize it (no validation)
- ✗ Schema does not declare the capability — genuine gap

### Container (layout.fbs)

| Capability | Status |
| --- | --- |
| Layout (Stack/Flex/Grid, direction, alignment, gap) | ✓ |
| Padding | ✓ |
| Sizing (width/height + min/max constraints) | ⚠ |
| Wrap, grid_columns, grid_rows | ⚠ |
| Overflow + clip | ⚠ |
| Positioning (absolute, top/right/bottom/left, z_index) | ⚠ |
| Visual (background, border, corner_radius, shadow, opacity) | ⚠ |
| Responsive (per-breakpoint overrides) | ✗ — needs `responsive: ResponsiveRule[]` link |
| Hover/focus/active state styles | ✗ |

**Verdict:** schema is ~95% complete; validator just doesn't see most of it.

### Surface (layout.fbs)

| Capability | Status |
| --- | --- |
| Visual (fill, stroke, stroke_width, corner_radius, shadow, opacity, border) | ⚠ |
| Sizing + padding | ⚠ |
| Link (href, target) | ✓ |
| Decorative flag | ✓ |
| Hover state | ✗ |

**Verdict:** schema is fine; validator/compiler coverage of fill/border/etc. should be audited.

### TextNode (layout.fbs)

| Capability | Status |
| --- | --- |
| Typography (font_family, size, weight, line_height, letter_spacing) | ⚠ |
| Text alignment, decoration | ⚠ |
| **Text overflow + max_lines** | ⚠ (declared `text_overflow: TextOverflow`, `max_lines: int32` — not in validator IR) |
| Color, opacity | ⚠ |
| Heading level | ✓ |
| Link (href, target) | ✓ |
| Localized content | ✓ |
| Hover state | ✗ |

**Verdict:** schema covers typography well. Validator doesn't read `text_overflow`/`max_lines`. Compiler emit coverage uncertain.

### MediaNode (layout.fbs)

| Capability | Status |
| --- | --- |
| src, alt, decorative | ✓ |
| **Dimensions (width, height, aspect_ratio)** | ⚠ (declared, not in validator IR) |
| **object_fit** | ⚠ (declared `object_fit: ObjectFit = Cover`, not in validator IR) |
| Loading strategy (lazy, eager, above_fold) | ⚠ |
| Corner radius, opacity | ⚠ |
| **srcset_widths + sizes** | ⚠ (declared for responsive images, not in validator IR) |

**Verdict:** schema is comprehensive (CLS-prevention fields are even there). Validator missing aspect_ratio + object_fit, srcset/sizes — all useful for the compiler to emit and for validation to enforce.

### FormNode (forms.fbs)

| Capability | Status |
| --- | --- |
| fields, submission, semantic_node_id | ✓ |
| validation_mode | ⚠ (declared, not in validator IR) |
| autosave | ⚠ |
| Layout (stacked vs. inline, max-width, gap) | ✗ — schema has no FormLayout struct |

**Verdict:** **genuine gap on layout.** Currently FormNode emits with the S61 baseline form CSS (`flex-column, gap 14px, max-width 520px`). Authors can't customize without overriding the global stylesheet, which the IR doesn't support.

### FormField (forms.fbs) — **biggest gap**

| Capability | Status |
| --- | --- |
| name, field_type, label, placeholder, description | ✓ |
| validations + async_validations | ✓ |
| options (Select/Radio), accept (File), step (Number) | ✓ |
| visible_when, disabled_when | ✓ |
| autocomplete | ✓ |
| **All visual styling (border, padding, background, font)** | ✗ |
| **Per-state styling (focus, error, disabled)** | ✗ |
| Inline help icon, prefix/suffix slot | ✗ |
| Group / fieldset wrapper at the IR level | ⚠ (FormFieldGroup table exists; not connected to FormField) |

**Verdict:** FormField is purely data. No style props at all. The S61 form regression was inevitable given this. **This is the schema gap that S72's actual schema-modification sprint should fill first.**

### DataNode (data.fbs)

| Capability | Status |
| --- | --- |
| source (Rest/GraphQL/etc.), auth_required, cache strategy | ⚠ |
| **error_state**: what to render when fetch fails | ✗ |
| **loading_state**: skeleton / placeholder shape | ✗ — has `loading_state_machine` (a state ref) but no inline skeleton IR |
| Retry policy | ✗ |

**Verdict:** **genuine gap on lifecycle states.** Today the compiler must emit a generic placeholder during load and a generic error message on failure — designers can't customize without authoring a separate StateMachine + multiple Containers.

### ContentSlot (data.fbs)

| Capability | Status |
| --- | --- |
| content_key, cache_strategy | ✓ |
| Skeleton during load | ✗ |
| Author/edit metadata | ✗ |

### GestureHandler (motion.fbs)

| Capability | Status |
| --- | --- |
| Gesture types, keyboard equivalent | ✓ |
| **Hover/pressed/disabled visual states for the target node** | ✗ — handler is purely event-binding; per-state visuals require StateMachine + style overrides |

### StateMachine (state.fbs)

| Capability | Status |
| --- | --- |
| States, transitions, guards, effects | ✓ |
| Per-state visual override hooks (e.g. `state_styles: { idle: …, loading: … }`) | ✗ |

### Other notable gaps

- **No global `responsive` field** on Container or Surface — breakpoint-specific style overrides require duplicating the entire subtree per breakpoint. The schema has `ResponsiveRule` in `theming.fbs` but it isn't widely connected to layout nodes.
- **No `disabled` boolean** on Surface (links and clickables) — disabled-state visuals require ad-hoc state machines.
- **No `tabindex` field on Container/Surface** — focus order can't be customized at the IR level beyond whatever the compiler emits.

---

## Recommended priority for S72 §2 (schema additions)

If the next round of `.fbs` changes can fill 5 gaps, these have the highest leverage:

1. **`FormField.style: FormFieldStyle?`** — closes the biggest single gap. New `FormFieldStyle` table with: `font` (Font ref), `padding` (EdgeInsets), `border` (BorderSides), `corner_radius` (CornerRadii), `background` (Color), `text_color` (Color). Per-state variants (`focus_style`, `error_style`, `disabled_style`) optional but valuable.
2. **`FormNode.layout: FormLayout?`** — `gap`, `max_width`, `direction`, `button_alignment`. Lets authors decide stacked vs. inline forms.
3. **`MediaNode` validator/compiler coverage** for `aspect_ratio`, `object_fit`, `srcset_widths`, `sizes` — declared in schema, just unused. Probably zero schema change required, just plumbing.
4. **`DataNode.error_state` + `loading_state`** as inline `ChildNode?` references — designers express what to show during fetch / on failure inline rather than via a separate StateMachine.
5. **Per-state visuals** on interactive nodes (Surface with href, GestureHandler target) — at minimum a `disabled: bool` flag + optional `disabled_style: StyleOverride`.

Each of these is 1-3 days of `.fbs` change + bindings regenerate + validator pass + compiler emit + tests. The first two close the form gap class permanently; the others move responsive/state expressivity into the IR rather than requiring StateMachine boilerplate.

---

## Validator coverage gaps (no schema change required)

These fields exist in the schema today but aren't deserialized by `packages/validator/src/ir.rs`. Adding them to the validator IR struct lets new validation rules check them and lets the compiler trust validator output:

- `Container`: width/height/min/max, overflow, position, top/right/bottom/left, z_index, opacity, background, border, corner_radius, shadow, gap, padding, layout (already partial), main_align, cross_align, wrap, grid_columns, grid_rows
- `Surface`: fill, stroke, stroke_width, padding, sizing, corner_radius, shadow, opacity, border
- `TextNode`: font_family, font_size, font_weight, line_height, letter_spacing, text_align, text_overflow, text_decoration, max_lines, color, opacity, target
- `MediaNode`: width, height, aspect_ratio, object_fit, loading, corner_radius, opacity, srcset_widths, sizes, above_fold

Adding these is mechanical (each is `pub field: Option<Type>`); the win is enabling rules like "MediaNode missing aspect_ratio causes CLS" or "TextNode font_weight outside FontWeight enum is invalid."

---

## Out of scope for the audit

- Per-target compiler coverage of these fields (DOM, iOS, Android, Email, WebGPU, WASM, Hybrid). That's S68 (cross-target parity) territory.
- Schema versioning strategy for additive changes — current convention is bumping `schema_version_minor`. Documented in CHANGELOG.
- Migration tooling for projects on older schema versions — not yet relevant; only one shipped version.
