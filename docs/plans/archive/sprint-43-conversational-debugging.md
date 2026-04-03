# Sprint 43 — Conversational Debugging

**Status:** Planned
**Phase:** 5 (Visual Inspector & Tooling)
**Depends on:** S40 (state/animation inspector), S42 (CMS visual editing)

---

## Goal

Enable AI-assisted debugging: describe a bug in natural language, and the system traces the state machine, identifies the IR node, and proposes a fix.

---

## Deliverables

- Bug description input: natural language ("the button doesn't change color on hover")
- AI traces state machine path to identify which transition/guard is failing
- Node identification: AI pinpoints the specific IR node(s) involved
- Fix proposal: AI generates an IR patch to resolve the issue
- State replay: reproduce the bug by replaying a state transition sequence
- Error context: AI explains why the current IR produces the observed behavior
- Fix preview: show before/after comparison before applying the patch
- Integration with inspector: AI can reference visible inspector state
- Conversation history: multi-turn debugging sessions with context

---

## Acceptance Criteria

- [ ] "The button doesn't respond to clicks" → AI identifies missing GestureHandler
- [ ] "The animation is janky" → AI identifies long transition duration or missing easing
- [ ] State replay reproduces the reported bug scenario
- [ ] Fix proposals generate valid IR patches
- [ ] Before/after preview shows the fix effect
- [ ] Multi-turn conversation maintains debugging context
- [ ] AI references inspector data (current state, animation timeline) in explanations
