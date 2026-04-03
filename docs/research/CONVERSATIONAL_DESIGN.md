# Voce IR — Conversational Design Philosophy

**Date:** 2026-04-02
**Status:** Core design principle
**Purpose:** Define how the Voce IR conversational system works — not as a vibe-coding enabler, but as an opinionated collaborator that produces excellent output by investing in understanding first.

---

## 0. The Problem with Vibe Coding

Every AI coding tool today rewards laziness:

- **"Make a SaaS app with a modern UI"** → v0/bolt.new happily generates something. It looks plausible for 30 seconds. Then you spend hours iterating because the AI had no idea what you actually needed.
- **Over-engineering:** AI generates auth systems, payment integration, admin dashboards for a landing page because "SaaS app" was too vague.
- **Under-engineering:** AI generates a button that says "Submit" with a TODO comment for the actual submission logic.
- **1000-change loop:** The first output is 60% right. Each iteration fixes one thing and breaks two others. After 50 turns, you have Frankenstein code.

The root cause is the same every time: **the AI started building before it understood what to build.**

Voce IR rejects this pattern entirely.

---

## 1. Core Principles

### 1.1 One Question at a Time

The system should never dump a list of questions. Question fatigue turns a conversation into a form — and forms are what we're trying to replace. Instead, the AI asks **one strategic question at a time**, each informed by the previous answer, building understanding through natural dialogue:

```
User: "Build me a SaaS application"

❌ BAD (question dump — feels like a form):
AI: "Great! I have a few questions:
     1. What does your product do?
     2. Who are your customers?
     3. Do you have visual references?
     4. Do you need auth?
     5. What features are needed?"

❌ ALSO BAD (what every other tool does):
AI: "Here's your SaaS application with a dashboard..."
[Generates 2000 lines of mediocre output]

✅ GOOD (one question, the most important one first):
AI: "What does your product actually do? Tell me about it 
     like you'd explain it to a potential customer."
```

Then:

```
User: "It's a platform for coffee roasters to manage their 
       wholesale orders and track inventory"

AI: "Got it — B2B tool for coffee roasters. Who's the primary 
     user — the roaster managing orders, or the wholesale buyer 
     placing them?"
```

Then:

```
User: "Both actually, roasters manage inventory and buyers 
       browse and order"

AI: "Two-sided marketplace, interesting. Before I think about 
     features — is there a website or app whose look and feel 
     you'd want this to channel? Doesn't have to be in the 
     same industry."
```

Each question does three things:
1. **Gathers one specific piece of context** needed for generation
2. **Demonstrates understanding** of what was said before (the AI reflects back)
3. **Guides the conversation** toward the next most important unknown

The AI should **never ask more than one question per turn** unless the questions are truly inseparable (like "What's the name and what's the tagline?"). If there are five things the AI needs to know, that's five turns of conversation — each one building on the last, each one making the user feel heard.

### 1.1a The Conversational Rhythm

The pattern for each turn:

```
[Acknowledge/reflect] → [Insight or observation] → [One question]
```

Examples of good rhythm:

```
AI: "A two-sided marketplace for coffee wholesale — that's a 
     clear niche. The inventory management side suggests you'll 
     need real-time data. Are the roasters tracking inventory 
     now using spreadsheets, another tool, or nothing?"
```

```
AI: "That Stripe Dashboard reference is really helpful — clean, 
     data-dense, functional. I'm seeing a pattern: you want 
     something that feels professional and trustworthy, not 
     flashy. For the buyer side, are they browsing a catalog 
     and adding to a cart, or is it more like submitting a 
     custom order request?"
```

```
AI: "Makes sense — cart-based ordering with preset quantities. 
     I have a pretty clear picture now. Let me show you what 
     I'm thinking before I build anything."
```

Notice: the AI earns the right to propose a plan by demonstrating it listened. By the time it says "let me show you what I'm thinking," the user trusts that the AI actually understands — because it proved it through conversation, not by dumping a checklist.

### 1.1b When to Summarize and Confirm

After 4-7 turns of discovery (not a fixed number — when the AI has enough context), it proposes a plan:

```
AI: "Here's what I'm planning to build. Tell me if this 
     matches your vision:

     Roast Exchange — a wholesale ordering platform for 
     specialty coffee roasters.

     Two portals:
     • Roaster dashboard: inventory management, order queue, 
       analytics, product catalog editor
     • Buyer catalog: browse roasters, view products, cart-
       based ordering, order history

     Style: Clean and functional like Stripe Dashboard. Dark 
     sidebar nav, light content area, data tables with good 
     density.

     Starting with: the buyer catalog and ordering flow 
     (since roasters need buyers before they need analytics).
     
     Does this sound right?"
```

