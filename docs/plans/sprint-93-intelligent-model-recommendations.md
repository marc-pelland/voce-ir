# Sprint 93 — Intelligent Model Recommendations

**Phase:** 7 — Production Readiness → Ecosystem
**Status:** Planned
**Goal:** On top of S92's pluggable per-role provider strategy, recommend **which model to assign to each role** for *this* project's shape — using transparent, explainable heuristics first, opt-in usage signals later. The user picks; Voce explains. No black-box "smart" picks.

**Depends on:** S92 (per-role assignment must exist before recommending anything to assign), S79 (capability surface — recommendations are part of the contract), `voce_graph` (the project-shape input), optionally S78 (opt-in telemetry, never required).

---

## Motivation

S92 lets users assign a model per role; without guidance, that's just configuration. Kilo's reaction-to-our-discussion ask was explicit: *"intelligent recommendations for which AI systems to use based on the specific needs of a project."* The risk in any such recommender is opacity — a recommendation users can't reason about turns into vendor lock-in dressed up as helpfulness.

Voce's stance: **recommendations are typed, reasoned, and overridable.** Each recommendation cites the project shape that drove it (from `voce_graph` + `voce skills`), the capability requirements of the role, and the trade-off (latency / cost / reasoning depth) that ranked the candidates. The user sees the *why* and can flip a flag.

---

## Deliverables

### D1. Project profile (`voce profile [--json]`)

Reflects what the project *needs* per role — derived from `voce_graph` + `voce skills`, no usage data required:

```json
{
  "contract_version": "1.0.0",
  "shape": {
    "node_count": 142,
    "state_machine_count": 3,
    "form_count": 2,
    "data_node_count": 5,
    "compile_targets_in_use": ["dom", "email"],
    "ir_complexity": "medium",
    "domain_hints": ["forms", "data-binding", "i18n"]
  },
  "role_demand": {
    "discovery":  { "reasoning": "high",   "context": "medium" },
    "design":    { "reasoning": "medium", "context": "medium" },
    "generator": { "reasoning": "medium", "context": "high", "structured_output": "required" },
    "repair":    { "reasoning": "high",   "context": "low",  "latency": "fast" }
  }
}
```

Pure function of the IR + contract — no telemetry, no network call.

### D2. Model catalog (`models.toml` — first-party seed + plugin-extensible)

