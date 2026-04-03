# Voce IR — Memory, Decision Tracking & Project Continuity

**Date:** 2026-04-02
**Status:** Core design principle
**Purpose:** Define how Voce IR maintains persistent memory across sessions, tracks decisions, enforces consistency with the project brief, and prevents drift from the "north star."

---

## 0. Why This Matters More for Voce IR Than Any Other Tool

Traditional development has built-in memory: source code. You can read what was decided by reading the code. Comments explain why. Git blame shows who and when. Pull requests document discussion.

Voce IR has no source code. The IR is binary. The authoring interface is conversation. If the system doesn't remember what was discussed, why a decision was made, and what the original brief said — **every session starts from scratch and the quality advantage of the conversational approach collapses.**

Memory is what makes Voce IR a platform for building serious applications over weeks and months, not just a demo generator for a single session.

---

## 1. The Memory Architecture

### 1.1 Three Layers of Memory

```
┌─────────────────────────────────────────────────┐
│  Layer 1: Project Brief (The North Star)         │
│  What are we building? Why? For whom?            │
│  Success criteria. Non-negotiables.              │
│  Changes require explicit confirmation.          │
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│  Layer 2: Decision Log (The Record)              │
│  Every architectural, design, and feature        │
│  decision with rationale and context.            │
│  Forms a decision tree for consistency checking. │
└──────────────────┬──────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────┐
│  Layer 3: Session Memory (The Context)           │
│  Conversation history, user preferences,         │
│  in-progress work, partial decisions.            │
│  Survives interruptions, resumes seamlessly.     │
└─────────────────────────────────────────────────┘
```

### 1.2 Layer 1: The Project Brief

The brief is the north star. It's established during the first discovery conversation and explicitly versioned when it changes.

**What the brief contains:**

```yaml
project_brief:
  name: "Roast Exchange"
  version: 3                    # incremented on major changes
  last_updated: "2026-04-15"
  
  vision: "A wholesale ordering platform for specialty coffee roasters"
  
  target_audience:
    primary: "Coffee roasters managing wholesale inventory"
    secondary: "Wholesale buyers browsing and ordering"
    
  success_criteria:
    - "Roasters can list products and manage inventory"
    - "Buyers can browse, add to cart, and place orders"
    - "Order status visible to both parties"
    - "WCAG AA accessible"
    - "< 50ms TTI on landing page"
    
  non_negotiables:
    - "Accessibility — every interactive element must be keyboard navigable"
    - "Mobile-first — buyers will primarily use phones"
    - "Real-time order updates — both parties see status changes immediately"
    
  out_of_scope:
    - "Payment processing (handled externally via Stripe)"
    - "Roaster onboarding/verification (manual for MVP)"
    
  style_direction:
    references: ["stripe.com", "linear.app"]
    feel: "Clean, professional, data-dense. Dark sidebar, light content."
    
  technical_decisions:
    backend: "Supabase"
    auth: "Supabase Auth with email/password"
    hosting: "Cloudflare Pages + Workers"
    
  feature_list:
    - id: "F001"
      name: "Product catalog"
      status: "completed"
      priority: "must-have"
    - id: "F002"
      name: "Shopping cart"
      status: "in-progress"
      priority: "must-have"
    - id: "F003"
      name: "Order management dashboard"
      status: "planned"
      priority: "must-have"
    - id: "F004"
      name: "Analytics for roasters"
      status: "planned"
      priority: "nice-to-have"
```

**How the brief is enforced:**

When the user makes a request, the AI checks it against the brief:

```
User: "Add a blog section to the platform"

AI: "I can add a blog, but I want to flag that it's not in your 
     current feature list or brief. Your north star is a wholesale 
     ordering platform — a blog would be a marketing/content play.
     
     Is this something you want to add to the scope? If so, I'll 
     update the brief and feature list. Or is this something for 
     a separate marketing site?"
```

