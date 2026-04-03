# Sprint 22 — Multi-Agent Architecture

**Status:** Planned
**Goal:** Replace the single-shot generator with a multi-agent pipeline: Discovery Agent (quality gate), Design Agent (UX patterns), Generator Agent (IR emission), and Repair Agent (validation fix loop). After this sprint, generation quality improves dramatically because each agent focuses on one concern, and invalid IR self-heals.
**Depends on:** Sprint 21 (AI bridge foundation, basic generation working)

---

## Deliverables

1. Agent base class with typed input/output, system prompt, and token budget
2. Discovery Agent: extracts structured requirements from vague prompts, blocks generation until sufficient context
3. Design Agent: selects layout patterns, color strategy, typography scale, spacing system
4. Generator Agent: emits complete Voce IR JSON using structured output mode
5. Repair Agent: takes validation errors + IR, outputs corrected IR (target >99% validity after 2 cycles)
6. `AgentOrchestrator`: chains agents, manages context passing, enforces ordering
7. Repair loop integration with `voce validate` — automatic, up to 3 cycles

---

## Tasks

### 1. Agent Base Class (`src/agents/base-agent.ts`)

Define `Agent<TInput, TOutput>` with: `systemPrompt`, `execute(input): Promise<TOutput>`, token budget, model override, retry config. Each agent gets its own system prompt optimized for its role. Log token usage per agent for cost tracking.

### 2. Discovery Agent (`src/agents/discovery-agent.ts`)

Input: raw user prompt. Output: `DiscoveryBrief` — structured object with: purpose, target audience, key sections, CTAs, tone, constraints. The agent asks itself "what's missing?" and flags gaps. Compute a readiness score (0-100). If score < 70, return follow-up questions instead of a brief. This is the quality gate that prevents garbage-in.

### 3. Design Agent (`src/agents/design-agent.ts`)

Input: `DiscoveryBrief`. Output: `DesignSpec` — layout structure (which containers, nesting), typography (font stack, scale), color palette (semantic tokens), spacing rhythm. The agent applies UX heuristics: F-pattern for landing pages, Z-pattern for marketing, single-column for articles. Does not emit IR — only design decisions.

### 4. Generator Agent (`src/agents/generator-agent.ts`)

Input: `DiscoveryBrief` + `DesignSpec` + schema context. Output: complete Voce IR JSON. Uses Claude structured output mode to ensure valid JSON shape. This agent has the full schema context in its system prompt and the design decisions as user context.

### 5. Repair Agent (`src/agents/repair-agent.ts`)

Input: IR JSON + validation error list from `voce validate`. Output: corrected IR JSON. System prompt contains common error patterns and fixes. Runs in a loop: generate fix, re-validate, repeat up to 3 times. Track repair success rate. If 3 cycles fail, return best attempt with remaining errors.

### 6. Orchestrator (`src/agents/orchestrator.ts`)

Chain: Discovery -> Design -> Generator -> Validate -> (Repair loop if needed). Pass context between agents via typed intermediate objects. Support skipping Discovery when the user provides a pre-built brief. Emit events for each phase transition (for UI progress reporting). Total pipeline timeout: 60 seconds.

### 7. Tests (`tests/ai-bridge/agents/`)

Unit tests per agent with mocked Claude responses. Integration test for full orchestrator pipeline. Test repair loop with intentionally broken IR. Test discovery gate blocks on vague prompts ("make something cool" -> follow-up questions). Test design agent produces valid DesignSpec for different page types.

---

## Files to Create

- `packages/ai-bridge/src/agents/base-agent.ts`
- `packages/ai-bridge/src/agents/discovery-agent.ts`
- `packages/ai-bridge/src/agents/design-agent.ts`
- `packages/ai-bridge/src/agents/generator-agent.ts`
- `packages/ai-bridge/src/agents/repair-agent.ts`
- `packages/ai-bridge/src/agents/orchestrator.ts`
- `packages/ai-bridge/src/agents/types.ts` (DiscoveryBrief, DesignSpec, RepairResult)
- `tests/ai-bridge/agents/discovery-agent.test.ts`
- `tests/ai-bridge/agents/repair-agent.test.ts`
- `tests/ai-bridge/agents/orchestrator.test.ts`

---

## Acceptance Criteria

- [ ] Vague prompt ("make me a website") triggers follow-up questions, not generation
- [ ] Detailed prompt ("SaaS landing page with hero, features grid, pricing table, CTA") passes discovery gate
- [ ] Design Agent outputs valid DesignSpec with layout, typography, colors, spacing
- [ ] Generator Agent produces valid Voce IR JSON using structured output
- [ ] Repair Agent fixes common errors: missing SemanticNode, invalid enum values, broken references
- [ ] Repair loop achieves >99% validity rate after 2 cycles on test corpus
- [ ] Full pipeline completes in under 60 seconds for a standard landing page
- [ ] Token usage logged per agent — total pipeline under 50K tokens for simple pages
- [ ] Orchestrator emits phase-transition events suitable for progress UI
- [ ] All unit tests pass with mocked responses; integration test passes with live API
