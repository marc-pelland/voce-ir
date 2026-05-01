# Sprint 65 — MCP Server Polish: Conversational Tools + .voce/ Memory

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Make `@voce-ir/mcp-server` a first-class Voce experience for *any* MCP-compatible client (Claude Code, Cursor, Cline, Claude Desktop, Continue.dev). Tool descriptions encode the conversational guardrails from `docs/research/CONVERSATIONAL_DESIGN.md`. The `.voce/` memory layout from `docs/research/MEMORY_AND_DECISIONS.md` becomes a real, readable, writable surface. Generation runs as a multi-step workflow that asks, validates, and refines instead of one-shot blind output.

**Depends on:** existing `@voce-ir/mcp-server` skeleton (S27), `@voce-ir/ai-bridge` (S21–S26), validator (S07), compiler-dom (S20)

---

## Motivation

`mcp-server` ships 6 tools today (`voce_validate`, `voce_compile`, `voce_inspect`, `voce_schema`, `voce_examples`, `voce_generate`). Every tool description is one sentence; none of them encode the project's stated conversational pillars; `voce_generate` is a one-shot wrapper. The MCP integration works, but it doesn't *behave* like Voce — it behaves like a thin RPC over the CLI.

The fix: bring the conversational pillars and the `.voce/` memory contract into the MCP surface itself. Every MCP client running through this server inherits the right behavior automatically — no client-side prompting required. This is the highest-leverage path to "the system feels like Voce" because every developer using an AI coding assistant already has an MCP client.

---

## Deliverables

### 1. `.voce/` canonical layout + persistence

The repo's `.voce/` directory becomes the single source of truth. Layout (matches `MEMORY_AND_DECISIONS.md`):

```
.voce/
  brief.md              The project's North Star. Markdown, hand-edited or AI-edited via tools.
  decisions.jsonl       Append-only log. Each line: {id, timestamp, summary, rationale, supersedes?, conflicts_with?}.
  session.jsonl         Per-session conversation log for crash recovery. Append-only; rotated per session.
  user-profile.md       (Optional) User preferences and recurring patterns the AI has learned.
  drift-warnings.jsonl  (Generated) Each entry: {timestamp, decision_id, drift_description, resolution}.
```

A new `packages/voce-memory` Rust crate (or TypeScript module under ai-bridge) owns read/write semantics: atomic writes, append-only enforcement on the JSONL files, schema validation on each entry.

### 2. Tool description rewrite — encode pillars in semantics

Every existing tool description grows to 3-6 sentences and includes the conversational stance the calling agent should take. Example:

**Before:**
```
voce_validate: Validate a Voce IR JSON file against all quality rules
```

**After:**
```
voce_validate: Validate a Voce IR JSON document. Returns per-pass diagnostics
with severity, code, node path, and (when available) actionable hints. Use this
BEFORE generating code from the IR — Voce treats accessibility, security, and
SEO as compile errors, not warnings. If validation fails, address each
diagnostic by category (a11y first, then security, then SEO, then forms) before
re-validating. Do not present an invalid IR to the user as "done"; that
violates the full-stack-completeness pillar.
```

All 6 existing tools get this treatment. Length budget: ~200 chars per tool description average.

### 3. Multi-step `voce_generate` workflow

Replace the one-shot `voce_generate` with a workflow. The calling agent invokes phases explicitly:

- `voce_generate_start({ user_intent: string }) → { session_id, next_question: string | null, ready: bool }`
- `voce_generate_answer({ session_id, answer: string }) → { next_question, ready, missing_context[] }`
- `voce_generate_propose({ session_id }) → { ir_json, readiness_score, completeness_check }`
- `voce_generate_refine({ session_id, feedback: string }) → { ir_json, diff }`
- `voce_generate_finalize({ session_id }) → { ir_json, validation, html, deployment_hints }`

Each phase enforces the "one question at a time" rule (`next_question` is always one or null). The agent calling these tools cannot skip discovery — `propose` returns an error if `ready: false`.

### 4. Memory tools

