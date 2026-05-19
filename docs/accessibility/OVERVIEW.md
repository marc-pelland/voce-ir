# Accessibility in Voce IR

Accessibility is one of Voce's three non-negotiable pillars, and the
project takes a deliberately strong position: **accessibility is a
compile error, not a lint.** An IR that omits required semantic
information does not produce a degraded page — it fails validation.

## What this means in practice

- A missing `SemanticNode` on an interactive node is a validation
  **error** (`A11Y001`), not a warning. The build stops.
- Images must resolve their accessible disposition explicitly: `alt`
  text, a labelled `SemanticNode`, or `decorative: true`. Silence is
  not a valid state (`A11Y003`).
- Color contrast is computed at compile time from the IR, not left to
  chance (`A11Y007`, WCAG 2.2 AA).
- These checks run in the same validator every other pass uses, so
  they cannot be skipped, disabled per-build, or "fixed later."

## Coverage model

Voce verifies what is **statically decidable from the IR** at compile
time — structure, semantics, contrast, focus order, target size, status
regions. It does not, and cannot, replace human testing of the lived
experience with a screen reader. The split is explicit:

| Layer | Owner | Where |
| --- | --- | --- |
| Compile-time WCAG checks | Voce validator | `RULES.md`, this dir |
| WCAG SC → rule mapping | this documentation | `WCAG_MAPPING.md` |
| Continuous corpus evidence | machine-checked test | `EVIDENCE.md` |
| Runtime audit (axe-core / Lighthouse) | **S89** (roadmap) | deployment adapters |
| Screen-reader / cognitive review | human | `MANUAL_TESTING.md` |

Runtime audit automation (axe-core via headless browser, Lighthouse,
Pa11y) is intentionally **out of S82's scope** and tracked as **S89 —
A11y Audit Automation** in `docs/plans/MASTER_PLAN.md`. S82 delivers the
compile-time-verifiable guarantee; S89 adds the runtime cross-check.

## Rules at a glance

See `RULES.md` for the full list with hints and fixes. The validator
also self-describes: `voce validate --list-codes` prints every code,
and each diagnostic carries a `hint` and a docs URL (S67).