```
User: "Actually let's skip keyboard navigation on the admin 
       dashboard to save time"

AI: "I can't do that — accessibility is listed as a non-negotiable 
     in your brief. Every interactive element must be keyboard 
     navigable. This isn't a nice-to-have that we can defer.
     
     If you're concerned about timeline, I can prioritize the 
     most critical keyboard paths first and ensure the rest are 
     covered before launch. What's driving the time pressure?"
```

The brief is **not silently overridden**. Changes require explicit confirmation:

```
User: "Fine, add the blog to the scope"

AI: "Updating the brief. Adding:
     
     F005: Blog section
     Status: planned
     Priority: nice-to-have
     
     Note: This expands scope beyond the core ordering platform. 
     I'd recommend completing F002 (cart) and F003 (order 
     management) before starting the blog, since those are 
     must-haves. Agreed?"
```

### 1.3 Layer 2: The Decision Log

Every decision is recorded with context, rationale, and implications:

```yaml
decisions:
  - id: "D001"
    date: "2026-04-10"
    session: "session-003"
    category: "architecture"
    decision: "Use Supabase for backend"
    rationale: "Open-source, Postgres-based, built-in auth, real-time subscriptions. Aligns with the need for real-time order updates."
    alternatives_considered:
      - "Firebase — rejected: less flexible querying for order management"
      - "Custom API — rejected: too much backend work for solo builder"
    implications:
      - "DataNodes will use Supabase adapter"
      - "Auth via Supabase GoTrue"
      - "Real-time via Supabase Realtime (Phoenix channels)"
    status: "active"
    
  - id: "D002"
    date: "2026-04-12"
    session: "session-005"
    category: "design"
    decision: "Dark sidebar navigation with light content area"
    rationale: "Matches Stripe Dashboard reference. Data-dense layout needs high contrast for readability."
    implications:
      - "ThemeNode uses dark palette for nav, light for content"
      - "All data tables must meet contrast ratios against light background"
    status: "active"
    
  - id: "D003"
    date: "2026-04-18"
    session: "session-012"
    category: "design"
    decision: "Changed from dark sidebar to full light theme"
    rationale: "User feedback from roaster beta testers — dark sidebar felt too technical, wanted warmer feel"
    supersedes: "D002"
    implications:
      - "ThemeNode needs full light palette"
      - "Contrast ratios need re-checking for all components"
      - "Nav needs visual hierarchy without dark background (use subtle border or shadow)"
    status: "active"
```

**How the decision log is used:**

1. **Consistency checking.** When the AI generates new IR or makes changes, it checks against active decisions. If a change contradicts a decision, it flags it:

```
AI: "You're asking me to use a tabbed interface for the 
     order dashboard. In session 8 (D007), we decided on 
     a single-page layout because roasters said they need 
     to see all order statuses at once without clicking.
     
     Do you want to revisit that decision? The tab approach 
     would hide some information but reduce scroll distance."
```

2. **Conflict detection.** When a new decision would conflict with existing ones:

```
AI: "If we switch the backend from Supabase to Firebase (your 
     suggestion), that conflicts with two existing decisions:
     
     - D001: Supabase chosen for flexible Postgres querying 
       (Firebase's document model is less suited for order 
       management queries)
     - D009: Real-time updates via Supabase Realtime (we'd 
       need to reimplement with Firestore listeners)
     
     These aren't blockers — Firebase can work — but I want 
     to make sure you're aware of the ripple effects. Want 
     to proceed?"
```

3. **Decision tree for future work.** When adding a new feature, the AI traces relevant decisions:

```
AI: "Adding the order management dashboard. Relevant decisions 
     I'm following:
     
     - D002/D003: Light theme, warm feel
     - D006: Table-based layouts for data-dense views
     - D007: Single-page layout, no tabs
     - D010: Supabase real-time for order status updates
     - D014: Mobile layout prioritizes order list over analytics
     
     Proceeding with these constraints. Let me know if any 
     should be revisited."
```

