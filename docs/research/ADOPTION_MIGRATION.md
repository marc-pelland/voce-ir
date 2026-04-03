# Voce IR — Adoption, Migration & Ecosystem Strategy

**Date:** 2026-04-02
**Status:** Living document
**Purpose:** Define how people start using Voce IR, how existing work migrates in, how the schema evolves, and how the project sustains itself.

---

## 0. The Strategic Reframe

**Voce IR's competition is not React or Next.js. It's the entire concept of writing UI code.**

The positioning should be: "What if building a UI was as easy as describing it?" — not "What if there was a better way to write components?" This determines the target audience (everyone, not just developers), the marketing narrative (democratization, not optimization), and the product roadmap (conversational experience first, compiler internals second).

---

## 1. The Wedge Use Case

**Non-technical people creating production-quality web pages through conversation.**

The 10x improvement is not for experienced React developers. It's for:

1. The marketing manager who currently waits 2 weeks for a landing page
2. The founder who can describe their product but can't code the homepage
3. The designer who has a perfect Figma mockup but needs a developer to build it
4. The small business owner who needs a professional web presence

For these users, Voce IR offers: describe what you want → 5 minutes → deployed, accessible, performant page with security baked in.

**Secondary wedge: Email HTML.** The pain is enormous, the domain is constrained, and quality is so bad that automated output exceeds hand-coded quality for most teams.

---

## 2. The "First 5 Minutes" Experience

```
1. Install (30 sec):     npm install -g voce-ir
2. Describe (60 sec):    voce generate "A landing page for a coffee subscription 
                          service with hero, pricing tiers, testimonial carousel"
3. See result (30 sec):  Browser opens with polished, animated, accessible page
4. Iterate (120 sec):    voce patch "Full-bleed hero image, warm earth tones, 
                          sticky header"
5. Deploy (30 sec):      voce deploy → live URL

Total: 5 minutes from zero to deployed production page
```

This is the wow moment. No one can do this today — not with code, not with no-code tools, not with AI code generation.

---

## 3. Migration Paths

### Design Tokens → ThemeNode (90%+ automatable)

The fastest migration path. Teams with existing design tokens (Style Dictionary, Tokens Studio, W3C Design Tokens format) can import directly:

- Color tokens → ThemeNode color palette
- Typography tokens → font families, sizes, weights
- Spacing tokens → spacing scale
- Border radius, shadow tokens → direct mapping

**Prioritize this.** It provides instant value and lowers adoption friction.

### Figma → IR (70-80% automatable)

Figma's REST API exposes the full design tree. Structural conversion is straightforward:

- Frames → Container/Surface nodes
- Text → TextNode
- Auto Layout → flex properties
- Components → reusable IR fragments
- Styles → ThemeNode properties

**What's lost:** Interactivity, responsive behavior, accessibility, real content. These must be specified conversationally — which is actually a strong value prop: "Import your Figma design, then talk to AI to make it real."

### Screenshot → IR (Vision AI)

Claude/GPT-4V can identify UI elements with good accuracy and estimate layout. Useful as onboarding: "Show me a screenshot of what you want, and I'll build it." Frame as starting point, not finished product.

### HTML/CSS → IR (50-60% automatable)

Works well for static, semantic HTML. For modern SPAs, combine DOM analysis with AI interpretation. Useful for migrating existing simple sites.

### React → IR (40-50% automatable)

Extract structural skeleton via static analysis (Babel AST, TypeScript compiler API), then AI interprets behavior. TypeScript types provide surprisingly rich starting points.

---

## 4. Schema Evolution Strategy

### Policy: Additive-Only with Extensions

**FlatBuffers guarantees:**
- Fields can be added (old readers skip unknown fields)
- Fields can be deprecated (remain for ID stability, not generated in code)
- Fields cannot be removed, reordered, or have types changed

**Voce IR policy:**

1. **Core schema is additive-only.** Within a major version, all changes are additive. Semantic versioning at the schema level.

2. **Extension mechanism for custom node types.** Reserve `custom_data` field (byte vector) on key node types. Define an extension registry (namespace + type ID → schema). Community can extend without waiting for core changes.

3. **Version field in IR header.** Every blob carries: schema major version, schema minor version, extensions used. Validators/compilers check compatibility before processing.

4. **Migration tooling.** `voce migrate` reads old IR, outputs new IR conforming to latest schema. For additive changes: trivial (add defaults). For major versions: explicit migration logic. Major bumps should be rare (every 2-3 years).