- `voce_brief_get() → { brief_md: string, last_modified }`
- `voce_brief_set({ brief_md })` — confirms with the agent before persisting
- `voce_decisions_list({ since?: timestamp }) → { decisions: Decision[] }`
- `voce_decisions_log({ summary, rationale, supersedes?, conflicts_with? }) → { id }`
- `voce_session_resume({ session_id? }) → { messages, current_ir, last_decision_id }`
- `voce_check_drift({ proposed_ir }) → { drift: DriftReport[], decisions_referenced: string[] }`

`voce_check_drift` is the keystone — every `propose` and `refine` step calls it implicitly so the model sees decision conflicts before they land in user-facing output.

### 5. Quality gate tools

From CONVERSATIONAL_DESIGN §3:

- `voce_generation_readiness({ session_id }) → { score: 0-100, missing: string[], blocking: string[] }`
- `voce_feature_completeness({ ir_json }) → { complete: bool, missing_pillars: ("a11y"|"error-states"|"loading-states"|"empty-states"|"validation"|"i18n"|"responsive")[] }`

These are advisory in tool output, but `voce_generate_propose` blocks on `score < 70` and `voce_generate_finalize` blocks on `complete: false`.

### 6. MCP resources wired to real `.voce/` store

Currently `voce://brief` and `voce://status` return placeholder strings. After this sprint:

- `voce://brief` — live read of `.voce/brief.md`
- `voce://decisions` — live read of `.voce/decisions.jsonl`, returns latest 50 by default with `?since=` filter
- `voce://drift-warnings` — live read of `.voce/drift-warnings.jsonl`
- `voce://schema` — full IR JSON Schema for the calling LLM to consume directly (already partially implemented as a tool; promote to a resource)

### 7. Tests + integration verification

- Vitest tests for each tool: input validation, error cases, success path
- Integration test using the `@modelcontextprotocol/sdk` test client: full discovery → propose → refine → finalize loop
- A documented walkthrough in `docs/mcp-walkthrough.md`: how to wire `voce-mcp` into Claude Code (`.claude/settings.json` MCP config), Cursor, Cline. Concrete config blocks.

---

## Acceptance Criteria

- [ ] `.voce/` layout documented and a `packages/voce-memory` module owns read/write
- [ ] All 6 existing tools have rewritten descriptions encoding conversational pillars
- [ ] `voce_generate` replaced by 5-phase workflow; calling agent cannot skip phases
- [ ] 6 memory tools and 2 quality-gate tools present and tested
- [ ] All MCP resources read from the real `.voce/` store
- [ ] `voce-mcp` can be wired to Claude Code, Cursor, and Cline using the documented `mcp.json`/`settings.json` blocks
- [ ] Vitest suite covers every tool and the integration loop
- [ ] Tool description length budget held (≤ 1 KB total across all descriptions for token efficiency)
- [ ] No regression in the 6 original tools' input contracts (additive only — new tools, expanded descriptions, but old call sites still work)

---

## Risks

1. **Memory contract churn.** `.voce/` schema choices made now will be hard to change later — every project repo will have a `.voce/` directory in its git history. Pin the schema in `packages/voce-memory/SCHEMA.md` and treat any field changes as breaking releases.
2. **MCP token budget.** Long tool descriptions cost tokens on every model call. The 1 KB total budget is tight; tool descriptions need to be dense, not flowery.
3. **Drift detection false positives.** Naïvely flagging every IR change as "drifts from decision #4" will be noisy. Drift detection should require semantic conflict (e.g., decision: "always use server-side rendering"; new IR: ContentSlot with cache: Dynamic) — not surface-level diffs. Consider rule-based heuristics for v1; ML-based later.
4. **Multi-step workflow vs. one-shot habits.** Calling agents (Claude Code etc.) like to one-shot complex tools. Enforcing the phased workflow may surprise them. Mitigation: clear errors with actionable next-step messages.

---

## Out of Scope

- Standalone `voce-chat` REPL polish — that's S66
- Validator diagnostic improvements — that's S67
- New IR node types
- Multi-user / multi-project memory (`.voce/` is single-project for now)
- Sync between `.voce/` and external systems (Linear, Notion, etc.)
