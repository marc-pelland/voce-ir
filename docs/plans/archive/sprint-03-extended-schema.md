# Sprint 03 — Extended Schema: State, Motion, Navigation

**Status:** Outlined (will be detailed before starting)
**Goal:** Add state management, animation, and navigation schemas. After this sprint, the IR can express: interactive state machines, animated transitions, scroll bindings, gesture handlers, and multi-route applications.
**Depends on:** Sprint 02 (types.fbs, layout.fbs)

---

## Deliverables

1. `packages/schema/schemas/state.fbs` — StateMachine, DataNode, ComputeNode, EffectNode, ContextNode
2. `packages/schema/schemas/motion.fbs` — Transition, Sequence, GestureHandler, ScrollBinding, PhysicsBody, ReducedMotion
3. `packages/schema/schemas/navigation.fbs` — RouteMap, RouteEntry, RouteTransition, RouteGuard
4. Updated `LayoutNode` union (or new master `Node` union) to include state/motion/nav nodes
5. Generated Rust bindings updated
6. Test fixtures: valid IR with state machine + transitions, invalid IR with unreachable states

---

## Key Schema Design Notes

### state.fbs

- **StateMachine:** Named machine with states (enum), transitions (event → target + guard + effect), initial state. The validator will check reachability and deadlock-freedom (Sprint 06).
- **DataNode:** Declares data source (provider, endpoint, query), cache strategy, loading/error state refs. References from `DATA_INTEGRATION.md`.
- **ComputeNode:** Pure function (inputs → output). Referentially transparent. The compiler can memoize or pre-compute.
- **EffectNode:** Side effect triggered by state transition (not by state). Analytics, API mutations, haptic feedback.
- **ContextNode:** Shared state scoped to a subtree. Typed read/write boundaries.

### motion.fbs

- **Transition:** Animates property changes between states. References the `Easing` type from `types.fbs` (including spring). Must have a `ReducedMotion` reference.
- **Sequence:** Choreographed timeline. Array of transitions with stagger offsets, parallel groups.
- **GestureHandler:** Maps input to state transitions or continuous property updates. Must declare keyboard equivalent.
- **ScrollBinding:** Binds node properties to scroll position. Range, property, mapping curve.
- **PhysicsBody:** Mass, velocity, friction, restitution, constraints. `interruptible: bool` flag determines compile strategy.

### navigation.fbs

- **RouteMap:** State machine where states are routes. Contains RouteEntries.
- **RouteEntry:** Path pattern, ViewRoot reference, guard conditions, sitemap metadata.
- **RouteTransition:** Visual transition between routes (View Transitions API). Shared element names.
- **RouteGuard:** Auth requirement, role check, redirect target.

---

## Open Questions for This Sprint

1. How do DataNode and ActionNode relate? Both involve network. Should they share a `DataSource` table?
2. Should StateMachine states be an enum (static, known at compile time) or a string set (dynamic)?
3. How are GestureHandler keyboard equivalents expressed? A parallel `KeyboardBinding` table?
4. Should ReducedMotion be inline on Transition (simpler) or a separate referenced node (more flexible)?

---

## Acceptance Criteria

- [ ] All three `.fbs` files compile with `flatc`
- [ ] Generated Rust bindings compile
- [ ] Test fixture: valid IR with a button StateMachine (idle → hover → pressed → idle)
- [ ] Test fixture: valid IR with entrance Sequence (3 staggered elements)
- [ ] Test fixture: valid IR with RouteMap (2 routes with transition)
- [ ] Test fixture: invalid IR with unreachable state in StateMachine
- [ ] JSON round-trip works for all new node types
- [ ] `cargo test --workspace` passes
