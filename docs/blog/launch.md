# We Built a Website Without Writing Code — And Without a Framework

**The code is gone. The experience remains.**

Every AI coding tool today uses AI to write code faster in systems designed for humans. v0 generates React. bolt.new scaffolds Next.js. Cursor autocompletes TypeScript. They're using a car engine to pull a horse carriage.

Voce IR asks a different question: **if AI is the only author and the end-user experience is the only output that matters, what does the optimal system look like?**

The answer: a binary intermediate representation — SPIR-V for UI — where AI generates typed IR from natural language, a validator enforces quality rules, and a compiler emits optimized output. No human-readable code anywhere in the pipeline.

## What We Built

Voce IR is three things:

1. **A schema** — 12 FlatBuffers files defining 27 node types across 11 domains (layout, state, motion, navigation, accessibility, theming, data, forms, SEO, i18n)

2. **A compiler** — takes validated IR and emits production HTML. Zero framework runtime. 6.6KB for a full landing page with layout, state machine, spring physics animation, form validation, SEO metadata, security headers, and ARIA accessibility.

3. **An AI bridge** — multi-agent architecture that conducts a thoughtful design conversation (not vibe coding), generates IR, validates it, repairs errors, and compiles to HTML. One question at a time. Brief enforcement. Decision tracking. Memory across sessions.

## Why This Matters

### The Framework Tax

A typical Next.js landing page ships 200KB+ of framework runtime, takes 200-400ms to become interactive, and requires hydration, virtual DOM reconciliation, and a synthetic event system — all for content that could be a static HTML file.

Voce IR's compiler output for the same page: **6.6KB, <50ms TTI, zero framework runtime.** Every style is pre-computed. Every event is directly bound. Every animation uses CSS (with compile-time spring ODE solving — zero JS for spring physics).

### Accessibility Is a Compile Error

In React, accessibility is a lint warning. Developers disable it. In Voce IR, a missing SemanticNode is a validation error that blocks compilation. Heading hierarchy is enforced. Keyboard equivalents are required on every GestureHandler. ReducedMotion alternatives are mandatory on every animation. CSRF is required on every mutation.

The result: compiled output that passes axe-core with 0 violations — guaranteed by the system, not by developer discipline.

### The Anti-Vibe-Coding Manifesto

Current AI tools reward laziness. "Make me a SaaS app" produces something generic in 3 seconds that takes 50 iterations to fix. Voce IR's conversational system does the opposite:

- **One question at a time.** Not a checklist — a real conversation.
- **Quality gate.** Generation is blocked until the system understands what to build.
- **Brief enforcement.** Every request is checked against the project's north star.
- **Decision tracking.** Every choice is logged with rationale and conflict detection.
- **Full feature completeness.** When you add a form, you get: HTML, validation, ARIA, progressive enhancement, and CSRF. No TODOs.

The tagline isn't "build faster." It's **"build right, through conversation."**

## How It Works

```
You: "Build a landing page for a coffee subscription service"
AI:  "What's the product? Tell me about it."
You: "Premium single-origin beans, monthly delivery"
AI:  "Who's the audience?"
You: "Millennial coffee enthusiasts"
AI:  "What sections should the page have?"
You: "Hero, how it works, selection grid, testimonials, pricing"
AI:  "What visual style?"
You: "Dark, warm, like Blue Bottle's site"
AI:  "Here's the plan: [structured summary]. Does this sound right?"
You: "Yes"
AI:  "Building... ✓ 42 nodes, 6.8KB, 0 errors."
```

Six turns. One page. Full accessibility, security, SEO, and responsive design — because the system enforces it, not the developer.

## Try It

```bash
cargo install voce-ir
voce generate "a hero section with headline, subtitle, and CTA button"
voce compile output.voce.json -o index.html
open index.html
```

Or use the MCP server in Claude Code:
```
claude mcp add voce
```

## What's Next

- **Phase 4:** WebGPU + WASM compile targets for 3D scenes and compute-heavy state
- **Phase 5:** Visual Inspector — debug without code, edit by clicking
- **Phase 6:** Native targets (SwiftUI, Compose), plugin ecosystem

The code is gone. The experience remains. And it's just getting started.

---

*Voce IR is open source under Apache 2.0. A [Fire Burns Up](https://fireburnup.com) project.*
*Named from "sotto voce" — quiet input, extraordinary output.*
