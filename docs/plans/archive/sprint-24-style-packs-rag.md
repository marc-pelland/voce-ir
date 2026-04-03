# Sprint 24 — Style Packs & RAG

**Status:** Planned
**Goal:** Create the first 3 style packs (minimal-saas, editorial, ecommerce) containing design tokens, layout patterns, and example IR. Build RAG retrieval that matches user intent to the closest examples for few-shot prompting. After this sprint, generated output has distinct, professional visual identity instead of generic defaults.
**Depends on:** Sprint 23 (conversational design, brief creation)

---

## Deliverables

1. Style pack format: `packs/<name>/` with `tokens.yaml`, `patterns.yaml`, `examples/` directory
2. `minimal-saas` pack: clean SaaS aesthetic (Inter, blue/gray, generous whitespace)
3. `editorial` pack: content-focused (serif headings, warm tones, readable line lengths)
4. `ecommerce` pack: conversion-optimized (bold CTAs, product grid patterns, trust signals)
5. RAG index: embeds example IR descriptions, retrieves top-k matches for a given brief
6. Few-shot prompt builder: injects retrieved examples into Generator Agent context
7. Style pack selection integrated into conversational flow (S23)

---

## Tasks

### 1. Pack Format Definition (`src/packs/types.ts`)

Define the style pack schema: `tokens.yaml` (colors, typography, spacing, radii, shadows as design tokens), `patterns.yaml` (named layout patterns like "hero-centered", "features-grid-3col", "pricing-table" with IR structure templates), `examples/` (complete IR JSON files with descriptions). Each pack self-describes: name, description, best-for tags.

### 2. Minimal SaaS Pack (`packs/minimal-saas/`)

Tokens: Inter/system font stack, blue-600 primary, gray-50/900 neutrals, 4px base spacing, 8px radius. Patterns: centered hero, 3-column feature grid, single pricing toggle, minimal footer. 3 complete example IR files: landing page, pricing page, feature detail page.

### 3. Editorial Pack (`packs/editorial/`)

Tokens: Playfair Display headings, Source Serif body, warm gray palette, amber accents, 8px base spacing, 0px radius (sharp edges). Patterns: large-type hero, long-form content layout, pull quotes, author bio card, related articles grid. 3 examples: blog post, magazine homepage, article listing.

### 4. Ecommerce Pack (`packs/ecommerce/`)

Tokens: DM Sans, green-600 primary (trust), high-contrast CTAs, tight 4px spacing, 12px radius. Patterns: product hero with image gallery, product grid, add-to-cart bar, trust badges row, review stars. 3 examples: product page, category listing, promotional landing.

### 5. RAG Index & Retrieval (`src/rag/index.ts`)

Build an in-memory vector index of all example IR files. Each entry: file path, description embedding (via Claude), tags, style pack name. At query time: embed the DiscoveryBrief, find top-3 nearest examples by cosine similarity. Fall back to tag matching if embeddings unavailable (offline mode). Cache embeddings in `.voce/cache/embeddings.json`.

### 6. Few-Shot Prompt Builder (`src/rag/few-shot.ts`)

Take top-k retrieved examples and inject them into the Generator Agent's prompt as few-shot context. Format: "Here is an example of a similar page:" + truncated IR JSON (keep under 2K tokens per example). Measure generation quality improvement vs zero-shot baseline.

### 7. Pack Selection in Conversation (`src/conversation/style-selection.ts`)

Add a style topic to the conversation flow: show available packs with one-line descriptions, let user pick or say "surprise me" (auto-select based on brief). Selected pack's tokens feed into Design Agent, examples feed into RAG retrieval.

---

## Files to Create

- `packages/ai-bridge/src/packs/types.ts`
- `packages/ai-bridge/src/packs/loader.ts`
- `packages/ai-bridge/packs/minimal-saas/tokens.yaml`
- `packages/ai-bridge/packs/minimal-saas/patterns.yaml`
- `packages/ai-bridge/packs/minimal-saas/examples/` (3 IR files)
- `packages/ai-bridge/packs/editorial/tokens.yaml`
- `packages/ai-bridge/packs/editorial/patterns.yaml`
- `packages/ai-bridge/packs/editorial/examples/` (3 IR files)
- `packages/ai-bridge/packs/ecommerce/tokens.yaml`
- `packages/ai-bridge/packs/ecommerce/patterns.yaml`
- `packages/ai-bridge/packs/ecommerce/examples/` (3 IR files)
- `packages/ai-bridge/src/rag/index.ts`
- `packages/ai-bridge/src/rag/few-shot.ts`
- `packages/ai-bridge/src/conversation/style-selection.ts`
- `tests/ai-bridge/rag/retrieval.test.ts`
- `tests/ai-bridge/packs/loader.test.ts`

---

## Acceptance Criteria

- [ ] All 3 style packs load successfully and pass schema validation
- [ ] Each pack contains at least 3 complete, valid example IR files
- [ ] Design tokens from selected pack propagate into ThemeNode in generated IR
- [ ] RAG retrieves relevant examples (SaaS brief matches SaaS examples, not editorial)
- [ ] Few-shot prompting measurably improves output quality vs zero-shot (manual eval on 5 prompts)
- [ ] Style selection works in conversational flow ("pick a style" question appears)
- [ ] `--style minimal-saas` flag on `voce generate` bypasses interactive selection
- [ ] Generated output with a style pack has visually distinct identity (not generic)
- [ ] Embedding cache persists in `.voce/cache/` and speeds up subsequent queries
- [ ] Offline mode (no API for embeddings) falls back to tag-based matching
