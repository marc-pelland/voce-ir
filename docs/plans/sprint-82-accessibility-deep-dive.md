# Sprint 82 — Accessibility Deep Dive

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Take Accessibility — one of the three non-negotiable pillars — from "5 validator rules + a focus ring" to compile-time-verifiable WCAG 2.2 AA conformance with documented evidence per fixture. Add color contrast computation, focus order verification, automated screen-reader text generation, and a Lighthouse-style automated audit baked into CI.

**Depends on:** validator (S07), compiler-dom (S20), S64 (rich defaults), S67 (validator hints + fixes). Independent of S65/S66/S68/S69/S70/S71/S72/S74.

---

## Motivation

Today's a11y posture:
- 5 validator rules (A11Y001–A11Y005)
- SemanticNode references emit ARIA attrs in the compiler
- `:focus-visible` rings emitted in baseline CSS
- `prefers-reduced-motion` partially honored

What's missing:
- **Color contrast** isn't computed. Any IR can specify `color: rgb(180,180,180)` on `background: rgb(240,240,240)` and ship. Validator doesn't notice.
- **Focus order** isn't verified. Tab order follows DOM order today; the compiler doesn't audit whether that's the *intended* order.
- **Screen-reader text** for icon-only buttons isn't generated. An IR with a Surface containing only a MediaNode (an icon) and an `href` ships with no accessible name.
- **Heading hierarchy** is partially checked (H1 count) but skipped levels (H1 → H3) aren't flagged as warnings.
- **Touch target size** isn't checked at compile time (S64 emits 44px on form buttons but other interactive elements aren't audited).
- **Live regions** for dynamic content (DataNode updates, form errors) aren't required by the validator.

This sprint closes those gaps.

---

## Deliverables

### 1. Color contrast pass (validator)

New rule `A11Y006` — every text-on-background combination computed from the IR (TextNode color + nearest ancestor's fill) must meet WCAG 2.2 contrast:
- AA: 4.5:1 for body text, 3:1 for large text (≥18pt or ≥14pt bold)
- AAA: 7:1 / 4.5:1 (configurable via `.voce/validator.toml`, AA default)

Implementation: walk the IR computing each text element's effective foreground/background. Use the WCAG contrast formula from `apca-w3` or hand-roll. Emit `A11Y006` with the actual ratio in the diagnostic message:

```
A11Y006: Text contrast ratio 3.2:1 fails AA (requires 4.5:1)
  at /root/children/2/value/children/0
  fix: Increase text color brightness, or change background to a darker shade
```

### 2. Focus order verification

New rule `A11Y007` — the IR's tab order (derived from DOM order + explicit `tab_index` on SemanticNode) is audited:
- Negative `tab_index` values are valid (programmatic focus only) — no warning
- Positive `tab_index` values trigger `A11Y007` warning with hint: "Positive tab_index disrupts natural focus order. Use 0 (default) and rely on DOM order, or restructure the IR."
- A future enhancement (out of scope here): explicit focus-order assertions in the IR

### 3. Auto-generated accessible names

New compiler pass: for every interactive element (Surface with `href`, GestureHandler, etc.) without an explicit aria-label or text content, the compiler generates one based on visible content (alt text of contained MediaNode, sibling TextNode content, etc.). If no name can be derived, emit `A11Y008` error with hint: "Interactive element has no accessible name. Add a SemanticNode with `label`, an aria-labelledby ref, or visible text."

### 4. Heading hierarchy strictness

Existing rule expanded:
- `A11Y009` — heading level skips (H1 → H3) emit a warning unless explicitly justified via a `skip_level: true` field on the TextNode (escape hatch for deliberate cases)

### 5. Touch target verification

New rule `A11Y010` — every interactive element's effective touch target (computed from inline padding + content height) must meet 24×24 CSS px (WCAG 2.2 AA target size minimum) — escalate to 44×44 for AAA. The S64 form button defaults satisfy this; this rule catches other interactive elements that don't.

### 6. Live region requirement for dynamic content

Existing schema has `LiveRegion`. Today the validator doesn't require it for dynamic content. Add `A11Y011` warning: any IR containing a DataNode with `cache: Dynamic` or a FormNode with validation should have at least one LiveRegion ancestor (so screen readers announce changes).

### 7. Automated audit per fixture

For each cross-target fixture (S68): run an automated a11y audit using `axe-core` (via Puppeteer headless). Report:
- Issues found (categorized: critical, serious, moderate, minor)
- Specific WCAG criteria violated
- Per-fixture audit log archived as a CI artifact

The audit results feed back into the validator rule set — anything axe finds that the validator doesn't catch is a candidate for a new rule.

### 8. Per-fixture conformance evidence

For each fixture (S68 set), generate a one-page `a11y-evidence.md` listing:
- WCAG criteria the fixture satisfies (with reference: SC 1.1.1, etc.)
- Validator rules that passed
- Manual review items (if any)
- Tested screen readers (NVDA, VoiceOver) — manual; tracker only

These docs become the project's "accessibility receipts."

### 9. CI gate

A new CI step runs the full a11y audit on the cross-target fixtures + the production landing IR. Fails on any new critical or serious issue. Existing issues are baselined in `docs/a11y/baseline.json` and require explicit PR action to update.

### 10. Documentation

`docs/accessibility/` directory containing:
- `OVERVIEW.md` — Voce's a11y posture in plain language
- `RULES.md` — every A11Y* validator rule with examples
- `WCAG_MAPPING.md` — table of WCAG 2.2 success criteria → which Voce rule covers it (if any)
- `MANUAL_TESTING.md` — what *can't* be automated; how to test with NVDA, VoiceOver, JAWS

---

## Acceptance Criteria

- [ ] 6 new validator rules (A11Y006–A11Y011) implemented with hints + fixes per S67 conventions
- [ ] Color contrast computation runs on all fixtures
- [ ] Auto-generated accessible names for icon-only buttons
- [ ] axe-core audit integrated; runs on cross-target fixtures + production landing
- [ ] Per-fixture `a11y-evidence.md` generated
- [ ] WCAG 2.2 mapping document complete
- [ ] CI gate active (fails on new critical/serious issues)
- [ ] Manual-testing guide written
- [ ] Baseline JSON committed

---

## Risks

1. **WCAG conformance is a moving target.** WCAG 2.2 ships now; 3.0 changes the model substantially. Pin to 2.2 and revisit when 3.0 is stable.
2. **Color contrast in nested IRs is computationally tricky.** Background can come from multiple ancestor surfaces, themes, or CSS variables. Initial implementation: compute from the nearest ancestor with an explicit fill; document edge cases as known gaps.
3. **axe-core can produce false positives.** Curate the rule set; allow per-fixture suppression with a documented rationale.
4. **Manual screen-reader testing isn't automated.** That's by design — but the test plan should be clear about what humans need to do at release time.

---

## Out of Scope

- Tested-with-screen-reader certification for every fixture (lab-shop work)
- Cognitive accessibility (WCAG 2.2 SCs around clear language, focus visibility) — partial coverage; full sweep is a follow-up
- Sign language interpretation, captioning, etc. (media features outside the IR's current scope)
- Accessibility for the AI bridge / cli-chat itself (CLI tools have different a11y considerations)
