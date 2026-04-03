# Voce IR — Security, Testing, Documentation, Tooling & AI Strategy

**Date:** 2026-04-02
**Status:** Living document
**Purpose:** Define how Voce IR handles security, testing, application documentation, developer tooling, and AI model strategy — the capabilities that turn a compiler into a production-ready platform.

---

## 0. The Guiding Principle

In traditional development, security, testing, and documentation are things developers *should* do but often *don't*. They're afterthoughts bolted on at the end. Voce IR has a unique opportunity: **because the system controls the entire pipeline from intent to output, it can make these concerns automatic and structural rather than opt-in.**

Just as accessibility is a compile error in Voce IR, security should be a compile error. Testing should be generated from the IR. Documentation should be derived from the intent history. The user never has to think about these — they happen because the system was designed for them.

---

## 1. Security Framework

### 1.1 The Voce Security Advantage

Traditional web security requires developers to manually avoid OWASP vulnerabilities in code they write. AI-generated code is *worse* at this — studies show LLMs frequently generate code with XSS, injection, and insecure defaults.

Voce IR eliminates most of these attack surfaces by design:

| OWASP Top 10 (2021) | Traditional Risk | Voce IR Status |
| -------------------- | ---------------- | -------------- |
| A01: Broken Access Control | Developers forget auth checks | **Enforceable.** RouteMap guards are validated — every guarded route must declare auth state machine. Compiler rejects unguarded routes that access protected DataNodes |
| A02: Cryptographic Failures | Devs use weak crypto, plaintext storage | **Partially enforceable.** DataNode transport declarations can require TLS. No user code means no accidental plaintext logging. Compiler emits secure defaults for cookies/storage |
| A03: Injection (XSS, SQL) | Unsanitized user input in DOM/queries | **Structurally eliminated for XSS.** The compiler controls all DOM emission — there is no mechanism for unsanitized string interpolation. TextNode content is always escaped. DataNode outputs are typed, not raw strings |
| A04: Insecure Design | Architecture-level security flaws | **Enforceable via IR validation.** The validator can check for anti-patterns: DataNodes without error states, StateMachines without timeout transitions, missing rate-limit annotations on mutation EffectNodes |
| A05: Security Misconfiguration | Missing headers, default creds, verbose errors | **Compiler responsibility.** The DOM compiler emits security headers (CSP, X-Frame-Options, etc.) by default. No user configuration needed — secure defaults are compiled in |
| A06: Vulnerable Components | Outdated dependencies with CVEs | **Structurally eliminated.** Voce IR has zero runtime dependencies. No node_modules, no supply chain. The compiled output is self-contained |
| A07: Auth Failures | Weak passwords, missing MFA, session issues | **Partially enforceable.** Auth state machines can be validated for completeness (login → session → timeout → re-auth). Session management is a compiler concern, not a user concern |
| A08: Data Integrity Failures | Unverified updates, insecure CI/CD | **Partially applicable.** IR validation ensures structural integrity. Compiled output is deterministic (same IR = same output) enabling integrity verification |
| A09: Logging & Monitoring | Insufficient audit trails | **Compiler concern.** The compiler can emit structured logging for state transitions, data fetches, and error states. The user describes *what* to log via EffectNode; the compiler handles *how* |
| A10: SSRF | Server-side request forgery | **Enforceable.** DataNode source declarations are validated — allowed origins can be whitelisted at the IR level. The compiler only emits fetch calls to declared sources |

### 1.2 Security Validation Pass

Add a security validation pass to the validator (`packages/validator/src/passes/security.rs`):

**Checks:**
- Every DataNode with `source: api` declares allowed origins
- Every DataNode with user-supplied input declares input type constraints (length, pattern, type)
- Every StateMachine handling authentication has timeout transitions
- Every EffectNode that performs mutations declares idempotency (retry-safe or not)
- Every RouteMap with protected routes declares authentication guards
- No DataNode outputs are used in raw HTML contexts (always through typed TextNode or MediaNode)
- Every form-like interaction (GestureHandler → DataNode mutation) has CSRF protection annotation

**Severity levels:**
- **Error (blocks compilation):** XSS-possible patterns, unguarded protected routes, DataNodes without error states
- **Warning (reported but compiles):** Missing rate-limit annotations, missing timeout transitions, overly permissive CORS origins