### 1.4 Layer 3: Session Memory

Session memory handles continuity:

**Interruption recovery:**
```
[Session starts — previous session was interrupted mid-conversation]

AI: "Welcome back. When we last spoke, we were working on the 
     shopping cart checkout flow. I had generated the cart page 
     with line items and quantity controls, and you asked me to 
     add a promo code field. I hadn't generated that yet.
     
     Want to pick up where we left off?"
```

**Cross-session context:**
```
[Session starts — 2 weeks since last session]

AI: "It's been a couple weeks. Quick recap of where the 
     project stands:
     
     Completed: Product catalog (F001), user auth (F006)
     In progress: Shopping cart (F002) — 70% done, needs 
       checkout flow and order confirmation
     Next up: Order management dashboard (F003)
     
     3 open items from last session:
     1. You wanted to revisit the mobile cart layout
     2. The promo code validation needs a backend endpoint
     3. You mentioned wanting to add Apple Pay — not yet 
        in the feature list
     
     What would you like to focus on today?"
```

**User preference learning:**

Over time, the system learns the user's patterns and preferences:

```yaml
user_profile:
  communication_style: "Prefers concise responses, dislikes over-explanation"
  design_preferences:
    - "Gravitates toward minimal, clean designs"
    - "Prefers functional animations over decorative"
    - "Always wants mobile-first"
  decision_patterns:
    - "Tends to scope-creep on nice-to-haves — gentle reminders help"
    - "Makes better design decisions when shown visual options"
    - "Prefers to complete features fully before starting new ones"
  technical_context:
    - "Comfortable with backend concepts (APIs, databases)"
    - "Less familiar with animation/motion design"
    - "Knows Supabase well"
```

---

## 2. Persistence Implementation

### 2.1 Storage Format

All memory is stored as structured files in the project directory:

```
.voce/
├── brief.yaml                    # The north star (Layer 1)
├── decisions/
│   ├── index.yaml                # Decision log index
│   ├── D001-supabase-backend.yaml
│   ├── D002-dark-sidebar.yaml
│   └── ...
├── sessions/
│   ├── session-001.yaml          # Full session transcript + metadata
│   ├── session-002.yaml
│   └── ...
├── memory/
│   ├── user-profile.yaml         # Learned user preferences
│   ├── feature-status.yaml       # Current feature list + status
│   └── open-items.yaml           # Unresolved items from any session
└── snapshots/
    ├── 2026-04-10.voce.json      # IR state snapshot (for rollback)
    ├── 2026-04-15.voce.json
    └── ...
```

**Why files, not a database:** Files are git-trackable. The `.voce/` directory commits alongside the IR. This means:
- Memory is version-controlled
- Collaboration works (multiple people share the decision history)
- Nothing is lost if the service goes down
- The AI loads memory from files, not from a server

### 2.2 Session Persistence

Every conversation turn is persisted immediately — not at session end:

```yaml
# sessions/session-012.yaml
session_id: "session-012"
started: "2026-04-18T14:30:00Z"
status: "interrupted"  # or "completed"
last_turn: 7

turns:
  - turn: 1
    user: "Let's work on the checkout flow today"
    ai_response: "Sure. The cart page is done..."
    timestamp: "2026-04-18T14:30:15Z"
    
  - turn: 2
    user: "Add a promo code field above the total"
    ai_response: "Adding a promo code field..."
    ir_changes: ["Added FormField(promo-code) to Container(cart-summary)"]
    timestamp: "2026-04-18T14:31:02Z"
    
  # ... turn 7 was the last before interruption
  
  - turn: 7
    user: "Make the apply button match the primary CTA style"
    ai_response: null  # interrupted before response
    timestamp: "2026-04-18T14:35:44Z"
```

If the session was interrupted (browser closed, CLI killed, network drop), the next session detects `status: "interrupted"` and offers to resume from the last complete turn.

