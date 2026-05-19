# WCAG 2.2 AA → Voce Rule Mapping

Which WCAG 2.2 success criteria are enforced at compile time, by which
rule, and which are explicitly out of automated scope (human or S89).

## Enforced by the validator

| WCAG 2.2 SC | Level | Voce rule | Enforcement |
| --- | --- | --- | --- |
| 1.1.1 Non-text Content | A | A11Y003 | error |
| 1.3.1 Info and Relationships | A | A11Y004, A11Y005 | error |
| 1.4.3 Contrast (Minimum) | AA | A11Y007 | error |
| 2.4.3 Focus Order | A | A11Y008 | warning |
| 2.4.4 Link Purpose (In Context) | A | A11Y006 | error |
| 2.4.6 Headings and Labels | AA | A11Y004, A11Y005 | error |
| 2.5.8 Target Size (Minimum) | AA | A11Y009 | warning (heuristic) |
| 3.3.2 Labels or Instructions | A | A11Y005 | error |
| 4.1.2 Name, Role, Value | A | A11Y001, A11Y006 | error |
| 4.1.3 Status Messages | AA | A11Y010 | warning |

## Partially covered / known gaps

| WCAG 2.2 SC | Status |
| --- | --- |
| 1.4.3 Contrast | Skipped when no explicit ancestor background is declared (light/dark-mode false-fire avoidance — declare a wrapper `fill` for strict mode) |
| 2.5.8 Target Size | Heuristic (padding/min-size); inline-link exception not auto-detected |
| 1.4.11 Non-text Contrast | Not yet computed (UI component/graphic contrast) — candidate rule |
| 2.4.7 Focus Visible | Emitted via baseline `:focus-visible` CSS, not validator-enforced |

## Out of automated scope (human or S89)

These cannot be decided from static IR and are owned by manual review
(`MANUAL_TESTING.md`) or runtime audit automation (**S89**):

- 1.4.10 Reflow, 1.4.12 Text Spacing — runtime/viewport behavior
- 2.1.1 Keyboard (full operability) — runtime interaction
- 2.4.1 Bypass Blocks — verified via runtime audit
- 3.x cognitive criteria (clear language, error recovery UX)
- Anything requiring a rendered DOM + AT (axe-core / screen reader)

S89 (axe-core / Lighthouse / Pa11y in deployment adapters) is the
runtime cross-check for the rendered-DOM criteria; this table is its
gap list.