For every known model: provider, context window, tool-use support, structured-output support, cost tier (S/M/L/XL), reasoning tier, latency tier, license/availability. Updated via PR (this is acceptable here — catalogs are a known-OK kind of hand-maintained list when they describe *external* state we don't generate, and a drift test checks that every catalogued model is actually reachable from at least one configured provider). Mirrors what Kilo Gateway exposes structurally without forcing users through it.

### D3. Recommendation engine (`voce recommend [role] [--json]`)

A *typed* ranker, not an LLM call. Each recommendation carries:

```json
{
  "contract_version": "1.0.0",
  "role": "generator",
  "recommendations": [
    {
      "rank": 1,
      "provider": "openai",
      "model": "gpt-5",
      "score": 0.92,
      "rationale": [
        "Role requires native structured_output: gpt-5 has json_schema mode",
        "IR complexity medium; gpt-5 context window 1M comfortably exceeds 64k typical",
        "Cost tier M aligns with project budget 'standard'"
      ],
      "trade_offs": { "latency": "fast", "cost_tier": "M", "reasoning": "high" }
    },
    {
      "rank": 2,
      "provider": "anthropic",
      "model": "claude-opus-4-7",
      "score": 0.89,
      "rationale": [
        "Strongest reasoning tier in catalog",
        "Native tool-use, structured output via prompted JSON",
        "Cost tier L — higher than #1 for this workload"
      ],
      "trade_offs": { "latency": "medium", "cost_tier": "L", "reasoning": "highest" }
    }
  ],
  "inputs_used": ["voce profile.shape", "voce profile.role_demand.generator", "models.toml"]
}
```

The ranker is deterministic and traceable. No machine learning, no opaque scoring. Bumps to ranking weights are version-controlled changes anyone can review.

### D4. `voce agents recommend --apply` flow

```
$ voce agents recommend
Suggested strategy for this project (IR complexity medium, 2 forms, i18n):
  discovery   anthropic/claude-opus-4-7   (reasoning-high)
  design      anthropic/claude-sonnet-4-6 (balanced, fast)
  generator   openai/gpt-5                (structured-output)
  repair      google/gemini-3.5-pro       (fast, low cost)

Apply these to .voce/agents.toml? [y/N/diff]
```

`--apply` writes; `diff` shows what would change vs current. Always prints the rationale per role; recommendations are never silent.

### D5. Opt-in feedback loop (gated; entirely optional)

If — and only if — S78 telemetry is opt-in and active, the ranker can incorporate **anonymized success rates per (role × provider × model)** observed across sessions:

- success = pillar gates passed first try
- failure = readiness rejection, validator failures, repair-loop divergence

Stored under `.voce/agent-stats.jsonl`, append-only, identical durability rules as the rest of `.voce/`. **Never** leaves the machine without explicit user opt-in. Never replaces the rationale-based recommendation — at most, surfaces "in your last N sessions, model X succeeded 87% on Generator" as one additional bullet in the rationale list.

### D6. MCP parity

`voce_profile`, `voce_recommend` as MCP tools. The conversational layer can answer "which model should I use for Generator on this project?" with a structured, explainable answer instead of a vibe.

---

## Acceptance Criteria

- [ ] `voce profile [--json]` deterministically reflects project shape + per-role demand from the IR
- [ ] `models.toml` seeded with current real catalog (Anthropic Opus 4.7 / Sonnet 4.6 / Haiku 4.5; OpenAI GPT-5; Google Gemini 3.5; representative open-weight via Ollama)
- [ ] `voce recommend [role] [--json]` produces typed, ranked, rationale-carrying output for every role; deterministic across runs
- [ ] `voce agents recommend --apply` round-trips through S92's `agents.toml`, preserving the user's overrides for any role they pinned
- [ ] Schema-locked under `docs/schema/contract/v1/{profile,recommendations}.schema.json`; drift gate per S79 A4
- [ ] MCP tools (`voce_profile`, `voce_recommend`) exposed; integration tests assert contract-versioned envelopes
- [ ] Feedback loop is opt-in *and* off-by-default; a test asserts no `.voce/agent-stats.jsonl` write occurs unless `telemetry.enabled = true`
- [ ] Docs page (`docs/agents/RECOMMENDATIONS.md`) explaining the heuristic, with examples — recommendations are auditable

## Out of Scope

- ML-based recommendation models (deliberately — opacity is the failure mode here)
- Recommending entire workflows / orchestrator changes (S92 territory)
- Real-time cost dashboards (S78 territory)
- Recommending across non-AI choices (compile targets, style packs) — separate sprint if pursued

## Risks

1. **Catalog rot.** Models change weekly. Mitigation: PR-based updates (acceptable for external state); a CI check probes each catalogued model's existence at provider endpoints (skipped offline); deprecate-in-place when a provider retires a model.
2. **Bias toward catalogued providers.** A model not in the catalog is invisible to recommendations. Mitigation: explicit `[provider.custom]` slot in `agents.toml` plus a doctor note when the configured strategy uses uncatalogued models so the user knows recommendations skip them.
3. **Cost-as-recommendation pressure.** Always-cheapest-first risks degrading pillars. Mitigation: rationale must cite reasoning + structured-output capability before cost; a recommendation that materially trades off pillar-critical capability for cost gets an explicit warning.
4. **Recommendation creep into pillar enforcement.** Never. The recommender suggests; the orchestrator enforces. A test asserts pillar gates run regardless of which model is recommended.

## Relationship to S92 / S79 / S78

- **S92** must ship first — there is no point recommending assignments to a system that can't act on them.
- **S79** provides the contract envelope shape and schema discipline; this sprint's outputs are schema-locked the same way.
- **S78** (opt-in telemetry, future) feeds D5 — the recommendation never depends on it, only refines with it.
