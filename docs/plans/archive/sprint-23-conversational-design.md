# Sprint 23 — Conversational Design

**Status:** Planned
**Goal:** Implement the anti-vibe-coding conversational flow: one question at a time, structured discovery phase, brief creation from dialogue, plan confirmation before generation. After this sprint, the AI bridge conducts a thoughtful design conversation rather than blindly generating from a single prompt.
**Depends on:** Sprint 22 (multi-agent architecture, Discovery Agent)

---

## Deliverables

1. `ConversationEngine`: stateful multi-turn dialogue manager
2. One-question-at-a-time flow with topic progression (purpose -> audience -> content -> style -> constraints)
3. Readiness score computed after each answer, displayed to user
4. Brief builder: accumulates answers into a structured `DiscoveryBrief`
5. Plan confirmation step: shows planned sections/layout, user approves before generation
6. `voce design` interactive CLI command (replaces raw `voce generate` for conversational flow)
7. Conversation transcript saved to `.voce/sessions/`

---

## Tasks

### 1. Conversation Engine (`src/conversation/engine.ts`)

Stateful engine managing a multi-turn dialogue. Tracks: current phase (discovery, design, confirmation, generating), answered topics, readiness score, accumulated brief. Methods: `start()`, `respond(userInput): ConversationTurn`, `getReadinessScore(): number`, `getBrief(): DiscoveryBrief`. Each turn returns the AI's next question plus metadata (phase, progress percentage, readiness score).

### 2. Topic Graph (`src/conversation/topics.ts`)

Define the ordered topic tree the Discovery Agent walks through: purpose/goal, target audience, key content sections, calls to action, tone/brand, visual preferences, technical constraints. Each topic has: a prompt template, extraction logic (parse answer into brief field), skip conditions (e.g., skip brand if user said "no preference"), and a weight toward readiness score.

### 3. One-Question Flow (`src/conversation/flow.ts`)

Enforce single-question turns. The AI must ask exactly one question per turn — no multi-part questions, no walls of text. If the user's answer covers multiple topics, advance through them and ask about the next uncovered one. Validate via post-processing: if the AI response contains "?" more than once, regenerate with stricter instruction.

### 4. Brief Builder (`src/conversation/brief-builder.ts`)

Incrementally constructs `DiscoveryBrief` from conversation answers. Each answer is parsed and slotted into the appropriate brief field. Handles ambiguity: if an answer is unclear, flag the topic for re-ask. Exports the brief as YAML (for `.voce/brief.yaml`) and as typed object (for pipeline input).

### 5. Plan Confirmation (`src/conversation/confirmation.ts`)

Before triggering generation, present a structured plan: "I'll build a landing page with: Hero (headline + CTA), Features (3-column grid), Testimonials (carousel), Footer. Estimated 12 nodes. Proceed?" User can approve, request changes, or add sections. Only after explicit confirmation does generation begin.

### 6. Interactive CLI (`src/cli/design-command.ts`)

`voce design` enters an interactive readline session. Shows phase indicator, readiness score bar, turn count. Colors: questions in blue, confirmations in green, warnings in yellow. Supports `--non-interactive` flag that accepts a brief.yaml directly (for CI/scripting). On completion, pipes brief into the agent orchestrator from S22.

### 7. Session Persistence (`src/conversation/session.ts`)

Save conversation state to `.voce/sessions/<timestamp>.json` after each turn. On `voce design --resume`, reload the most recent session and continue where the user left off. Include: all turns, current brief state, readiness score, phase. Prune sessions older than 30 days.

---

## Files to Create

- `packages/ai-bridge/src/conversation/engine.ts`
- `packages/ai-bridge/src/conversation/topics.ts`
- `packages/ai-bridge/src/conversation/flow.ts`
- `packages/ai-bridge/src/conversation/brief-builder.ts`
- `packages/ai-bridge/src/conversation/confirmation.ts`
- `packages/ai-bridge/src/conversation/session.ts`
- `packages/ai-bridge/src/cli/design-command.ts`
- `tests/ai-bridge/conversation/engine.test.ts`
- `tests/ai-bridge/conversation/brief-builder.test.ts`

---

## Acceptance Criteria

- [ ] `voce design` starts an interactive conversation asking one question at a time
- [ ] Readiness score increases with each answered topic, displayed after every turn
- [ ] Conversation covers all required topics before reaching readiness >= 70
- [ ] Brief is correctly assembled from conversation answers (validated against schema)
- [ ] Plan confirmation shows structured section list before generation begins
- [ ] User can reject plan and request changes without restarting
- [ ] `--resume` flag successfully continues an interrupted session
- [ ] `--non-interactive` accepts a brief.yaml and skips straight to generation
- [ ] Session files are saved to `.voce/sessions/` with full conversation state
- [ ] No turn contains more than one question from the AI