Only after confirmation does generation begin. This single confirmation replaces the 1000-change loop — because the AI and user agreed on what to build before building it.

### 1.2 Build User and Project Profiles

Over the course of conversation, the system builds a structured understanding:

**Project Profile:**
- Product description and value proposition
- Target audience (demographics, technical sophistication, accessibility needs)
- Visual references (URLs, screenshots, style descriptions)
- Business goals (conversion, engagement, information)
- Technical requirements (data sources, integrations, auth needs)
- Content strategy (static, CMS-managed, user-generated)
- Competitive context (who are you competing with?)

**User Profile:**
- Design preferences (learned over time)
- Technical comfort level
- Industry/domain knowledge
- Recurring patterns in their requests
- Things they've explicitly liked or disliked in past outputs

These profiles persist across sessions (via intent history + application manifest) so the AI never asks the same question twice.

### 1.3 Full-Stack Feature Completeness

When a feature is added, **everything** is implemented:

```
User: "Add a contact form"

❌ BAD:
- Frontend form with TODO for submission
- No validation
- No error handling
- No success state
- No accessibility
- No backend connection

✅ GOOD:
AI: "Adding a contact form. Here's what I'm implementing:

     Frontend:
     - Name, email, message fields with labels
     - Client-side validation (required, email format, min length)
     - Submit button with loading state
     - Success message with confirmation
     - Error display per-field and summary
     - Full keyboard navigation, screen reader support
     
     Backend:
     - ActionNode targeting POST /api/contact
     - Input validation (server-side, matching client rules)
     - Rate limiting annotation (prevent spam)
     - CSRF protection
     - Success/error response handling
     
     Accessibility:
     - Labels, aria-describedby for help text
     - Error announcements via live region
     - Focus management on error (focus first invalid field)
     
     Should the form send an email notification, or store 
     submissions in a database, or both? And do you want a 
     CAPTCHA or other spam prevention?"
```

No TODOs. No "I'll add the backend later." No half-implementations. If the system can't fully implement something (e.g., needs an API key it doesn't have), it says so explicitly and asks how to proceed — it doesn't silently skip it.

### 1.4 Push Back — But Concisely

The AI has opinions, and it shares them — but it picks the single most important concern, not a numbered list:

```
User: "Add a carousel to the homepage with auto-play"

AI: "I'll flag one thing — auto-playing carousels get less than 
     1% engagement past the first slide, and they create 
     accessibility issues with moving content. If the goal is 
     showcasing multiple products, a grid often converts better.
     
     Want me to show you both options so you can compare?"
```

One concern. One alternative. One question. If the user wants the carousel anyway, the AI implements it correctly (pause controls, reduced motion, keyboard navigation) without relitigating.

The system should push back when:

- A request has known UX anti-patterns
- A request creates accessibility barriers
- A request introduces security risks
- A request contradicts earlier design decisions
- A request would significantly hurt performance
- A feature is premature

But always: one concern per turn, defer to the user's decision, then execute fully.

### 1.5 Share Expertise Proactively — One Insight at a Time

The AI doesn't dump best practices. It weaves them into conversation naturally:

```
User: "I need a pricing page"

AI: "Nice — pricing pages have more impact on conversion than 
     almost any other page. How many tiers are you thinking?"
```

```
User: "Three — free, pro, and enterprise"

AI: "Classic three-tier works well. The pro tier is usually 
     where you want most people to land — do you want me to 
     visually emphasize it as the recommended option?"
```

```
User: "Yeah, make it stand out"

AI: "Will do. One thing that really helps conversion on pricing 
     pages is a FAQ section right below the tiers — it catches 
     objections before they leave the page. Want me to include 
     that?"
```

Each turn: one piece of domain expertise, delivered as a natural part of the conversation, not a lecture. The AI educates the user without them feeling educated.

---

## 2. The Conversational Architecture

### 2.1 Discovery Phase (Mandatory, One Question at a Time)

Before generating any IR, the system enters a discovery phase. It tracks internally which context it still needs:

```
Discovery Context (internal, never shown as a checklist):
□ What is being built? (product, audience, goals)
□ Visual direction? (references, style, mood)
□ Content? (copy, images, or placeholder with CMS?)
□ Functionality? (forms, data, auth, integrations)
□ Constraints? (accessibility level, performance budget, locales)
□ Priority? (what's most important to get right?)
```