### 1.3 Compiled Output Security

The DOM compiler emits secure defaults:

```html
<!-- Auto-generated security headers -->
<meta http-equiv="Content-Security-Policy" 
      content="default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'">
<meta http-equiv="X-Content-Type-Options" content="nosniff">
<meta http-equiv="X-Frame-Options" content="DENY">
<meta name="referrer" content="strict-origin-when-cross-origin">
```

- All inline JavaScript is CSP-nonced or hashed
- All fetch calls use HTTPS by default (HTTP requires explicit opt-in with justification)
- Cookie attributes include `Secure`, `HttpOnly`, `SameSite=Strict` by default
- No `eval()`, `innerHTML`, or `document.write()` in compiled output — ever

### 1.4 Security as the Fourth Concern

Security joins Stability, Experience, and Accessibility as a core concern. It doesn't need to be a separate "pillar" — it's part of Stability (a secure app is a stable app). But the validator and compiler must treat it with the same rigor.

---

## 2. Testing Strategy for Codeless Applications

### 2.1 The Testing Problem

Traditional testing verifies code behavior. Voce IR has no user-authored code. So what do you test?

**Answer: You test the contract between intent and output.** The IR is a formal specification of what the application should do. Tests verify that the specification is correct and the compiled output matches it.

### 2.2 Testing Layers

#### Layer 1: IR Validation (Already Planned)
The validator is the first test layer. It catches structural errors, type mismatches, missing accessibility, and security anti-patterns before compilation.

#### Layer 2: State Machine Testing (New)
Every StateMachine in the IR is a formal specification. It can be tested exhaustively:

```
For each StateMachine:
  - Verify all states are reachable from the initial state
  - Verify no deadlock states (states with no outgoing transitions except terminal states)
  - Verify all transitions have defined effects
  - Verify guard conditions are satisfiable
  - Generate test traces: sequences of events that exercise all transitions
  - Check for conflicting transitions (same event, same state, different guards that could both be true)
```

This is model-based testing — the state machine IS the model. The validator can generate test cases automatically.

#### Layer 3: Compiled Output Testing (New)
After compilation, verify the output:

- **Structural tests:** Does the HTML contain the expected elements? Do ARIA attributes match SemanticNodes?
- **Behavioral tests:** Do click handlers trigger the correct state transitions? Does the state machine runtime produce correct outputs?
- **Visual regression tests:** Render the compiled output in a headless browser, screenshot it, compare against a reference image
- **Accessibility tests:** Run axe-core on compiled output (already planned)
- **Performance tests:** Measure TTI, bundle size, Lighthouse score
- **Security tests:** Verify CSP headers, cookie attributes, no inline event handlers, XSS resistance

#### Layer 4: Intent-Level Testing (New — Phase 3+)
Test the full pipeline: intent → IR → compiled output → rendered result

- **Golden intent tests:** A library of (intent description, expected output properties) pairs
  - "Build a login form" → output has: form element, email/password inputs, submit button, ARIA labels, CSRF token, autocomplete attributes
- **Fuzzing:** Generate random intent variations, verify the pipeline doesn't crash and output passes validation
- **Regression tests:** When a user reports a bug, save the (intent, IR, expected fix) as a test case

#### Layer 5: AI-Generated Test Scenarios (New — Phase 3+)
The AI bridge can generate test scenarios from the intent:

```
User: "Build a product page with add-to-cart"

AI generates IR AND test scenarios:
  - "User clicks add-to-cart → cart count increments"
  - "User clicks add-to-cart with no stock → error message appears"  
  - "Screen reader announces cart update via LiveRegion"
  - "Keyboard user can tab to add-to-cart and activate with Enter"
  - "Reduced motion user sees no animation on cart update"
```

These test scenarios are compiled into assertions that run against the compiled output.

### 2.3 Test Generation from IR

The IR contains enough information to auto-generate many tests:

| IR Node | Auto-Generated Tests |
| ------- | -------------------- |
| StateMachine | Reachability, deadlock-freedom, transition coverage |
| GestureHandler | Keyboard equivalent works, touch/click equivalent works |
| DataNode | Error state renders correctly, loading state renders correctly |
| Transition/Sequence | Reduced motion alternative works, animation completes |
| SemanticNode | ARIA attributes present in output, focus order correct |
| RouteMap | All routes reachable, guards prevent unauthorized access |
| ThemeNode | Light/dark mode switch works, high-contrast mode works |
| ResponsiveRule | Layout correct at each declared breakpoint |

### 2.4 The Test Report

Every compilation produces a test report alongside the output:

```
voce-compile-dom input.voce -o dist/index.html

✓ Validation: 0 errors, 0 warnings
✓ Security: CSP headers, no XSS vectors, HTTPS-only fetches
✓ Accessibility: 14 SemanticNodes, 3 LiveRegions, full keyboard coverage
✓ State machines: 4 machines, all reachable, no deadlocks
✓ Animations: 6 transitions, all have ReducedMotion alternatives
✓ Output: 8.2KB, estimated TTI <40ms
✓ axe-core: 0 violations (run with --headless flag)

Output: dist/index.html
Report: dist/index.voce-report.json
```

---

## 3. Application Documentation System

### 3.1 The Documentation Problem

If there's no source code, how does the user understand what they built? How does the AI maintain context across sessions? How does a team (or a future AI) pick up where the previous conversation left off?

### 3.2 The Application Manifest

Every Voce IR application generates an **Application Manifest** — a human-readable document that describes the application in terms the user understands:

```markdown
# My Sneaker Store — Application Manifest

Generated: 2026-04-15
Last modified: 2026-04-18
IR version: 0.3.2
Compiled targets: DOM

## Pages
- **Home** (`/`) — Hero with 3D shoe viewer, featured products grid, newsletter signup
- **Product Detail** (`/product/:id`) — 3D model, size selector, add-to-cart, reviews
- **Cart** (`/cart`) — Line items, quantity controls, checkout button

## State Machines
- **AuthMachine** — anonymous → logged_in → session_expired (timeout: 30min)
- **CartMachine** — empty → has_items → checkout_in_progress → order_confirmed
- **ProductViewerMachine** — loading → interactive → error

## Data Sources
- `GET /api/products` — Product catalog (cached: 5min)
- `GET /api/products/:id` — Single product (cached: 1min)
- `POST /api/cart` — Cart mutations (no cache, CSRF protected)

## Accessibility
- Full keyboard navigation on all interactive elements
- Screen reader announcements for: cart updates, form errors, page transitions
- Reduced motion alternatives for: hero animation, page transitions, add-to-cart effect
- High contrast theme variant available

## Security
- Authentication required for: /cart, /checkout
- CSRF protection on all mutation endpoints
- CSP: default-src 'self', script-src 'self'
- All API calls over HTTPS

## Design Decisions (from conversation history)
- Dark theme chosen for "premium feel" (conversation turn 3)
- 3D viewer uses orbit controls, mouse-reactive rotation at 0.4 sensitivity (adjusted from 1.0 in turn 7)
- Add-to-cart uses gold particle burst (changed from white in turn 7)
```

### 3.3 How the Manifest is Generated

The manifest is derived from three sources:

1. **IR analysis** — pages, state machines, data sources, accessibility features, security configuration are all extractable from the IR structure
2. **Intent history** — the conversation that produced the IR. Key decisions, adjustments, and rationale are logged and summarized
3. **Compilation report** — performance metrics, security audit, accessibility audit results

The manifest is regenerated on every compilation. It's the "README" for an application with no source code.

### 3.4 Intent History Log

Every conversation session that modifies the IR produces a log entry:

```json
{
  "session": "2026-04-18T14:30:00Z",
  "turns": [
    {
      "intent": "Add a newsletter signup form to the footer",
      "changes": ["Added Container(footer-newsletter)", "Added StateMachine(newsletter-form)", "Added DataNode(newsletter-submit)"],
      "validation": "passed",
      "notes": "User specified email-only, no name field"
    },
    {
      "intent": "Make the submit button more prominent",
      "changes": ["Updated Surface(newsletter-cta) fill to accent color", "Added Transition(newsletter-cta-hover)"],
      "validation": "passed"
    }
  ]
}
```