---

## 5. Collaboration Model

### Phase 1: Canonical JSON in Git

The pragmatic choice. Convert IR to deterministic JSON for git storage. Text-diffable, mergeable, reviewable. Binary IR is a build artifact (like compiled output). The JSON isn't human-authored code — it's machine-readable text serialization.

**Key:** Deterministic output (sorted keys, consistent formatting) so diffs are meaningful.

### Phase 2: Semantic Diff

`voce diff` produces semantic diffs: "Added TextNode 'Welcome' to Container 'hero'" rather than raw text diffs. Makes code review meaningful.

**Lesson from Unity:** Encourage small, composable IR fragments rather than monolithic blobs. Node-addressable structure makes merging easier.

### Phase 3+: CRDT Collaboration (Premium)

Figma-style real-time multiplayer editing. Requires a collaboration server. CRDT-based (Automerge/Yjs). This is a premium hosted feature, not a core open-source requirement.

---

## 6. Ecosystem & Extension Architecture

### Extension Points

1. **Custom validator passes** — community rules for a11y, performance, brand compliance
2. **Custom compile targets** — React Native, Flutter, email HTML, PDF
3. **Import adapters** — Figma, Sketch, design tokens, HTML, Markdown → IR
4. **Style packs** — curated ThemeNode configurations + UI patterns
5. **AI prompt libraries** — tested prompts for common UI patterns
6. **Content adapters** — CMS-specific content fetch + rich text conversion

### Governance

BDFL (Benevolent Dictator for Life) initially — the only realistic model for a solo builder. Transition to committee/foundation only when there are 5+ active core contributors and corporate interest.

---

## 7. Business Model

### Open Source (Free Forever)
- Core IR specification and schema
- Reference validator
- DOM compiler
- CLI tools
- Figma import adapter
- Design token import

### Premium/Paid
1. **Hosted compilation + deployment** ("Vercel for Voce IR") — monthly subscription
2. **Style pack marketplace** — community + curated design packs, revenue share
3. **Team collaboration** — real-time multiplayer editing, version history, approval workflows
4. **Enterprise** — SSO, audit logs, brand compliance validators, SLA
5. **Specialized AI model** — fine-tuned IR generation model, API access

### Realistic Timeline
- Months 1-6: Open source, no revenue, focus on adoption
- Months 6-12: Simple hosted compilation service
- Months 12-18: Style pack marketplace
- Months 18-24: Enterprise features
- Alternative: Get hired by a company (Vercel, Anthropic) to work on it full-time

---

## 8. The Trust Gap

"I can't see the code — how do I know it's right?"

**Mitigations:**
1. **Inspector tool** — visual DevTools showing IR structure overlaid on rendered output
2. **Validation report** — every compilation produces a11y, security, performance audit
3. **HTML export** — escape hatch: export to standard HTML/CSS/JS at any time
4. **Transparency** — show what the AI generated in conversational interface
5. **Comparison mode** — render same IR through multiple views (DOM output, React equivalent, a11y tree)

---

## 9. Adoption Lessons from Precedents

| Tool | Time to Mainstream | Key Pattern | Voce IR Lesson |
| ---- | ------------------ | ----------- | -------------- |
| React | ~3 years | "Just the view layer" — incremental adoption | Voce has no incremental path — needs a stronger wow moment |
| TypeScript | ~6 years | Gradual typing — existing JS works | Design token import is the closest "existing work just works" path |
| Docker | ~2 years | Solved universal pain point with visceral demo | Target the "wait 2 weeks for a landing page" pain point |
| Svelte | ~4 years | Solo creator + strong narrative | Solo builder can succeed with great storytelling + community |

**Timeline reality:** Even fast paradigm shifts take 2+ years. Plan accordingly.

---

## 10. Must Exist Before Public Launch

1. Working end-to-end demo (natural language → IR → compiled output → browser)
2. DOM compiler producing valid, accessible, performant HTML
3. Validator catching structural errors and a11y violations
4. 5+ visually impressive example outputs
5. Inspector tool (even basic)
6. HTML export escape hatch
7. Documentation (not just API docs — the "why" narrative)
8. Try-it playground (zero install, browser-based)

---

*This document should be read alongside `DEEP_RESEARCH.md`, `SECURITY_TESTING_TOOLING.md`, and `DATA_INTEGRATION.md`.*
