---
name: Memory and Decision Tracking as Core System
description: Memory is non-negotiable infrastructure — brief enforcement, decision logging with conflict detection, session persistence, drift prevention. Not a feature, it's the connective tissue.
type: feedback
---

Memory is what turns a conversation into a collaboration. Three layers:

1. **Project Brief (north star)** — what we're building, why, for whom, success criteria, non-negotiables, feature list with status. Changes require explicit confirmation. Requests checked against it — drift is caught.

2. **Decision Log** — every architectural/design/feature decision with rationale, alternatives considered, implications, and status (active/superseded). New decisions checked for conflicts with existing ones. Temporal consistency enforced (day 1 vs day 100).

3. **Session Memory** — conversation history persisted on every turn (not at session end). Interrupted sessions detected and resumed. User preferences learned over time. Open items carried forward.

**Why:** Marc has observed that AI projects drift from their original vision over time. Each individual request is reasonable but cumulative effect is scope creep. The brief enforcement prevents this. Also: session interruptions should never lose work.

**How to apply:**
- `.voce/` directory stores all memory (brief.yaml, decisions/, sessions/, memory/, snapshots/)
- Memory lives in the project (git-tracked), not in the AI's conversation history
- Works across AI providers (exposed via MCP resources)
- `voce brief`, `voce decisions`, `voce features`, `voce memory` CLI commands
- Phase 1: directory structure, brief/decisions format, session persistence
- Phase 3: brief enforcement, conflict detection, drift detection, user preference learning