This log serves multiple purposes:
- **AI context:** The AI bridge reads the log to understand the application's history and the user's preferences
- **Collaboration:** A second person (or AI) can understand how the application evolved
- **Debugging:** When something breaks, trace back to the conversation turn that introduced it
- **Audit trail:** For compliance contexts, the log shows who requested what and when

### 3.5 Architecture Diagram Generation

The IR is a formal graph. Generate visual architecture diagrams automatically:

- **Page map:** All routes and their transitions
- **State machine diagrams:** For each machine, states and transitions (Mermaid or SVG)
- **Data flow diagram:** DataNodes, their sources, and which visual nodes consume them
- **Accessibility tree:** Parallel semantic tree visualization

These can be emitted alongside the compiled output or viewed in the Visual Inspector.

---

## 4. Tooling & CLI Ecosystem

### 4.1 Core CLI Tools

| Command | Purpose | Phase |
| ------- | ------- | ----- |
| `voce validate <file>` | Validate IR blob (structural, a11y, security, state machines) | 1 |
| `voce compile <file> --target dom` | Compile IR to target output | 2 |
| `voce inspect <file>` | Pretty-print IR as human-readable summary (not code — structured description) | 1 |
| `voce json2bin <file.json>` | Convert JSON canonical format to binary FlatBuffer | 1 |
| `voce bin2json <file.voce>` | Convert binary FlatBuffer to JSON canonical format | 1 |
| `voce diff <a.voce> <b.voce>` | Semantic diff between two IR blobs (node-level changes, not byte-level) | 2 |
| `voce test <file>` | Run auto-generated tests from IR (state machine coverage, a11y, security) | 2 |
| `voce manifest <file>` | Generate the application manifest document | 2 |
| `voce preview <file>` | Compile + serve + hot-reload on IR changes | 2 |
| `voce generate "<intent>"` | Generate IR from natural language (AI bridge) | 3 |
| `voce patch "<change>"` | Apply incremental change to existing IR | 3 |
| `voce report <file>` | Full compilation report (perf, a11y, security, test results) | 2 |

### 4.2 The `voce` CLI as the Development Environment

The `voce` CLI isn't just a build tool — it's the primary development interface (alongside the AI conversation). Developers should be able to:

```bash
# Start a new project from conversation
voce generate "Build a SaaS landing page with pricing tiers, 
  dark mode, and a demo request form"

# Preview it
voce preview output.voce

# Iterate
voce patch "Make the hero section taller, add a subtle 
  gradient background"

# Check everything
voce report output.voce

# See what you built
voce manifest output.voce

# Deploy
voce compile output.voce --target dom -o dist/
```

### 4.3 The Visual Inspector (Expanded from Phase 5)

The inspector is not just a debugging tool — it's the "IDE" for Voce IR applications. Consider pulling it earlier (Phase 2-3) because it's load-bearing for adoption.

**Inspector capabilities:**
- Scene graph overlay (click any element → see IR node, state machine state, data bindings)
- State machine visualizer (live state, transitions, history)
- Animation timeline (scrubable, frame-by-frame)
- Accessibility auditor (live a11y tree, focus order, screen reader simulation)
- Security audit panel (CSP status, data flow, auth state)
- Performance profiler (frame timing, memory, network)
- **Conversational debugging:** "Why doesn't the modal close when I click outside?" → AI traces the state machine and identifies the issue
- **Application manifest viewer:** Navigate the auto-generated documentation

### 4.4 Editor/IDE Integration

For developers who want to work alongside the CLI:

- **VS Code extension:** Syntax highlighting for JSON canonical format, IR node tree view, state machine diagram preview, live validation errors
- **Cursor/Claude Code integration:** If the user is already using an AI coding tool, Voce IR should work within it — not replace it. The `voce` CLI commands should work from any terminal

---

## 5. AI Model Strategy & Design Pattern System

### 5.1 AI Roles in the Pipeline

Different parts of the pipeline have different AI requirements:

