# Sprint 66 — Standalone Conversational REPL

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Make `voce-chat` (the standalone CLI) feel like a real conversational tool — comparable in quality to running `claude` in a terminal — for users who don't already have an MCP-compatible client. Tool-use loop, multi-line input, persistent project memory via `.voce/`, broader slash command surface, and conversational pillars enforced via system prompt and tool semantics.

**Depends on:** S65 (`.voce/` memory contract + tool semantics shaped there), `@voce-ir/cli-chat` skeleton (S30), `@voce-ir/ai-bridge` (S21–S26)

---

## Motivation

`voce-chat` exists (354 lines, `packages/cli-chat/src/index.ts`) but is shallow: streams Claude responses, parses ` ```json ` fences out of the output, exposes 7 fixed slash commands, loses everything on exit. The model can't run validate/compile itself — the user must type slash commands to discover failures. This is a "chatbot wrapping a CLI" experience, not a "conversational tool" experience.

This sprint lifts `voce-chat` to roughly Claude Code's level of conversational quality, scoped to Voce's domain. After S66, a user can run `voce-chat` in a fresh terminal, describe a UI in plain English, watch the model ask clarifying questions, propose IR, validate it itself, fix errors itself, and ship — with the full conversation persisted to `.voce/session.jsonl` for resumption.

---

## Deliverables

### 1. Tool-use loop replacing text streaming

Replace `messages.stream` with a tool-use loop that hands the model the same tools the MCP server exposes (S65). The model can:
- Call `voce_validate` and react to diagnostics
- Call `voce_compile` and report sizes/errors
- Call `voce_inspect` for structural summaries
- Call `voce_check_drift` before proposing changes
- Call `voce_decisions_log` when the user makes a binding choice
- Call `voce_brief_get`/`voce_brief_set` to sync the project North Star

Implementation: import `@voce-ir/ai-bridge`'s tool-use orchestrator (or build a thin one in cli-chat) and feed the same tool definitions. Both `cli-chat` and `mcp-server` then expose the same Voce semantics; only the transport differs.

### 2. Persistent `.voce/` memory

On startup:
- Read `.voce/brief.md` (if present) into the system context
- Read the last N decisions from `.voce/decisions.jsonl` (last 20 by default)
- If `--resume <session-id>` was passed, load `.voce/session.jsonl` for that session

On every turn:
- Append the user message and the assistant response to `.voce/session.jsonl` (atomic write, one JSON object per line)
- Append any tool calls + results to the session log

On `/decision <text>`:
- Append a structured decision entry to `.voce/decisions.jsonl`

Crash recovery: a `voce-chat --resume` with no session ID picks the most recent unfinished session.

### 3. Conversational system prompt

Replace the existing 22-line system prompt with one that explicitly encodes the six pillars from `docs/research/CONVERSATIONAL_DESIGN.md`:

1. One question at a time during discovery
2. Build user/project profiles from prior turns
3. Full-stack feature completeness (loading, error, empty, validation, a11y states)
4. Push back concisely on anti-patterns
5. Share expertise proactively, one insight at a time
6. Do not generate IR until the readiness score is ≥70

Plus: explicit instruction to consult `voce_check_drift` before proposing IR changes that touch decisions in the log.

### 4. Multi-line input + Ctrl+C interrupt

- Replace single-line readline with a multi-line editor: paste a block of text, press Enter twice to submit (or use a `\` line-continuation pattern)
- Ctrl+C cancels in-flight model responses cleanly (currently kills the process)
- Optional: integrate `prompts` or `ink` for richer terminal UI; out of scope unless the simpler readline approach is too limiting

### 5. Slash command surface

Existing 7 commands stay. Add:
- `/brief` — show/edit the project brief inline
- `/decisions [n]` — list last N decisions, or filter
- `/decision <text>` — log a new decision
- `/diff` — show the IR diff between this turn and the previous proposal
- `/undo` — revert to the previous IR
- `/explain` — model explains why the current IR is shaped the way it is
- `/model <name>` — switch model (sonnet → opus or vice versa) mid-session
- `/cost` — show cumulative token usage and estimated cost
- `/ir` — paste an existing IR to load into the session
- `/load <file>` — load IR from disk
- `/resume [session-id]` — resume a prior session (also exposed as a launch flag)
- `/preview` — open the compiled HTML in the system browser

Slash command framework: each command is a small handler module under `packages/cli-chat/src/commands/`. New commands drop in as files.

### 6. Generation Readiness Score visible to the user

When the model believes it's ready to propose IR, show the user:

```
Generation Readiness: 82/100
  ✓ Domain understood (e-commerce checkout)
  ✓ Visual style established (minimal, dark)
  ✓ Required interactions captured
  ✗ Empty states not specified — assuming "show illustration + CTA"
  ✗ Tablet breakpoint not specified — assuming 768px → single column

