# Accessibility Evidence (S82 D8)

The receipt is **machine-checked, not hand-written.** The test
`packages/validator/tests/accessibility_evidence.rs` validates every
fixture in the reference corpus and fails if any produces an
error-severity diagnostic. It runs in `cargo test --workspace` and the
`cross-target-parity` adjacent CI, so the guarantee below cannot
silently rot — a regressing or newly-added failing fixture breaks CI.

> Hand-written per-fixture `.md` evidence was deliberately rejected: it
> drifts from reality the moment a rule or fixture changes. A standing
> test is the honest form of "receipt."

## Guarantee

**All 13 reference fixtures validate with zero errors.** Accessibility
is a compile error in Voce; a shipped fixture failing `voce validate`
would contradict the pillar, so the corpus is held to the same bar as
user IR.

Regenerate / inspect:

```
cargo test -p voce-validator --test accessibility_evidence
# per-fixture detail:
voce validate tests/fixtures/<name>.voce.json --format json
```

## Per-fixture status (non-blocking warnings only)

Errors: **0 across the corpus.** Remaining warnings, with rationale —
each is a deliberate, documented non-defect:

| Fixture | Warnings | Why this is acceptable |
| --- | --- | --- |
| container-grid / -row / decorative-surface / gesture-tap / media-image / semantic-a11y / state-machine / surface-card / theme-dark | SEO003 | Single-purpose component fixtures, not full pages — a bare grid/surface legitimately has no `h1`. SEO003 is a page-completeness warning, not an a11y error. |
| form-contact | SEO003, A11Y010 | Minimal contact form with no surrounding page or status region. A11Y010 (no `LiveRegion`) is the *correct* signal for a bare form — real forms should add one; the fixture exists to exercise form compilation, and the warning documents the gap rather than hiding it. |
| links-and-nav | A11Y009 | The 3 inline nav links trip the touch-target heuristic. WCAG 2.5.8 has an explicit exception for links in a line of text; the validator's padding/min-size heuristic cannot detect that context (see `RULES.md`). Known heuristic limit, not a content defect. |
| nested-layout, text-heading | none | Fully clean. |

## What this evidence does *not* claim

Zero validator errors ≠ verified accessible experience. It means the
**statically decidable** WCAG criteria (`WCAG_MAPPING.md`) hold. The
rendered-DOM and lived-experience criteria are covered by runtime audit
automation (**S89**, roadmap) and the human pass in `MANUAL_TESTING.md`.
This file is scoped honestly to the compile-time guarantee.

## History

Three fixtures (`form-contact`, `gesture-tap`, `links-and-nav`) shipped
`valid=false` before S82 closeout — A11Y001/004/006/007 plus a latent
forms defect (`form-contact` used an obsolete `action` shape instead of
`submission`, and a `keyboard_equivalent` typo for `keyboard_key`).
All were fixed; snapshots were reviewed (only the intended color /
anchor / form-semantic deltas) and accepted. This is why the standing
test exists — to prevent that class of drift from recurring.
