# Sprint 26 — Incremental Generation

**Status:** Planned
**Goal:** Support patch/delta IR updates so users can modify one section without regenerating the entire page. Add session context so the AI knows the current IR state, and hierarchical generation for complex multi-section UIs. After this sprint, iteration is fast and surgical rather than full-page regeneration.
**Depends on:** Sprint 25 (memory layer, brief/decision persistence)

---

## Deliverables

1. IR diff format: JSON Patch (RFC 6902) operations scoped to Voce IR nodes
2. Patch Generator Agent: takes current IR + change request, emits minimal patch
3. Patch applicator: applies JSON Patch to existing IR, re-validates
4. Session context: feeds current IR structure summary to all agents
5. Hierarchical generation: generate page skeleton first, then fill sections in parallel
6. `voce edit "<change>"` CLI command for incremental updates
7. Undo support: revert last patch via stored patch history

---

## Tasks

### 1. IR Diff Format (`src/incremental/diff.ts`)

Define the patch format based on JSON Patch (RFC 6902): add, remove, replace, move operations targeting IR node paths. Example: `{ op: "replace", path: "/children/2/props/text", value: "New headline" }`. Validate patches against IR schema before applying. Include metadata: change description, timestamp, agent that produced it.

### 2. Patch Generator Agent (`src/agents/patch-agent.ts`)

New agent type: input is current IR + user's change request + brief context. Output is a minimal JSON Patch, not a full IR. System prompt emphasizes: touch only what the user asked to change, preserve everything else, maintain structural validity. Much cheaper than full regeneration (target: under 5K tokens per edit).

### 3. Patch Applicator (`src/incremental/applicator.ts`)

Apply JSON Patch operations to an existing IR JSON file. Validate result with `voce validate` after applying. If validation fails, pass to Repair Agent (S22) for fix-up. Support dry-run mode: show what would change without modifying the file. Atomic: either all operations succeed or none apply.

### 4. Session Context Builder (`src/context/session-context.ts`)

Summarize the current IR state for agent consumption: list of sections (type + brief description), node count, style pack, key design decisions. Keep under 2K tokens. Updated after every generation or edit. Prevents the AI from losing track of what already exists.

### 5. Hierarchical Generation (`src/generator/hierarchical.ts`)

For complex pages (>8 sections), generate in two passes. Pass 1: skeleton — ViewRoot, top-level Container structure, placeholder nodes. Pass 2: fill each section in parallel (one Generator Agent call per section, all running concurrently). Merge results into final IR. 3x faster for complex pages.

### 6. Edit CLI Command (`src/cli/edit-command.ts`)

`voce edit "make the hero headline bigger and change CTA to green"` — loads current IR from `.voce/output/`, sends to Patch Generator Agent, applies patch, re-validates, re-compiles. Shows diff summary: "Changed 2 nodes, added 0, removed 0." Supports `--dry-run` to preview changes.

### 7. Patch History & Undo (`src/incremental/history.ts`)

Store patches in `.voce/history/` as numbered JSON files. `voce undo` reverts the last patch by applying its inverse. `voce history` lists recent changes with descriptions. Keep last 50 patches. Enable time-travel: `voce history checkout 5` restores state after patch 5.

---

## Files to Create

- `packages/ai-bridge/src/incremental/diff.ts`
- `packages/ai-bridge/src/incremental/applicator.ts`
- `packages/ai-bridge/src/incremental/history.ts`
- `packages/ai-bridge/src/agents/patch-agent.ts`
- `packages/ai-bridge/src/context/session-context.ts`
- `packages/ai-bridge/src/generator/hierarchical.ts`
- `packages/ai-bridge/src/cli/edit-command.ts`
- `tests/ai-bridge/incremental/applicator.test.ts`
- `tests/ai-bridge/incremental/history.test.ts`
- `tests/ai-bridge/agents/patch-agent.test.ts`

---

## Acceptance Criteria

- [ ] `voce edit "change headline to Welcome"` modifies only the headline node, nothing else
- [ ] Patch operations are valid JSON Patch (RFC 6902) format
- [ ] Applied patches pass `voce validate` — Repair Agent fixes issues if needed
- [ ] `--dry-run` shows planned changes without modifying files
- [ ] Session context accurately summarizes current IR (verified manually on 3 test cases)
- [ ] Hierarchical generation produces identical output to single-pass for a 5-section page
- [ ] Hierarchical generation is measurably faster (>2x) for 10+ section pages
- [ ] `voce undo` correctly reverts the last change
- [ ] `voce history` lists all patches with descriptions and timestamps
- [ ] Patch generation uses under 5K tokens for a single-section edit
- [ ] Edit round-trip (edit + validate + compile) completes in under 10 seconds
