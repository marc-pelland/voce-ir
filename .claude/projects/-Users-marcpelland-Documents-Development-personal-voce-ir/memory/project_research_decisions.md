---
name: Research-Informed Architecture Decisions
description: Key decisions confirmed by deep research in April 2026 — FlatBuffers, Taffy, SPIR-V analogy, JSON canonical format, a11y opt-outs
type: project
---

Deep research completed 2026-04-02 and documented in `docs/research/DEEP_RESEARCH.md`. Key decisions:

- **FlatBuffers confirmed** — zero-copy aligns with read-heavy UI workloads. Immutability means runtime state lives outside the buffer in a reactive layer.
- **JSON canonical representation** — lossless round-trip to/from binary. AI generates JSON; bridge encodes to binary. Also serves as debugging, diffing, and escape hatch format.
- **Taffy** (Rust) for compile-time layout resolution (flexbox/grid engine).
- **SPIR-V is the reference pipeline** — binary IR + validator + multi-target compilation.
- **Compose's slot table** is the closest existing analog to a binary UI IR.
- **SolidJS/Svelte compiled output** is the quality benchmark for DOM compiler output.
- **A11y opt-outs added** — `decorative: true`, `presentation: true`, `motion_functional: true` to prevent false positives while keeping default behavior strict.
- **No major competitor uses binary IR for AI-generated UI** — this is genuinely novel territory.
- **Performance targets validated** — <50ms TTI and <10KB achievable for landing pages, but recommend per-route budgets for complex apps.

**Why:** These decisions were based on comparative analysis of Flutter, SwiftUI, Compose, Svelte, SolidJS, Qwik, SPIR-V, game engines, and all major AI UI generation tools.

Additional decisions from second research round (2026-04-02):
- **Data trinity:** DataNode (queries), ActionNode (mutations), SubscriptionNode (real-time) — the GraphQL trinity generalized
- **ActionNode** is the server action equivalent — declares endpoint + input/output types + optimistic strategy + cache invalidation
- **DOM compiler emits TanStack Query** for cache management (~13KB) rather than building custom. Pages without data remain zero-dependency
- **ContentSlot** with cache_strategy (static/isr/dynamic) for CMS content. RichTextNode for structured rich text
- **FormNode** as a form coordinator — compiler generates validation JS, progressive enhancement, accessible markup
- **SEO as PageMetadata on ViewRoot** — compiler emits all meta tags, validates heading hierarchy, generates sitemap.xml
- **i18n: static mode (default)** — per-locale output compilation, zero runtime. Runtime mode available for locale switching
- **LocalizedString** replaces raw strings in TextNode content when i18n enabled. Missing translations = compile errors
- **Wedge use case:** Non-technical creators building production landing pages via conversation. Competition is Squarespace/Wix, not React
- **Supabase** as recommended Tier 1 BaaS. Convex for reactive patterns
- **Design token import** is the fastest migration path (90%+ automatable). Figma→IR at 70-80%

**How to apply:** Reference `docs/research/` directory when making architecture decisions. Five research docs cover: landscape analysis, security/testing/tooling, data integration, forms/SEO/i18n, and adoption/migration.
