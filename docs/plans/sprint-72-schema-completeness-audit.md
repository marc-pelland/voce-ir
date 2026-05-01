# Sprint 72 — Schema Completeness Audit

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Audit every node type in the IR schema for missing style, responsive, accessibility, and semantic-state fields. The S61 form regression — `FormNode` and `FormField` had no style fields, the compiler had no defaults, the output looked plain — is a class of bug. This sprint closes the door on it by going node-by-node and asking "what does this need to express to produce great output without compiler guessing?"

**Depends on:** schema (S02–S05). Independent of S65–S71.

---

## Motivation

The schema has 27 ChildUnion variants. Container has rich style fields (padding, gap, alignment, direction, fill, corner_radius, etc.). Surface has style fields. TextNode has typography fields. But:

- **FormNode/FormField** have no style fields at all (S61's bug)
- **MediaNode** has loading hints but no aspect-ratio, object-fit, object-position
- **TextNode** has font properties but no text-overflow, line-clamp, white-space
- **ContentSlot** has cache strategy but no skeleton/loading state shape
- **DataNode** has no error state shape (what to render when fetch fails)
- **GestureHandler** has no hover/pressed/disabled visual state hints
- **StateMachine** has no per-state visual override hooks

Each gap forces the compiler to guess at defaults (S64) or emit unstyled output (S61). This sprint identifies and closes the gaps.

---

## Deliverables

### 1. Per-node-type audit document

`docs/schema/COMPLETENESS_AUDIT.md` — a table for each of the 27 node types:

| Node | Style fields | Responsive fields | A11y fields | State/lifecycle fields | Gap analysis |
| --- | --- | --- | --- | --- | --- |
| Container | ✓ | partial | via SemanticNode | ✗ none | needs hover_fill, focus_ring |
| Surface | ✓ | ✗ none | via SemanticNode | ✗ none | needs disabled state |
| TextNode | ✓ partial | ✗ none | via SemanticNode | ✗ none | needs line-clamp, text-overflow |
| FormNode | ✗ none | ✗ none | semantic_node_id | submission state | needs container styling |
| FormField | ✗ none | ✗ none | autocomplete, aria | validation state | needs label/input/error styling |
| ... | ... | ... | ... | ... | ... |

For each row, an explicit gap analysis: what fields would let an authored IR produce great output without compiler guessing.

### 2. Schema additions (priority gaps)

Add the highest-impact missing fields to the .fbs files. Initial scope (close the 5 worst gaps):

- **FormField** gets: `style: FormFieldStyle?` (label_typography, input_padding, input_border, error_color)
- **FormNode** gets: `layout: FormLayout?` (max_width, gap, button_alignment)
- **MediaNode** gets: `aspect_ratio: float?`, `object_fit: ObjectFit?`, `object_position: string?`
- **TextNode** gets: `line_clamp: uint?`, `text_overflow: TextOverflow?`, `white_space: WhiteSpace?`
- **DataNode** gets: `error_state: ChildUnion?` (what to render on fetch failure), `loading_state: ChildUnion?` (skeleton)

Each addition: schema definition, regenerated bindings, validator coverage, compiler support in DOM target, tests.

### 3. Backwards compatibility

All additions are *optional* fields. Existing IRs continue to validate and compile. New fields fill gaps that previously required either compiler guessing or unstyled output.

### 4. Compiler emission for new fields

Each new field gets emission support in `voce-compiler-dom`:

- `FormFieldStyle` overrides the S64 baseline form CSS
- `MediaNode.aspect_ratio` emits `aspect-ratio: <value>` inline style
- `TextNode.line_clamp` emits `-webkit-line-clamp: <n>; display: -webkit-box; overflow: hidden`
- `DataNode.error_state` and `loading_state` compile as alternate render branches with appropriate state machine glue

Deferred for follow-up sprints: emission for iOS/Android/email targets.

### 5. Validator coverage for new fields

New rules:

- `MOT006` — `aspect_ratio` if specified must be > 0
- `FRM010` — `FormFieldStyle.input_padding` if specified must have non-negative values
- `STR006` — `DataNode.error_state` and `loading_state` referenced ChildUnion must be valid

Each new rule: hint + JSON Patch fix per S67 conventions.

### 6. Migration of existing fixtures

Update the cross-target fixtures (S68) and the production landing IR to use the new fields where they make the output better. Verify all existing fixtures still validate.

### 7. Schema versioning bump

Schema additions move us from `schema_version_minor: 1` → `schema_version_minor: 2`. Document in `CHANGELOG.md` with migration notes (none required since fields are optional).

### 8. AI bridge prompt updates

The schema context the AI bridge feeds the model needs to mention the new fields so the model can use them. Update `packages/ai-bridge/src/config/schema-context.md` (or wherever schema context lives).

---

## Acceptance Criteria

- [ ] `docs/schema/COMPLETENESS_AUDIT.md` covers all 27 node types
- [ ] At least 5 priority gaps closed via new schema fields
- [ ] All schema additions are optional (no breaking changes)
- [ ] Generated Rust + TypeScript bindings updated and committed
- [ ] Validator coverage for all new fields
- [ ] DOM compiler emits all new fields correctly
- [ ] Cross-target fixtures (S68) updated to use new fields where they improve output
- [ ] All existing fixtures still validate
- [ ] Schema version bumped to 1.2
- [ ] AI bridge schema context updated
- [ ] CHANGELOG.md entry with migration notes

---

## Risks

1. **Scope explosion.** It's tempting to add every nice-to-have field. Hold the line at 5 priority gaps; defer the rest to S72b (next round).
2. **Schema design choices ossify.** Once a field name and shape ship, changing it is breaking. Spend a day getting the names right; review with the existing research docs.
3. **AI bridge needs to discover and use new fields.** Without prompt updates, the AI keeps generating IRs without them. The S65 conversational tools should be updated in lockstep.
4. **Cross-target compiler gaps.** Adding a field to the schema doesn't make iOS/Android render it. Acceptable for v1: DOM emits, others ignore (with a documented warning). Full cross-target support is a follow-up.

---

## Out of Scope

- New node types entirely (3D, Scene3D, etc.) — different sprint
- Full per-target emission of new fields (DOM only here)
- Schema compaction (removing unused fields) — separate cleanup sprint
- Theme tokens for the new fields (e.g., a global `default_form_field_style`) — defer