### 2.3 Crash Recovery

The system should never lose work:

1. **IR saved after every successful generation.** Not just at session end — after every `voce generate` or `voce patch` that produces valid IR.
2. **Snapshots at meaningful milestones.** Completing a feature, finishing a session, before a major change. Stored in `.voce/snapshots/`.
3. **Auto-save draft IR.** Even if validation fails, save the last attempted IR as `.voce/draft.voce.json` so nothing is lost.
4. **Decision log is append-only.** Decisions are never deleted, only superseded. The full history is always available.

---

## 3. Drift Detection — The Brief as Guardrail

### 3.1 What Drift Looks Like

Day 1: "Build a wholesale ordering platform for coffee roasters."
Day 30: "Add a blog."
Day 45: "Add a recipe section."
Day 60: "Can we add a social feed where roasters post updates?"
Day 75: The app is now a social media platform for coffee people, and the ordering flow still doesn't work.

This happens constantly with AI-built projects. Each individual request is reasonable. The cumulative effect is scope creep that destroys the product.

### 3.2 How Voce IR Prevents Drift

**Every request is checked against the brief's feature list and success criteria:**

```
User: "Add a social feed where roasters can post updates"

AI: "That's an interesting idea, but it's a significant scope 
     expansion. Your brief's success criteria focus on ordering 
     and inventory management. A social feed would be a new 
     product surface.
     
     Current must-have features status:
     ✓ F001: Product catalog — done
     ◐ F002: Shopping cart — 70%
     ○ F003: Order management — not started
     ○ F005: Real-time order updates — not started
     
     I'd recommend completing the core ordering flow before 
     adding social features. Want to add 'social feed' to the 
     feature list as a future nice-to-have, or would you like 
     to reprioritize?"
```

**The AI tracks feature completion and surfaces it:**

```
[Start of session — 3 must-haves incomplete]

AI: "Quick status check — you have 3 must-have features 
     remaining: cart checkout, order management, and real-time 
     updates. Last session we started working on a blog (F005, 
     nice-to-have). Want to continue with the blog, or should 
     we focus on the must-haves first?"
```

### 3.3 Decision Consistency Checking

When a new request would contradict an existing decision:

```
User: "Let's switch to a tab-based layout for the dashboard"

AI: "Flagging a potential conflict — in session 8, we decided 
     on a single-page layout (D007) because beta testers wanted 
     to see all order statuses without clicking between tabs.
     
     Has something changed? If the testers now prefer tabs, 
     that's a valid reason to update the decision. I just want 
     to make sure we're not reverting something we deliberately 
     chose."
```

The AI doesn't block the change — it surfaces the context so the user makes an informed choice. If the user confirms, the old decision is superseded and the new one is logged with rationale.

### 3.4 Temporal Consistency

The system can detect contradictions between what was asked at different times:

```
AI: "I noticed something — on April 10th you said 'mobile 
     layout should prioritize the order list over analytics' 
     (D014). Today you're asking me to put the analytics 
     dashboard as the first thing users see on mobile.
     
     Which direction do you want to go? Both are valid — I 
     just want to make sure we're intentionally changing 
     course, not accidentally contradicting ourselves."
```

---

## 4. The Memory-Informed Conversation

### 4.1 How Memory Changes the AI's Behavior

**Without memory (every other AI tool):**
- Every session starts with "What are you building?"
- Same mistakes get made repeatedly
- No awareness of previous decisions or rationale
- User must manually maintain consistency
- Scope creep goes undetected