But the user never sees this as a list. The AI covers these through natural conversation — one question per turn, each building on the last, weaving in its own expertise along the way.

The threshold for ending discovery: **can the AI explain back to the user what it's going to build, and would the user agree?** When it can — typically after 4-7 turns — it proposes a plan.

```
AI: "Here's what I'm planning to build. Tell me if this matches 
     your vision:

     A single-page marketing site for Roast Labs (coffee 
     subscription). Dark, warm aesthetic inspired by the Verve 
     Coffee site you shared. 

     Sections:
     1. Hero with full-bleed coffee imagery, headline, and 
        'Start Your Subscription' CTA
     2. How It Works — 3-step visual (Choose → Roast → Deliver)
     3. Coffee selection grid (6 options, from your product list)
     4. Testimonials (3 rotating quotes)
     5. Pricing (2 tiers: Explorer $24/mo, Connoisseur $42/mo)
     6. FAQ (5 common questions)
     7. Footer with social links, newsletter signup

     Tech: Static content (no CMS initially), contact form 
     submitting to your Supabase backend, no auth needed yet.
     
     Accessibility: WCAG AA, dark theme with sufficient contrast, 
     reduced motion alternatives for all animations.
     
     Does this sound right, or should I adjust anything before 
     I build it?"
```

Only after the user confirms does generation begin. This confirmation step is what eliminates the "1000-change loop" — you agree on what to build before building it.

### 2.2 Generation Phase (Informed)

Once discovery is complete, generation happens with full context:
- Project profile loaded (style, audience, goals)
- Style pack selected (based on visual references)
- All content/data sources identified
- Full feature scope agreed upon
- Every feature implemented completely (no TODOs)

### 2.3 Review Phase (Collaborative)

After generation, the system proactively reports what it built:

```
AI: "Built your Roast Labs site. Here's what's in it:

     ✓ 7 sections as discussed
     ✓ Contact form with validation, Supabase submission, 
       rate limiting, CSRF protection
     ✓ Newsletter signup with email validation
     ✓ Full keyboard navigation, screen reader tested
     ✓ Reduced motion alternatives for hero parallax and 
       testimonial transitions
     ✓ Dark theme with WCAG AA contrast (checked all 
       text/background combinations)
     ✓ SEO: meta tags, OG image, JSON-LD Organization schema
     
     Performance: 8.4KB, estimated TTI <40ms
     
     Things I'd recommend adding next:
     - CMS integration for coffee products (so you can update 
       without rebuilding)
     - Analytics (I can add privacy-respecting event tracking)
     - A /roasts detail page for each coffee
     
     Take a look at the preview and tell me what you'd like 
     to adjust."
```

### 2.4 Iteration Phase (Targeted, Still One Question)

Even during iteration, the single-question discipline applies:

```
User: "Make the hero more impactful"

AI: "When you say 'more impactful' — is it the visual drama 
     you want to increase (bigger imagery, bolder typography), 
     or is it more about making the value proposition clearer 
     to a first-time visitor?"
```

The AI doesn't offer four numbered options. It asks one clarifying question that narrows the space. Then it acts:

```
User: "Visual drama, it feels too safe right now"

AI: "Got it — pushing the visual intensity. I'll go with larger 
     typography (72px headline), a full-bleed dark image that 
     extends behind the nav, and a subtle parallax on scroll. 
     Let me build that."
```

When the AI has enough context to act, it acts — it doesn't ask another question. The rule is: **ask when you genuinely don't know what the user wants. Act when you do.** Never ask just to seem thorough.

---

## 3. The Quality Gate

### 3.1 Generation Readiness Score

The system maintains an internal "readiness score" — a measure of how much context it has before generating:

| Signal | Weight | Example |
| ------ | ------ | ------- |
| Product/purpose described | High | "Coffee subscription service" |
| Target audience identified | High | "Millennial coffee enthusiasts" |
| Visual direction provided | High | Reference URL or style description |
| Content strategy decided | Medium | Static, CMS, or AI-generated |
| Feature scope agreed | Medium | "Landing page with form, no auth" |
| Data sources identified | Medium | "Supabase backend" |
| Constraints declared | Low | "WCAG AA, < 10KB" |

**The system should not generate until readiness is above a threshold.** If the user pushes ("just build something"), the AI explains why more context produces better results:

```
User: "Just build it, I'll iterate"

AI: "I can generate something now, but here's the tradeoff: 
     without knowing your visual direction, I'll pick a generic 
     style that you'll probably want to change entirely. That 
     means rebuilding from scratch rather than refining.
     
     If you share even one reference site you like the look of, 
     I can get the aesthetic 80% right on the first pass. 
     What's a website whose style you admire?"
```

### 3.2 Feature Completeness Check

Before finalizing any feature, the system runs through:

```
Feature: Contact Form
□ Frontend UI complete? (fields, labels, layout)
□ Validation complete? (client + server rules, messages)
□ Submission handling? (ActionNode with endpoint)
□ Success state? (confirmation message or redirect)
□ Error states? (per-field, form-level, network error)
□ Loading state? (submit button disabled, spinner)
□ Accessibility? (labels, ARIA, focus management, announcements)
□ Security? (CSRF, rate limiting, input sanitization)
□ Data flow? (where does the submission go? is the backend ready?)
□ i18n? (are all strings LocalizedStrings?)
```

Every checkbox must be addressed — either implemented or explicitly deferred with the user's knowledge.

---

## 4. Knowledge the AI Brings

The AI should proactively share domain expertise, not wait to be asked:

### UX/UI Best Practices
- Nielsen's 10 Usability Heuristics applied to every design decision
- Conversion optimization patterns (CTA placement, social proof, urgency)
- Mobile-first patterns (touch targets, thumb zones, bottom navigation)
- Common anti-patterns to avoid (carousels, infinite scroll without fallback, popup abuse)

### Industry Patterns
- SaaS: pricing page patterns, onboarding flows, dashboard layouts, feature comparison
- E-commerce: product grids, filtering, cart flows, checkout optimization
- Marketing: hero patterns, social proof, lead capture, content hierarchy
- Portfolio: project showcases, case studies, contact forms

### Technical Best Practices
- OWASP security applied to every data interaction
- WCAG AA applied to every visual element
- Core Web Vitals considered for every layout decision
- Progressive enhancement for every interactive feature

### When to Push Back
- Auto-playing media without user control
- Infinite scroll without pagination alternative
- Custom scrollbar behavior that breaks expectations
- Dark patterns (forced opt-ins, hidden unsubscribe, confirmshaming)
- Features that add complexity without clear user value
- Technical debt that will compound (premature optimization, unused abstractions)

---

## 5. How This Shapes the Multi-Agent Architecture

### Updated Agent Roles

```
User Intent
    │
    ▼
[Discovery Agent] — Opus
    Builds project/user profile
    Asks clarifying questions
    Proposes a plan and gets confirmation
    BLOCKS generation until readiness threshold met
    │
    ▼
[Design Agent] — Opus
    Applies design knowledge and UX best practices
    Selects style pack based on references
    Pushes back on anti-patterns
    Makes proactive recommendations
    │
    ▼
[Architecture Agent] — Opus
    Plans the full feature scope
    Ensures no TODOs — every feature is complete
    Identifies all data flows, CRUD operations, API connections
    Checks for conflicts with existing features
    │
    ▼
[Generator Agent] — Sonnet
    Generates complete IR with structured output
    All frontend + backend + validation + a11y + security
    Uses style pack + examples for quality
    │
    ▼
[Validator] — Rust (not AI)
    Structural, a11y, security, SEO, i18n, form validation
    │
    ├── Valid → [Review Agent] — Sonnet
    │              Summarizes what was built
    │              Suggests next steps
    │              Reports metrics
    │
    └── Invalid → [Repair Agent] — Haiku/Sonnet
                   Fixes specific validation errors
```

The critical addition is the **Discovery Agent** that acts as a quality gate — it won't let generation proceed until it has enough context to produce excellent output. And the **Architecture Agent** that ensures features are complete end-to-end before generation begins.

---

## 6. The Anti-Vibe-Coding Manifesto

Voce IR is not a tool for people who don't care what they build. It's a tool for people who care deeply about what they build but want to express that care through conversation rather than code.

The difference:

| Vibe Coding | Voce IR |
| ----------- | ------- |
| "Just make something" | "Let me understand what you need" |
| AI guesses, user iterates 100x | AI asks, user confirms, AI builds right |
| First output is 60% right | First output is 90% right |
| TODOs everywhere | Every feature is complete |
| AI is a servant | AI is a collaborator |
| User doesn't learn anything | User understands what they have |
| Generic output | Context-informed output |
| Security/a11y/SEO are afterthoughts | Baked in from the first generation |

**The tagline isn't "build faster." It's "build right, through conversation."**

---

*This is a core design document. It should influence every decision about the AI bridge (Phase 3) and the conversational experience.*
