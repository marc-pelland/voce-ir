# Sprint 16 — Forms & Data Layer Compilation

**Status:** Planned
**Goal:** Compile FormNode to accessible, progressively-enhanced HTML forms with client-side validation. Compile ActionNode to mutation functions with optimistic updates. Compile ContentSlot to build-time content injection. After this sprint, forms work without JS (progressive enhancement) and enhance with validation + loading states when JS is available.
**Depends on:** Sprint 15 (theming/SEO/i18n — forms may use localized labels and theme styles)

---

## Deliverables

1. FormNode → `<form>` with `method`/`action` for progressive enhancement
2. FormField → `<label>` + `<input>`/`<select>`/`<textarea>` with ARIA
3. ValidationRule → client-side JS validation (generated from rules)
4. FormSubmission → `fetch()` with loading/error/success states
5. ActionNode → mutation function with optimistic update + cache invalidation
6. ContentSlot (static) → content baked into HTML at compile time

---

## Tasks

### 1. Form HTML Emission (`lower/form.rs`)

FormNode → native `<form>` that works without JavaScript:
