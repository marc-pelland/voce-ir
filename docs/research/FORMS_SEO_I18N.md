# Voce IR — Forms, SEO & Internationalization

**Date:** 2026-04-02
**Status:** Living document
**Purpose:** Define how Voce IR handles three table-stakes web capabilities that any credible Next.js replacement must have.

---

## 0. Why These Matter

Forms, SEO, and i18n are where frameworks prove they're production-ready. A landing page demo doesn't need them. A real business does. Voce IR's structural advantages (compilation, validation, accessibility enforcement) give it genuine superiority in all three areas — but only if the IR models them correctly.

---

## 1. Forms

### 1.1 The Voce IR Advantage

Current frameworks require developers to manually wire validation (Zod), form libraries (React Hook Form), accessibility (ARIA attributes), and progressive enhancement (server actions). Voce IR's compiler generates all of this from a single declarative FormNode:

| Concern | Next.js + RHF + Zod | Voce IR |
| ------- | -------------------- | ------- |
| Validation | Developer writes Zod schema AND wires to RHF | Compiler generates from FormField validation rules |
| Accessibility | eslint warnings (ignorable) | Compile error if label/aria missing |
| Progressive enhancement | Opt-in via server actions | Default — every form works without JS |
| Performance | RHF avoids re-renders via refs | No VDOM — compiler emits surgical DOM updates |
| Client-server parity | Developer must maintain both | One source of truth in IR |

### 1.2 FormNode Schema

```
FormNode {
  id, name
  fields: [FormField]               // ordered field list
  field_groups: [FormFieldGroup]     // fieldset equivalents
  validation_mode: enum              // on_submit | on_blur | on_change | on_blur_then_change
  cross_validations: [CrossFieldValidation]
  submission: FormSubmission         // ActionNode + encoding + success/error handling
  initial_values: DataNode           // static or fetched
  autosave: AutosaveConfig           // optional draft persistence
  semantic: SemanticNode             // role: "form", required
}

FormField {
  id, name, field_type              // text | email | password | number | select | checkbox | radio | textarea | file | date | hidden
  label: TextNode (required)         // also used for a11y
  placeholder: LocalizedString
  validations: [ValidationRule]      // required | min_length | pattern | email | custom
  async_validations: [AsyncValidation]  // server-side checks (email uniqueness)
  visible: DataBinding               // conditional visibility
  autocomplete: AutocompleteHint     // browser autocomplete
  description: TextNode              // help text (aria-describedby)
  semantic: SemanticNode (required)
}

ValidationRule {
  rule_type: enum                    // required | min_length | max_length | pattern | min | max | email | url | custom
  value: Scalar
  message: LocalizedString (required)  // i18n-ready error message
}

FormSubmission {
  action: ActionNode (required)      // the mutation
  encoding: enum                     // url_encoded | multipart | json
  progressive: bool = true           // works without JS
  success_transition: StateTransition
  error_handling: FormErrorHandling  // field_errors map + form_errors list
}
```

### 1.3 Compiled Output

The compiler emits:
- Native `<form>` with `action` and `method` for progressive enhancement
- `<label>` + `<input>` pairs with `for`/`id` association
- `aria-required`, `aria-describedby`, `aria-invalid` attributes
- Error containers with `role="alert"` and `aria-live="polite"`
- Client-side validation JS (generated from ValidationRule declarations)
- Submit handler with loading state, error display, and success transition
- `autocomplete` attributes for browser autofill

### 1.4 Validator Enforcement

- Every FormField has a label TextNode
- Every FormField has a SemanticNode
- Every required field has `aria-required`
- FormNode has a submit trigger
- Validation messages are LocalizedStrings (i18n-ready)
- Password fields have correct `autocomplete` attributes
- File upload fields have type/size constraints

### 1.5 Multi-Step Forms

Modeled as a StateMachine where states are form pages:

```
StateMachine {
  states: [step_1, step_2, step_3, review, submitted]
  transitions: [
    { from: step_1, event: next, guard: step_1_valid, target: step_2 },
    { from: step_2, event: back, target: step_1 },
    { from: step_2, event: next, guard: step_2_valid, target: step_3 },
    ...
  ]
}
```

Each step's FormNode validates only its own fields. Progress tracking derives from state machine position.

---

## 2. SEO

### 2.1 The Voce IR Advantage

Voce IR has inherent SEO advantages that no framework-based approach can match:

- **Zero-JS static HTML** — no hydration delay, instant LCP
- **Pre-computed layout** — no CLS from late-loading content
- **Heading hierarchy enforced at compile time** — not a lint warning
- **Semantic HTML guaranteed** — SemanticNode roles compile to `<h1>`, `<nav>`, `<main>`, `<article>`
- **Output < 10KB** — fastest possible page load on any connection
- **Sitemap automatic** — RouteMap already contains all routes

### 2.2 Schema Additions

**PageMetadata on ViewRoot:**

```
PageMetadata {
  title: LocalizedString (required)
  title_template: string             // "%s | My Site"
  description: LocalizedString
  canonical_url: string
  robots: RobotsDirective            // index, follow, max-snippet, etc.
  open_graph: OpenGraphData          // og:title, og:description, og:image, og:type
  twitter_card: TwitterCardData
  alternates: [AlternateLink]        // hreflang for i18n
  custom_meta: [MetaTag]             // escape hatch
}
```

**StructuredData (JSON-LD):**

```
StructuredData {
  schema_type: string (required)     // "Article", "Product", "FAQ", "BreadcrumbList"
  properties: DataNode (required)    // typed JSON-LD content
}
```

**Sitemap on RouteMap:**

Each RouteEntry gets: `sitemap_priority`, `sitemap_change_freq`, `sitemap_last_modified`, `exclude_from_sitemap`. The compiler generates `sitemap.xml` and `robots.txt` automatically.

### 2.3 Validator Enforcement

- Every ViewRoot has PageMetadata with non-empty title
- Title length warning > 60 characters
- Description length warning > 160 characters
- Exactly one heading level 1 per ViewRoot
- Heading hierarchy is sequential (no h1 → h4 skips)
- At least one `<main>` landmark per ViewRoot
- Unique titles across routes
- OG image required when OG data is present
- Hreflang alternates form complete reciprocal sets
- StructuredData properties match Schema.org expectations
- Every visible MediaNode has explicit dimensions (CLS prevention)

### 2.4 Compiled Output

- Complete `<head>` with meta tags, OG, Twitter cards, canonical, hreflang
- JSON-LD `<script>` blocks for structured data
- Semantic HTML from SemanticNode roles
- `<img>` with explicit `width`/`height` (CLS prevention)
- Font `display: swap` strategy
- Preload hints for critical resources
- Static `sitemap.xml` and `robots.txt`
- `<html lang="xx">` from ViewRoot

---

## 3. Internationalization (i18n)

### 3.1 The Voce IR Advantage

| Concern | Next.js + next-intl | Voce IR |
| ------- | -------------------- | ------- |
| Runtime overhead | ~13KB i18n library | Zero (static mode) or ~1KB (runtime mode) |
| Missing translations | Silent — shows key string at runtime | Compile error |
| RTL support | Manual CSS logical properties | Automatic — compiler always emits logical properties |
| Hreflang | Manual configuration (error-prone) | Automatic from RouteMap |
| Per-locale optimization | Same bundle for all locales | Compiler tunes each locale independently |

### 3.2 Two Compilation Modes

**Static i18n (default, recommended):**
- Compiler resolves all LocalizedStrings against target locale's MessageCatalog
- Emits separate HTML per locale: `/en/index.html`, `/fr/index.html`, `/ar/index.html`
- Zero runtime i18n code — translations are baked in
- Like Paraglide's compiled i18n, but taken to its logical conclusion

**Runtime i18n (for apps needing locale switching):**
- Compiler emits message keys in HTML + minimal i18n runtime (~1KB)
- Active locale's catalog inlined, others loaded on demand
- Locale switching triggers re-resolution without page reload

### 3.3 Schema Additions

**LocalizedString (replaces raw strings in TextNode content):**

```
LocalizedString {
  message_key: string (required)     // "hero.title"
  default_value: string              // fallback if translation missing
  parameters: [MessageParameter]     // interpolation: {name}, {count}
  description: string                // context for translators
}

MessageParameter {
  name: string                       // "count"
  param_type: enum                   // string | number | date | currency | plural | select
  format_options: FormatOptions      // locale-specific formatting
}
```

**MessageCatalog (companion file, one per locale):**

```
MessageCatalog {
  locale: string (required)          // BCP 47: "en-US", "fr-FR", "ar-SA"
  messages: [Message]
  fallback_locale: string            // "en" as fallback for "en-US"
}

Message {
  key: string
  value: string                      // ICU MessageFormat
  // e.g., "{count, plural, one {# item} other {# items}}"
}
```

**RTL Support:**

- ViewRoot gains `direction: LayoutDirection` (ltr | rtl)
- Container uses logical layout properties (start/end, not left/right)
- Compiler emits CSS logical properties (`margin-inline-start`, `padding-inline-end`)
- MediaNode gains `mirror_in_rtl: bool` for directional icons
- PersonalizationSlot with locale condition switches direction per locale

### 3.4 Validator Enforcement

- Every LocalizedString key exists in every declared locale's catalog
- Pluralization rules match CLDR for target locales (Arabic: 6 forms, English: 2, Russian: 3, etc.)
- ICU MessageFormat syntax is valid in all message values
- RTL locales have `direction: rtl` set
- Warning if raw string used in TextNode when i18n is enabled
- Hreflang alternates are complete and reciprocal
- Font declarations include fallbacks for scripts used by declared locales

### 3.5 Translation Workflow

1. AI generates IR with LocalizedString references (message keys + default values in primary language)
2. MessageCatalogs are exported to XLIFF/JSON for professional translators
3. Translated catalogs are imported back
4. Validator checks completeness — missing translations are compile errors
5. Static compiler emits per-locale output

The JSON canonical representation of MessageCatalog serves as the translator exchange format.

---

## 4. Cross-Cutting: How These Interact

- **Forms + i18n:** Every validation message is a LocalizedString. Every label and placeholder is translatable. Form layout adjusts for RTL.
- **Forms + SEO:** Multi-step forms on separate routes each need PageMetadata. Success pages need canonical URLs.
- **SEO + i18n:** Hreflang tags, per-locale sitemaps, localized meta descriptions, localized structured data. Compiler handles all automatically from RouteMap + LocalizedString.

---

## 5. New IR Nodes Summary

| Node | Domain | Phase |
| ---- | ------ | ----- |
| FormNode | Forms | 1 (schema), 2 (compiler) |
| FormField | Forms | 1 (schema), 2 (compiler) |
| ValidationRule | Forms | 1 (schema), 2 (compiler) |
| FormSubmission | Forms | 1 (schema), 2 (compiler) |
| PageMetadata | SEO | 1 (schema), 2 (compiler) |
| OpenGraphData | SEO | 1 (schema), 2 (compiler) |
| StructuredData | SEO | 1 (schema), 2 (compiler) |
| LocalizedString | i18n | 1 (schema), 2 (compiler) |
| MessageCatalog | i18n | 1 (schema), 2 (compiler) |
| FormatOptions | i18n | 1 (schema), 2 (compiler) |

---

## 6. What Makes This a Credible Next.js Replacement

| Capability | Next.js | Voce IR |
| ---------- | ------- | ------- |
| Form validation | Zod + RHF (developer-wired) | Compiler-generated, zero wiring |
| Progressive enhancement | Server actions (opt-in) | Default — every form works without JS |
| Form accessibility | eslint warnings (ignorable) | Compile errors (enforced) |
| SEO meta tags | generateMetadata (per-route) | PageMetadata on ViewRoot, automatic |
| Heading hierarchy | No enforcement | Compile error if broken |
| Core Web Vitals | Good with discipline | Guaranteed by architecture |
| Static i18n | Not built-in (next-intl addon) | First-class, zero-runtime compilation |
| RTL | Manual CSS logical properties | Automatic — compiler emits them always |
| Missing translations | Silent runtime key display | Compile error |
| Hreflang | Manual, error-prone | Automatic from RouteMap |
| Structured data | Manual JSON-LD | Validated at compile time |

**The pattern is consistent:** Next.js requires developer discipline. Voce IR guarantees it through compilation.

---

*This document should be read alongside `DEEP_RESEARCH.md`, `DATA_INTEGRATION.md`, `SECURITY_TESTING_TOOLING.md`, and `ADOPTION_MIGRATION.md`.*
