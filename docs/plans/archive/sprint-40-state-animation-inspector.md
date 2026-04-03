# Sprint 40 — State & Animation Inspector

**Status:** Planned
**Phase:** 5 (Visual Inspector & Tooling)
**Depends on:** S39 (inspector core)

---

## Goal

Add state machine visualization and animation timeline tools to the inspector, enabling live debugging of state transitions and motion.

---

## Deliverables

- State machine visualizer: node-graph view of states and transitions
- Live state indicator: current state highlighted, transition history log
- Guard condition display: why a transition was blocked
- Effect tracking: which effects fired on each transition
- Animation timeline: horizontal scrubable timeline for all active animations
- Timeline controls: pause, play, step forward/backward, speed adjustment
- Per-animation breakdown: easing curve preview, duration, delay, target properties
- Data flow monitor: live DataNode values and binding updates
- Keyboard shortcuts for timeline control (space=pause, arrow keys=step)

---

## Acceptance Criteria

- [ ] State machine visualizer shows all states and transitions as a graph
- [ ] Current state is visually highlighted in real-time
- [ ] Transition history shows last 50 transitions with timestamps
- [ ] Animation timeline supports pause, scrub, and frame-by-frame step
- [ ] Speed adjustment works (0.25x, 0.5x, 1x, 2x)
- [ ] Data flow monitor shows live DataNode value changes
- [ ] All timeline controls work via keyboard shortcuts