| Role | Task | Model Requirements | Recommended |
| ---- | ---- | ------------------ | ----------- |
| **Intent Parser** | Understand what the user wants, ask clarifying questions | Strong reasoning, conversation, context retention | Claude Opus / Sonnet |
| **IR Generator** | Emit valid JSON IR from parsed intent | Strong structured output, schema adherence | Claude Sonnet with tool use / structured output |
| **Repair Agent** | Fix validation errors in generated IR | Good at precise, targeted edits to structured data | Claude Sonnet / Haiku (fast, focused) |
| **Design Advisor** | Suggest improvements to aesthetics, UX patterns | Design knowledge, visual reasoning | Claude Opus (best design reasoning) |
| **Test Generator** | Generate test scenarios from intent + IR | Understanding of expected behavior, edge cases | Claude Sonnet |
| **Documentation Writer** | Generate manifest, architecture descriptions | Clear writing, ability to summarize technical structures | Claude Sonnet |
| **Debug Assistant** | Trace bugs through state machines, suggest fixes | Strong analytical reasoning, IR structure understanding | Claude Opus |

**Key principle:** Use the right model for each role. The intent parser needs deep reasoning (Opus). The repair agent needs speed and precision (Haiku/Sonnet). The IR generator needs reliable structured output (Sonnet with tool use).

### 5.2 Multi-Agent Architecture

The AI bridge could use a multi-agent architecture:

```
User Intent
    │
    ▼
[Intent Agent] — Opus — Understands what the user wants,
    │              asks clarifying questions, resolves ambiguity
    ▼
[Design Agent] — Opus — Applies design patterns, color theory,
    │              typography rules, UX best practices
    ▼
[Generator Agent] — Sonnet — Emits valid JSON IR using
    │                 structured output / tool use
    ▼
[Validator] — Rust (not AI) — Structural, a11y, security checks
    │
    ├── Valid → [Test Generator] — Sonnet — Auto-generate test scenarios
    │              │
    │              ▼
    │          [Compiler] — Rust — Emit optimized output
    │
    └── Invalid → [Repair Agent] — Haiku/Sonnet — Fix errors, retry
```

### 5.3 Baked-In Knowledge: The Standards Library

The AI bridge should have deep knowledge of established best practices, loaded as context (not just training data):

#### Web Security Standards
- **OWASP Top 10** — mapped to IR validation rules (see Section 1)
- **OWASP ASVS** (Application Security Verification Standard) — for more granular security validation
- **CSP best practices** — default restrictive, loosen only with justification
- **Cookie security** — Secure, HttpOnly, SameSite defaults

#### Accessibility Standards
- **WCAG 2.2 AA** — mapped to IR validation rules (already planned)
- **ARIA Authoring Practices Guide** — correct ARIA patterns for common widgets (combobox, dialog, tab panel, etc.)
- **Inclusive Design Principles** — broader than WCAG, covers cognitive accessibility

#### UI/UX Best Practices
- **Nielsen's 10 Usability Heuristics** — encoded as design advisor guidelines
- **Material Design 3 / Apple HIG** — common interaction patterns, spacing systems, touch target sizes
- **Web Vitals thresholds** — LCP, FID, CLS targets as compiler optimization goals
- **Common UI patterns** — login flows, checkout flows, data tables, search interfaces, navigation patterns

### 5.4 Design Pattern Library — "Style Packs"

This is your LoRA concept, adapted for IR generation. Instead of fine-tuning the model, create **curated collections of IR examples** that embody specific design languages:

#### What a Style Pack Contains

```
style-packs/
├── minimal-saas/
│   ├── manifest.json          # Name, description, design principles
│   ├── tokens.json            # Color palette, typography, spacing scale
│   ├── patterns/
│   │   ├── hero.voce.json     # Hero section pattern
│   │   ├── pricing.voce.json  # Pricing tier pattern
│   │   ├── cta.voce.json      # Call-to-action pattern
│   │   ├── nav.voce.json      # Navigation pattern
│   │   └── footer.voce.json   # Footer pattern
│   └── examples/
│       ├── landing.voce.json  # Full landing page example
│       └── dashboard.voce.json
├── editorial/
│   ├── manifest.json
│   ├── tokens.json
│   ├── patterns/...
│   └── examples/...
├── brutalist/
├── luxury-ecommerce/
├── developer-docs/
└── mobile-first-app/
```

#### How Style Packs Work

1. User says: "Build a SaaS landing page, minimal style"
2. AI bridge loads the `minimal-saas` style pack
3. Design tokens from the pack are applied to the ThemeNode
4. Relevant patterns are used as few-shot examples for the IR generator
5. The generator produces IR that follows the style pack's design language

#### Style Pack as "LoRA-equivalent"

Style packs are to Voce IR what LoRAs are to Stable Diffusion:
- They're **small, composable** additions to the base capability
- They **don't require retraining** — they're loaded as context/examples
- They can be **community-contributed** — anyone can create and share a style pack
- They can be **mixed** — "Use the luxury-ecommerce layout with the brutalist color palette"

#### Training Future Fine-Tuned Models

Style packs also serve as structured training data. Each pack contains (pattern description, IR output) pairs. Collect enough packs and you have a fine-tuning dataset:
- 20 style packs × 10 patterns each × 50 parameter variations = 10,000 training pairs
- This is enough for meaningful fine-tuning when the time comes (Phase 3+)

### 5.5 Learning from the Best — Design Pattern Scraping

To build high-quality style packs, study the best existing UIs:

1. **Curate a reference library** of excellent web applications (Stripe, Linear, Vercel, Apple, etc.)
2. **Analyze them** — extract color palettes, typography scales, spacing systems, animation patterns, layout structures
3. **Encode them** as Voce IR patterns — translate the design decisions into IR tokens and node structures
4. **Use them as few-shot examples** for the AI bridge

This is not automated scraping — it's deliberate curation of design excellence, translated into the IR vocabulary. Think of it as "teaching the AI what good design looks like" by showing it examples in the language it speaks (IR, not CSS).

### 5.6 Contextual AI — Maintaining Understanding Across Sessions

The AI system should maintain persistent context about each application:

- **Application manifest** (auto-generated) — what exists, how it works
- **Intent history log** — every conversation turn and what it changed
- **User preferences** — design preferences, accessibility requirements, security constraints learned over time
- **Style pack selections** — which design language the user prefers
- **Validation history** — recurring issues, patterns the AI tends to get wrong for this project

This context is loaded into the AI bridge at the start of every session. The AI doesn't start from scratch — it knows the application, the user's preferences, and the project's history.

---

## 6. The Complete Voce IR Experience

Putting it all together — the full developer experience:

```
1. USER: "Build a fintech dashboard with transaction history,
         account balance, and spending charts. Must be WCAG AA,
         SOC 2 compliant, dark mode."

2. INTENT AGENT (Opus): Clarifies scope, maps requirements
   → Auth required? Yes → adds auth state machine
   → Transaction data source? REST API → adds DataNode with cache policy
   → SOC 2? → enables strict security validation, audit logging

3. DESIGN AGENT (Opus): Loads fintech style pack, selects patterns
   → Dashboard layout, data table, chart components
   → Dark theme tokens, accessible color contrast ratios
   → Applies Nielsen heuristics (visibility, feedback, error prevention)

4. GENERATOR (Sonnet): Emits JSON IR with structured output
   → Uses style pack patterns as few-shot examples
   → Includes all security annotations, a11y nodes, state machines

5. VALIDATOR (Rust): Checks everything
   ✓ Structural completeness
   ✓ WCAG AA compliance (SemanticNodes, keyboard, contrast, motion)
   ✓ OWASP security (auth guards, CSRF, CSP, input validation)
   ✓ State machine completeness (no deadlocks, all reachable)

6. TEST GENERATOR (Sonnet): Creates test scenarios
   → "Login with invalid credentials → error message"
   → "Transaction list loads → screen reader announces count"
   → "Chart renders → keyboard navigable data points"

7. COMPILER (Rust): Emits optimized DOM output
   → Single HTML file, <15KB, <50ms TTI
   → Security headers baked in
   → ARIA attributes from SemanticNodes
   → Test report generated

8. DOCUMENTATION:
   → Application manifest generated
   → Architecture diagrams (routes, state machines, data flow)
   → Security audit report
   → Accessibility audit report

9. USER: Reviews preview, iterates via conversation
   "Make the balance card more prominent"
   "Add a spending trend sparkline"
   "The chart colors need more contrast"

10. Each iteration: validate → test → compile → document → preview
```

The user never writes code, never configures security headers, never adds ARIA attributes, never writes tests, never writes documentation. But they have all of it — automatically, correctly, and up to date.

---

*This document should be read alongside `DEEP_RESEARCH.md` for the full research context.*
