# Sprint 13 — State & Interaction Compilation

**Status:** Planned
**Goal:** Compile state machines, gesture handlers, data nodes, and effect nodes to minimal JavaScript. State changes trigger surgical DOM mutations — only affected nodes update, no virtual DOM diffing. After this sprint, interactive pages compile with working state transitions, event handlers, and optional data fetching.
**Depends on:** Sprint 12 (layout compilation, styled HTML output)

---

## Deliverables

1. StateMachine → JS state variable + transition function (~20 lines per machine)
2. GestureHandler → `addEventListener` with direct event binding
3. DataNode → TanStack Query integration (conditional — only when data nodes present)
4. EffectNode → imperative calls on state transitions
5. Surgical DOM mutation system (state change → targeted element updates)
6. `data-voce-bind` attribute system for DOM mutation targets
7. Integration test: interactive page with state transitions

---

## Tasks

### 1. State Machine Compilation (`lower/state_machine.rs`)

Each StateMachine compiles to a self-contained JS block:
