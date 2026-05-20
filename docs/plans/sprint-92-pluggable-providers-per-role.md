# Sprint 92 — Pluggable Providers & Per-Role Model Strategy

**Phase:** 7 — Production Readiness → Ecosystem
**Status:** Planned
**Goal:** Let users assign **any AI provider/model to each role** in Voce's multi-agent pipeline (Discovery, Design, Generator, Repair, plus extensible new roles like Reviewer, Tester, Documenter), with first-class API-key/permission management and user-defined custom roles. Today the orchestrator (S22) is hard-wired to Claude; this sprint generalizes it without breaking the conversational pillars.

**Depends on:** S22 (multi-agent orchestrator, shipped), **S73 (orchestrator hardening — promote to a full plan and ship first)**, S65 (`.voce/` storage), S79 A1 (capability surface — providers register through it). Independent of S60/S68/S82.

---

## Motivation

[Kilo Code](https://kilo.ai/) (recent VS Code agent) lets a developer **assign a different AI model to each "mode"** — Code / Architect / Debug / Ask / Custom — backed by [500+ models via the Kilo Gateway](https://blog.kilo.ai/p/kilo-code-4191-orchestrator-mode). The orchestrator delegates each subtask to whichever mode (and therefore model) fits best. The user reaction in our session was strong and immediate: this is the right shape for AI-native authoring.

Voce already has the *internal* architecture for this (`packages/ai-bridge/src/agents/`: `discovery-agent`, `design-agent`, `generator-agent`, `repair-agent`, plus `orchestrator`). What it lacks is **provider pluralism** and **user control**:

- Every agent currently calls `ClaudeClient`. There is no provider abstraction.
- No per-role model configuration.
- No API-key management UI / store.
- No way for users to define custom roles (e.g. "Reviewer", "Tester", "Documenter").
- No subagent-delegation surface (a primary role spawning a subordinate one).

**Why this is a particularly clean fit for Voce specifically** — and a differentiator over Kilo / similar tools:

1. **Provider-agnostic by design.** Voce's IR is a single JSON shape; every provider emits the same artifact. There is no "Claude style" vs "GPT style" IR. A Kilo-style per-role assignment in a source-code generator forces the user to reconcile style drift; in Voce, the output is invariant.
2. **The agent contract (S79) already standardizes the interface.** Providers consume `voce_skills` / `voce_graph` / `voce_doctor` / diagnostic codes — exactly what they need to participate, with no provider-specific glue.
3. **Three pillars are non-negotiable.** Pluggable providers must not become a backdoor for skipping a11y/security/SEO — the orchestrator's quality gates stay, irrespective of which provider runs a role. This is what stops this turning into vibe-coding.

S93 (sibling, follow-on) adds **intelligent recommendations** on top of the strategy this sprint exposes.

---

## Deliverables

### D1. Provider abstraction (`packages/ai-bridge/src/providers/`)

A typed `Provider` interface every backend implements:

```ts
interface Provider {
  id: string;                       // "anthropic", "openai", "google", "kilo-gateway", ...
  displayName: string;
  models(): ModelInfo[];            // capabilities, context window, cost tier
  complete(req: CompletionRequest): Promise<CompletionResponse>;
  stream?(req: CompletionRequest): AsyncIterable<CompletionChunk>;
  toolUse?: ToolUseSupport;         // "native" | "via-prompting" | "unsupported"
  structuredOutput?: "json-schema" | "function-call" | "regex" | "none";
}
```

Built-in implementations: **Anthropic** (extracted from today's `ClaudeClient`), **OpenAI**, **Google (Gemini)**, **Local (Ollama-compatible)**. **Kilo Gateway** as a meta-provider fronting many. Third-party providers land via a documented plugin contract — no source patches required.

### D2. Per-role model strategy (`.voce/agents.toml`)

User-editable, single source of truth, lives alongside `.voce/brief.md`:

```toml
[provider.anthropic]
api_key_env = "ANTHROPIC_API_KEY"     # or api_key_file = "..."

[provider.openai]
api_key_env = "OPENAI_API_KEY"

[role.discovery]
provider = "anthropic"
model    = "claude-opus-4-7"           # reasoning-heavy → strongest model
fallback = ["claude-sonnet-4-6"]
max_tokens = 16000

[role.design]
provider = "anthropic"
model    = "claude-sonnet-4-6"

[role.generator]
provider = "openai"
model    = "gpt-5"
structured_output = "json-schema"

[role.repair]
provider = "google"
model    = "gemini-3.5-pro"
fallback = ["claude-haiku-4-5"]
```

Schema-locked (extends S79 A4 with `agents.schema.toml`). Loaded by the orchestrator; absence falls back to today's all-Claude defaults so existing users see no change. Validated by `voce doctor` (new check IDs `DOC-AGENTS-NNN`).

### D3. API-key & permission store

- Keys resolved in order: `.voce/agents.toml api_key_env` → `.voce/agents.toml api_key_file` → environment → OS keychain (macOS Keychain / Windows Credential Manager / `pass`).
- **Never** written to logs, transcripts, session JSONL, or telemetry.
- Permission scopes per provider: `discovery_only`, `generation_only`, `read_only`, `full`. The orchestrator enforces — a `read_only` provider is silently filtered out of write-capable roles with a doctor warning.
- `voce auth login <provider>` / `voce auth logout <provider>` / `voce auth status` subcommands. Equivalent MCP tools.

### D4. Custom roles & subagent delegation

Users can declare additional roles in `.voce/agents.toml`:

```toml
[role.reviewer]
provider = "anthropic"
model    = "claude-opus-4-7"
system_prompt_file = ".voce/agent-prompts/reviewer.md"
trigger = "pre-finalize"

[role.documenter]
provider = "openai"
model    = "gpt-5"
trigger = "post-finalize"
```

Triggers: `pre-discovery`, `post-design`, `pre-generate`, `post-generate`, `pre-finalize`, `post-finalize`. The orchestrator runs custom roles at the named hooks; their output is appended to the session JSONL but cannot override pillar gates.

**Subagent delegation:** any role may spawn a one-shot subagent for a bounded subtask (cf. Kilo's Orchestrator-mode delegation, [now folded into primary modes](https://kilo.ai/docs/code-with-ai/agents/orchestrator-mode)). The subagent inherits the parent role's provider unless the call specifies otherwise. Subagent calls show up in `voce_graph`-style structured form so the user can audit who decided what.

### D5. Pillar gates are not negotiable

Three guarantees the strategy can never disable:

1. **Discovery readiness gate** survives — every provider's Discovery output is scored the same way (S22's readiness score). No provider can claim "I'm ready" without it.
2. **Validator runs.** No provider's Generator output is finalized without `voce validate` passing — same a11y/security/SEO compile-error stance as today.
3. **Drift detection on Decisions.** Same `.voce/` decision-log check runs regardless of provider.

These three live in the orchestrator, not the providers, so swapping providers can never weaken them. Tested in CI per provider.

### D6. CLI + MCP surface

- `voce agents list` / `voce agents set <role> <provider:model>` / `voce agents test` (dry-run a roundtrip per role to verify keys+models work).
- MCP tools: `voce_agents_get`, `voce_agents_set` — same store, same validation.
- `voce skills --json` (S79) gains a `providers[]` and `roles[]` block describing what is configured and what is available. Agents discover the strategy through the existing contract envelope; no new surface needed.

---

## Acceptance Criteria

- [ ] `Provider` interface + at least 3 first-party implementations (Anthropic, OpenAI, Google) + Kilo-Gateway meta-provider
- [ ] `.voce/agents.toml` schema published under `docs/schema/contract/v1/agents.schema.toml`; drift gate per S79 A4
- [ ] Existing default (all-Claude) survives absence of `agents.toml` — zero migration burden for current users
- [ ] API keys resolved via env → file → OS keychain; never written to any logged surface (a redaction test asserts this)
- [ ] Custom roles + per-hook triggers working end-to-end with a worked example fixture (Reviewer + Documenter)
- [ ] Subagent delegation visible in session JSONL with parent/child role attribution
- [ ] Per-provider conformance suite: pillar gates pass under each provider on a representative IR; CI matrix runs at least Anthropic, OpenAI, Google
- [ ] `voce agents test` dry-runs every configured role and reports `pass | warn | fail` with stable IDs
- [ ] MCP parity: new tools added to `@voce-ir/mcp-server`; tool count surfaces in `voce_skills`

## Out of Scope

- Hosting a model gateway (consume Kilo Gateway / OpenRouter / etc.; don't build one)
- Model fine-tuning or RLHF — out of scope forever; Voce trains nothing
- A web UI for key management — CLI + config-file first; UI is post-S60
- Pricing / cost optimization analytics (S78 telemetry territory — opt-in only)
- **Recommendations:** which model for which role is **S93**'s job, not this sprint's

## Risks

1. **Capability drift across providers.** Tool-use support, structured-output support, and JSON-mode reliability vary. Mitigation: the `Provider` interface declares capabilities; the orchestrator picks the strongest compatible path per provider, with a clear doctor warning when a role's chosen provider is structurally weaker (e.g., a provider without native tool use is downgraded to prompt-engineered JSON output).
2. **Cost surprise.** Per-role assignment makes runaway cost a real risk. Mitigation: per-role `max_tokens` + per-session budget cap (`max_session_cost_usd`) + an explicit doctor warning when budgets are unset.
3. **Conversational pillar erosion.** If users plug in a cheap model for Discovery to "skip the questions," authoring quality drops and so does Voce's positioning. Mitigation: Discovery readiness score is provider-agnostic and orchestrator-enforced; a Discovery role producing < threshold readiness is rejected the same way today's Claude Discovery is.
4. **Anti-vibe-coding stance.** This sprint must not become a "let any model write anything" surface. The pillar gates (D5) are the line; the spec is written so they can never be bypassed by provider choice.
5. **Plugin security.** Third-party provider plugins execute in-process. Mitigation: signed plugins for v1.x; sandboxing as a v2 concern.

## Relationship to S22 / S73 / S79 / S93

- **S22** built the role-based orchestrator (shipped). This sprint generalizes its single-provider assumption.
- **S73** (currently a one-liner — *promote to a full plan and ship before this sprint*) hardens the orchestrator's handoff state machine, error recovery, and per-role test coverage. Doing it first means S92 builds on a stable substrate, not a moving one.
- **S79** provides the contract surface providers consume (`skills`/`graph`/`doctor`); A4 schema discipline carries `agents.toml` validation.
- **S93** is the recommendation layer that sits on top of the strategy this sprint enables. Cleanly separable; ship S92 first.