**With memory (Voce IR):**
- Sessions resume with context: "Last time we were working on X"
- Decisions are tracked and enforced: "This contradicts decision D007"
- Preferences are learned: "You prefer minimal animations" (doesn't ask again)
- Brief drift is caught: "This feature isn't in your scope"
- Feature status is always visible: "3 must-haves remaining"
- The AI gets better over time for this specific project

### 4.2 Memory Across AI Providers

Since Voce IR is AI-agnostic (MCP server, SDK, any provider), memory must work regardless of which AI generates the IR:

- All memory is in `.voce/` files (not in any AI's conversation history)
- The MCP server exposes memory as resources: `voce://project/brief`, `voce://project/decisions`, `voce://project/session-history`
- Any AI provider reads these resources at session start
- Any AI provider writes decisions and session logs through MCP tools

This means you could use Claude Code one day and Cursor the next — the project memory is continuous because it lives in the project, not the AI.

### 4.3 The `voce memory` CLI

```bash
# View current brief
voce brief

# View decision log
voce decisions
voce decisions --category design
voce decisions --since 2026-04-15

# View feature status
voce features
voce features --status in-progress

# View session history
voce sessions
voce sessions --last 5

# Search memory
voce memory search "sidebar navigation"

# View open items
voce memory open-items

# Export project context (for onboarding a new collaborator or AI)
voce memory export > project-context.md
```

---

## 5. How This Relates to Existing Tools

### Tools Focused on AI Project Memory

Several tools have emerged focusing on persistent memory for AI-assisted development:

- **Windsurf/Cascade memories** — stores user preferences and project context across sessions
- **Claude Code's CLAUDE.md + memory system** — file-based project instructions and conversation memory
- **Cursor's .cursorrules** — project-level AI instructions
- **mem0, Zep, Letta** — memory layers for AI applications

**What Voce IR adds beyond these:**
- **Brief enforcement** — not just remembering context, but actively checking requests against the north star
- **Decision tracking with supersession** — decisions form a chain, contradictions are surfaced
- **Feature-level tracking** — the AI knows what's done, what's in progress, what's planned
- **Temporal consistency** — catching contradictions between day 1 and day 100
- **Cross-provider persistence** — memory lives in the project, not the AI tool

### What We Can Leverage

- The `.voce/` directory structure is inspired by Claude Code's `.claude/` memory system
- YAML for human-readable memory files (git-diffable, manually editable as escape hatch)
- The MCP resource pattern for exposing memory to any AI provider
- Session transcript storage follows common patterns from chat history systems

### What We Must Build

- Brief enforcement logic (checking requests against success criteria and feature list)
- Decision conflict detection (cross-referencing new decisions against existing ones)
- Drift detection (tracking cumulative scope changes against original brief)
- Session recovery (detecting interrupted sessions, offering resume)
- IR state snapshots (for rollback after bad changes)

---

## 6. Phase Mapping

### Phase 1 (Schema + Validator)
- Define the `.voce/` directory structure
- Implement brief.yaml and decisions/ format
- Session persistence (write on every turn, detect interruption)
- `voce brief` and `voce decisions` CLI commands

### Phase 2 (DOM Compiler)
- IR state snapshots at milestones
- Feature status tracking
- `voce features` and `voce memory` CLI commands

### Phase 3 (AI Bridge)
- Brief enforcement in Discovery Agent (check requests against north star)
- Decision conflict detection in Architecture Agent
- Drift detection (cumulative scope tracking)
- Session resume with context summary
- User preference learning
- MCP resources for memory (`voce://project/brief`, etc.)
- Memory export for collaborator onboarding

---

## 7. The Core Principle

**Memory is what turns a conversation into a collaboration.**

Without memory, each session is a transaction — user asks, AI generates, relationship resets. With memory, the AI becomes a partner that knows the project intimately, remembers why decisions were made, protects the vision from drift, and gets better at serving this specific project over time.

The brief is the north star. Decisions are the record. Sessions are the context. Together, they ensure that what gets built on day 100 is consistent with what was decided on day 1 — unless the team deliberately chose to change course, in which case the change is documented and its implications are understood.

---

*This is a core design document. It should influence every aspect of the AI bridge (Phase 3) and the `.voce/` project structure (Phase 1).*
