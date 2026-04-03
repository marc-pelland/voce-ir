# Sprint 04 — Schema: Accessibility & Theming

**Status:** Ready for review
**Goal:** Add accessibility and theming schemas. After this sprint, the IR can express: semantic node trees, live regions, focus traps, reduced motion preferences, design tokens, responsive breakpoints, and personalization slots.
**Depends on:** Sprint 03 (state.fbs, motion.fbs, navigation.fbs)

---

## Deliverables

1. `packages/schema/schemas/a11y.fbs` — SemanticNode, LiveRegion, FocusTrap
2. `packages/schema/schemas/theming.fbs` — ThemeNode, PersonalizationSlot, ResponsiveRule
3. Updated ChildUnion in `voce.fbs` to include new node types
4. Regenerated Rust bindings
5. Tests for semantic roles, live region politeness, theme tokens, responsive breakpoints

---

## Tasks

### 1. Design `a11y.fbs`

Key types:

- **SemanticNode** — parallel tree mirroring visual nodes. Carries: role (60+ ARIA roles), label, described_by, labelled_by, controls, keyboard focus order, heading level. The validator will enforce: every interactive ChildNode must have a SemanticNode reference.

- **LiveRegion** — declares dynamic content areas for screen reader announcements. Properties: politeness (polite/assertive/off), atomic (announce whole region or just changes), relevant (additions/removals/text/all).

- **FocusTrap** — constrains keyboard focus to a subtree (for modals, drawers). Properties: initial focus target, escape behavior, restore focus on exit.

Design decisions:
- SemanticNode is referenced BY other nodes (via `semantic_node_id: string` field on Container/Surface/TextNode) — it's not a child in the tree, it's a parallel annotation
- This means we need to add a `semantic_node_id` field to Container, Surface, TextNode, and MediaNode in layout.fbs
- The validator (Sprint 07) will check that every interactive node has a valid semantic reference

Roles to support (subset of WAI-ARIA 1.2):
```
alert, alertdialog, application, article, banner, button, cell,
checkbox, columnheader, combobox, complementary, contentinfo,
definition, dialog, directory, document, feed, figure, form, grid,
gridcell, group, heading, img, link, list, listbox, listitem, log,
main, marquee, math, menu, menubar, menuitem, menuitemcheckbox,
menuitemradio, navigation, none, note, option, presentation,
progressbar, radio, radiogroup, region, row, rowgroup, rowheader,
scrollbar, search, searchbox, separator, slider, spinbutton, status,
switch, tab, table, tablist, tabpanel, term, textbox, timer,
toolbar, tooltip, tree, treegrid, treeitem
```

### 2. Design `theming.fbs`

Key types:

- **ThemeNode** — named set of design tokens. Multiple themes can coexist (light, dark, high-contrast). Theme selection is a StateMachine transition.
  - Color palette: primary, secondary, accent, background, surface, text (primary/secondary/tertiary), error, success, warning
  - Typography scale: font families (body, display, mono), size scale, weight scale, line height scale
  - Spacing scale: base unit + multipliers
  - Border radii scale
  - Shadow definitions
  - Transition defaults (duration, easing for theme switches)

- **PersonalizationSlot** — adapts based on user context: locale, preferences, device capabilities, A/B test cohort. Declares variants and conditions.

- **ResponsiveRule** — adapts layout/content based on viewport. Explicit breakpoints with layout overrides (not CSS media queries with cascading patches).
  - Breakpoint name + min-width
  - Per-breakpoint property overrides (applied to specific nodes)

### 3. Update layout.fbs — Add Semantic References

Add to Container, Surface, TextNode, MediaNode:
```
/// Reference to a SemanticNode for accessibility.
semantic_node_id: string;
```

This is a non-breaking additive change (new optional field).

### 4. Update voce.fbs — Expand ChildUnion

Add to the union:
```
// A11y nodes
SemanticNode,
LiveRegion,
FocusTrap,
// Theming nodes
ThemeNode,
PersonalizationSlot,
ResponsiveRule
```

Also add to VoceDocument:
```
/// Application-level theme (can be overridden per-ViewRoot).
theme: ThemeNode;
```

### 5. Tests

- SemanticNode with button role, label, and keyboard focus
- LiveRegion with assertive politeness
- FocusTrap with initial focus target and escape behavior
- ThemeNode with color palette and typography scale
- ResponsiveRule with 3 breakpoints and property overrides
- ChildUnion completeness check (now 21 types)

---

## Acceptance Criteria

- [ ] `a11y.fbs` and `theming.fbs` compile via regeneration script
- [ ] Layout nodes have `semantic_node_id` field
- [ ] ChildUnion includes all 6 new node types
- [ ] Generated Rust bindings compile
- [ ] 10+ total tests passing (6 existing + 4+ new)
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] JSON round-trip works for SemanticNode and ThemeNode

---

## Open Questions

1. **Should SemanticNode be a ChildNode or only referenced by ID?** Referenced-by-ID keeps the visual tree clean but requires a separate collection. ChildNode makes it part of the tree but clutters it. Recommendation: SemanticNodes live in a `semantic_nodes: [SemanticNode]` field on ViewRoot (flat list), referenced by ID from visual nodes.

2. **Theme token structure** — flat key-value pairs vs structured (colors.primary, typography.body.size)? Structured is more type-safe but more complex in FlatBuffers. Recommendation: structured tables for core categories (ColorPalette, TypographyScale, SpacingScale).

3. **ResponsiveRule granularity** — per-node overrides (verbose but explicit) vs container-query-like rules (more powerful but harder to validate)? Recommendation: per-node overrides initially, matching how the compiler will emit media queries.
