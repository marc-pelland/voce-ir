# Accessibility Validator Rules (A11Y*)

Every rule below runs in the `accessibility` validation pass
(`packages/validator/src/passes/a11y.rs`). Severities: **error** blocks
validation; **warning** is surfaced but non-blocking. Each diagnostic
carries a `hint` (see source `CodeMeta`) and a docs URL (S67).

`A11Y002` is intentionally unused (historical gap; never reassigned —
codes are a stable contract).

| Code | Sev | Checks | Fix |
| --- | --- | --- | --- |
| A11Y001 | error | Interactive node has no `SemanticNode` | Add a `SemanticNode` with the right `role`, reference via `semantic_node_id` |
| A11Y003 | error | `MediaNode` has no alt and isn't decorative | Set `alt`, add a labelled `semantic_node_id`, or `decorative: true` |
| A11Y004 | error | Heading hierarchy skips a level (h1→h3) | Demote the heading or add the missing intermediate level |
| A11Y005 | error | Form field has no label/aria-label | Set `label`, or `aria_label` if visually hidden |
| A11Y006 | error | Link/button has no accessible text | Add text content, or a `SemanticNode` with a `label` |
| A11Y007 | error | Text/background contrast fails WCAG 2.2 AA | Lighten text, darken background, or mark surrounding Surface decorative |
| A11Y008 | warn | `SemanticNode` uses a positive `tab_index` | Use `0` (DOM order) or `-1` (programmatic); restructure IR if a custom order is needed |
| A11Y009 | warn | Interactive target < 24×24 CSS px | Increase padding, set `min_width`/`min_height`, or move interactivity to a larger ancestor |
| A11Y010 | warn | Dynamic content with no `LiveRegion` | Add a `LiveRegion` (polite for results, assertive for errors) |

## Notes on the warnings

- **A11Y008 / A11Y010** are warnings because a valid design can have
  defensible exceptions; they signal review, not a hard stop.
- **A11Y009** uses a padding/min-size heuristic. WCAG 2.5.8 exempts
  links embedded in a sentence of text ("inline" exception); the
  heuristic can over-fire on bare inline navigation links. When that
  happens it is a known heuristic limit, documented per fixture in
  `EVIDENCE.md`, not a defect in the content.

## Contrast specifics (A11Y007)

Computed via `packages/validator/src/contrast.rs` (WCAG relative
luminance + contrast ratio). Thresholds: 4.5:1 body text, 3:1 large
text (≥18pt, or ≥14pt bold). Only fires when an explicit ancestor
background exists in the IR — implicit page defaults are skipped so
dark-mode-authored text doesn't false-fire (documented known gap;
declare an explicit wrapper `fill` for stricter enforcement).