Proceed? [Y/n/q for question]
```

Pulled from S65's `voce_generation_readiness` tool. Lets the user override missing fields or accept defaults.

### 7. Push-back UX

When the model detects a decision conflict (via `voce_check_drift`), surface it as a quoted concern, not a refusal:

```
voce  >  Hold on — decision #4 from 2026-04-12 says "all forms must use 
         optimistic updates." This new IR uses synchronous submission. 
         Was this intentional?
         
         [r] revise the IR  [s] supersede decision #4  [c] continue anyway
```

Three-key resolution. Logs the chosen path back to `.voce/decisions.jsonl`.

### 8. Tests

- Unit: each slash command handler, system-prompt assembly, `.voce/` read/write
- Integration: full session through tool-use loop, validate, fix, finalize, persist, resume
- Snapshot: terminal output for canonical conversation flows (use `ansi-escapes` or similar to strip color before snapshotting)

---

## Acceptance Criteria

- [ ] cli-chat uses tool-use loop, not raw text streaming, for any prompt that produces IR
- [ ] `.voce/brief.md`, `.voce/decisions.jsonl`, `.voce/session.jsonl` are read at startup and written on every turn
- [ ] `voce-chat --resume` recovers the most recent session
- [ ] System prompt encodes all 6 conversational pillars, ≤ 800 tokens
- [ ] Multi-line input works; Ctrl+C interrupts in-flight responses without killing the process
- [ ] All listed slash commands implemented and tested
- [ ] Generation Readiness Score is shown before every IR proposal
- [ ] Drift detection surfaces decision conflicts with [r/s/c] resolution
- [ ] Vitest suite covers every command + the full session flow
- [ ] `voce-chat` startup time under 500ms (cold) on a developer laptop
- [ ] No telemetry phones home without explicit user opt-in (if any added at all)

---

## Risks

1. **Tool-use loop complexity.** The model can call tools in unexpected orders (validate before generate, compile something invalid, etc.). The orchestrator needs a small state machine that gates which tools are available at each phase. Same logic the MCP server enforces (S65); reuse.
2. **Multi-line readline is finicky on Windows.** Test on Windows Terminal early; if it's painful, ship the simpler single-line input for v1 with a "press / for command" affordance.
3. **`.voce/` ownership.** When two `voce-chat` instances run on the same project (rare but possible), session.jsonl writes can race. Use atomic writes + per-session filenames (`.voce/session-{id}.jsonl`) and leave merging for future work.
4. **Token cost from rich system prompt + tool definitions.** Every turn carries the full conversational context. Use prompt caching (Anthropic SDK supports it) to keep this affordable; document cache-hit-rate expectations in the build journal.

---

## Out of Scope

- A TUI / Ink-based UI overhaul — defer
- Voice input — exists in `ai-bridge/src/voice/` as a separate effort, not wired here
- Real-time collaboration / multi-user sessions
- Web UI for the same flow — that's a different product surface
- Plugin system for custom slash commands — defer to future
