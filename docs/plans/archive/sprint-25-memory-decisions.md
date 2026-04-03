# Sprint 25 — Memory & Decisions

**Status:** Planned
**Goal:** Implement the `.voce/` persistence layer: brief enforcement (check requests against the north star), decision logging with conflict detection, session recovery, and drift detection. After this sprint, the AI remembers project context across sessions and prevents contradictory design decisions.
**Depends on:** Sprint 24 (style packs, brief creation fully operational)

---

## Deliverables

1. `.voce/` directory structure: `brief.yaml`, `decisions/`, `sessions/`, `config.toml`
2. Brief enforcement: every generation request checked against the saved brief
3. Decision log: timestamped YAML entries recording what was decided and why
4. Conflict detection: new decisions compared against existing ones, warnings on contradiction
5. Session recovery: resume interrupted work with full context restoration
6. Drift detection: compare current IR against brief, flag divergence
7. `voce status` command showing brief summary, decision count, drift score

---

## Tasks

### 1. Directory Manager (`src/memory/directory.ts`)

Initialize and manage `.voce/` structure. Auto-create on first `voce design` run. Functions: `ensureVoceDir()`, `readBrief()`, `writeBrief()`, `listDecisions()`, `appendDecision()`. Respect `.gitignore` — add `.voce/sessions/` and `.voce/cache/` but track `brief.yaml` and `decisions/`.

### 2. Brief Persistence (`src/memory/brief.ts`)

Save `DiscoveryBrief` as `brief.yaml` after conversation completes. Load on subsequent sessions as project context. Brief includes: purpose, audience, sections, tone, style pack, constraints, creation date, last modified. Validate brief schema on load — migrate if format changes between versions.

### 3. Brief Enforcement (`src/memory/enforcement.ts`)

Before generation, compare the user's request against the saved brief. Flag conflicts: "Your brief says 'minimal and clean' but you're requesting 'add animated particle background'." Severity levels: warning (mild tension), block (direct contradiction of a core constraint). User can override with `--force` or update the brief.

### 4. Decision Log (`src/memory/decisions.ts`)

Each significant choice becomes a decision entry in `decisions/<timestamp>-<slug>.yaml`: what was decided, alternatives considered, rationale, which agent made it, brief field it relates to. Examples: "chose 3-column grid over 2-column for features section because brief specifies 6+ features." Queryable: `voce decisions list`, `voce decisions search <term>`.

### 5. Conflict Detection (`src/memory/conflicts.ts`)

When appending a new decision, scan existing decisions for contradictions. Use the Claude API to compare: "Does decision A conflict with decision B?" Flag conflicts with a confidence score. Example: "Previous decision chose 'minimal animations' but new decision adds 'parallax scrolling' — conflict (confidence: 0.85)." User resolves by picking one.

### 6. Drift Detection (`src/memory/drift.ts`)

Compare current IR output against the brief's intent. Score 0-100: how well does the generated output match what was planned? Check: are all planned sections present? Does the style match the chosen pack? Are there elements not in the brief? Report drift on each generation. Alert if drift > 30%.

### 7. Status Command (`src/cli/status-command.ts`)

`voce status` prints: project brief summary (one-liner), style pack, section count, decision count, last session date, drift score (if IR exists). Color-coded: green (on track), yellow (minor drift), red (significant drift or stale brief). Quick at-a-glance project health.

---

## Files to Create

- `packages/ai-bridge/src/memory/directory.ts`
- `packages/ai-bridge/src/memory/brief.ts`
- `packages/ai-bridge/src/memory/enforcement.ts`
- `packages/ai-bridge/src/memory/decisions.ts`
- `packages/ai-bridge/src/memory/conflicts.ts`
- `packages/ai-bridge/src/memory/drift.ts`
- `packages/ai-bridge/src/cli/status-command.ts`
- `tests/ai-bridge/memory/enforcement.test.ts`
- `tests/ai-bridge/memory/conflicts.test.ts`
- `tests/ai-bridge/memory/drift.test.ts`

---

## Acceptance Criteria

- [ ] `voce design` creates `.voce/` directory with `brief.yaml` on first run
- [ ] Subsequent sessions load the saved brief as project context
- [ ] Contradictory requests trigger a warning with explanation (not silent override)
- [ ] `--force` flag bypasses brief enforcement with a logged override
- [ ] Decision log entries are created for each significant agent decision
- [ ] Conflict detection catches: "minimal" brief + "add heavy animations" request
- [ ] `voce decisions list` shows all decisions with timestamps
- [ ] `voce status` displays brief summary, decision count, and drift score
- [ ] Drift score correctly identifies missing planned sections as divergence
- [ ] Session recovery restores conversation state, brief, and decision context
- [ ] `.voce/sessions/` and `.voce/cache/` are in `.gitignore` template
