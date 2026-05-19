# Sprint 79 — Agent Capability Surface ("The Agent Contract")

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Consolidate Voce's scattered agent-facing affordances into one **documented, versioned, machine-consumable contract**. Today an agent learns Voce through MCP tool descriptions, `--list-passes`, `--list-codes`, `--perf-report`, and JSON Patch fixes — useful, but disjoint and undocumented as a stable interface. After this sprint, any AI agent (MCP client, the REPL, a third-party harness) can discover *everything Voce can do*, check *whether a project and toolchain are healthy*, inspect *the IR as a structured graph*, and rely on *a semver'd output schema* — without reading prose or source.

**Depends on:** validator (S06–S07), S67 (diagnostics quality — hints/fixes/codes), S65 (MCP storage + tools), S71 (perf-report). Independent of S60/S62/S68; S68 conformance work is a sibling, cross-referenced below.

---

## Motivation

Vercel Labs shipped [`vercel-labs/zero`](https://github.com/vercel-labs/zero) (2026-05) — a general-purpose systems language whose entire thesis is "the toolchain is designed to be consumed by AI agents from day one." Its agent surface is deliberate and unified: `zero skills` (capability discovery), `zero doctor --json` (health), `zero graph --json` (structured facts), `zero size --json` (artifact reporting), and a stable-coded JSON diagnostic envelope with typed repair metadata.

This is external validation of the exact direction S67 took — and a checklist. Voce already has most of the *pieces* (per-pass diagnostics, hints, JSON Patch fixes, docs URLs, `--perf-report`, MCP tools/resources, an internal `NodeIndex`). What it lacks is the **unification and the contract**: there is no single self-describing manifest, no project/toolchain health command, no exported IR graph, and the JSON output envelopes are not a published, versioned interface third parties can build on.

Because Voce is a UI IR with **no human-readable source text in the pipeline**, the agent contract is not a convenience — it is the *only* interface. That raises the bar relative to Zero (an agent can never "just read the code") and is also the differentiator: the contract can expose *semantic UI, state, and data-flow facts* that a general-purpose language toolchain structurally cannot.

---

## Part A — Parity: match Zero's agent surface

Mapping of Zero's agent affordances to Voce's status today and this sprint's target:

| Zero affordance | Voce today | Status | S79 deliverable |
| --- | --- | --- | --- |
| `zero skills` (capability discovery) | `--list-passes`, `--list-codes`, MCP tool descriptions | **Partial / scattered** | `voce skills --json` — unified capability manifest |
| `zero doctor --json` (health) | none (no toolchain/project health command) | **Gap** | `voce doctor --json` |
| `zero graph --json` (structured facts) | internal `NodeIndex` (not exported) | **Gap** | `voce graph --json` |
| `zero size --json` (artifact reporting) | `voce compile --perf-report` (S71) | **Have** | align to shared envelope + stable codes |
| Stable JSON diagnostics + typed repair | S67 diagnostics: codes, hints, JSON Patch `fix` | **Have, not versioned** | publish semver'd JSON Schemas |
| Conformance suite + benchmarks | per-compiler snapshot tests | **Gap** | cross-reference S68; ship `voce conformance` runner |
| "One obvious way" / regular surface | S72 schema audit (Part 1 done) | **Partial** | "agent-authorability" lint pass (Part B) |

### A1. `voce skills [--json]` — unified capability manifest

One command emits the complete machine-readable description of what this Voce build can do:

```json
{
  "contract_version": "1.0.0",
  "voce_version": "1.0.0",
  "schema_version": 3,
  "node_types": [
    { "name": "Container", "union_tag": 2, "fields": [...], "required": [...],
      "docs_url": "https://voce-ir.xyz/docs/nodes/container" }
  ],
  "compile_targets": [
    { "id": "dom", "stable": true, "outputs": ["html"], "notes": "single-file, zero-runtime" },
    { "id": "ios-swiftui", "stable": true, "outputs": ["swift"] }
  ],
  "validation_passes": [
    { "name": "accessibility", "codes": ["A11Y001", "...", "A11Y009"] }
  ],
  "diagnostic_codes": [
    { "code": "A11Y001", "severity": "error", "fixable": false,
      "docs_url": "https://voce-ir.xyz/docs/codes/A11Y001" }
  ],
  "style_packs": [...],
  "cli_commands": [...]
}
```

- Sourced by *reflecting existing structures* — pass registry from `engine.rs`, codes from the S67 code table, targets from the compiler registry, node types from the generated schema bindings. No hand-maintained list (a hand list rots; this must be generated or it fails its own purpose).
- Exposed identically as an MCP tool (`voce_skills`) and MCP resource so the conversational layer and CLI share one source.
- `--json` is the contract; default human output is a rendered summary.

### A2. `voce doctor [--json]` — toolchain + project health

Project-level analog of the IR-level `voce fix`. Checks and reports, with stable check IDs:

- **Toolchain:** `flatc` present/version, schema bindings in sync with `.fbs`, contract version vs. installed `voce` version.
- **Project (`.voce/`):** brief present and parseable, decision log integrity, session-history readable, drift status (reuses S65 drift v1), preferences valid.
- **IR set:** every `*.voce.json` validates; orphaned/unreferenced fixtures flagged.

```json
{
  "ok": false,
  "checks": [
    { "id": "DOC-TOOLCHAIN-001", "title": "flatc available", "status": "pass" },
    { "id": "DOC-VOCE-003", "title": ".voce/ brief present", "status": "warn",
      "detail": "No project brief; drift detection disabled.",
      "hint": "Run `voce init` or create .voce/brief.md.",
      "docs_url": "https://voce-ir.xyz/docs/doctor/DOC-VOCE-003" }
  ]
}
```

Exit code: `0` all pass, `1` any error-level check, `0` with warnings (configurable via `--strict`).

### A3. `voce graph [--json]` — structured IR facts

Promote the internal `NodeIndex` (`packages/validator/src/index.rs`) to a public, documented export. Not just a node tree — the *graph facts an agent needs to reason and self-correct*:

- **Composition graph:** nodes with `id`, `type`, `path`, `parent`, `children`.
- **Reference edges:** `semantic_node_id`, `theme_id`, data/action/subscription references, route targets — as typed edges with resolution status (resolved / dangling).
- **State-machine graph:** states, transitions, and **reachability** (unreachable states flagged — overlaps but is distinct from validator STA codes; here it is queryable data, not a verdict).
- **Data-flow graph:** DataNode → ContentSlot/ComputeNode edges; unbound slots flagged.

This is the deliverable Zero's `zero graph` inspired but Voce can go materially further because the IR encodes UI semantics, not just module dependencies.

### A4. Versioned output schemas (`contract_version`)

Publish JSON Schema documents for every machine envelope — validator output (S67 shape), `skills`, `doctor`, `graph`, `perf-report` (S71). Live in `docs/schema/contract/v1/` and served at `voce-ir.xyz/docs/contract`. Rules:

- `contract_version` (semver) on every machine output; additive changes bump minor, breaking changes bump major.
- A CI test asserts every command's `--json` output validates against its published schema (the contract is enforced, not aspirational).
- Diagnostic codes and check IDs are declared **stable** — documented removal/rename policy (deprecate-then-remove across a major).

### A5. `voce conformance` runner (cross-reference S68)

S68 builds the cross-target fixture set + semantic-summary verifier. S79 wraps it as a *publishable* command so third-party compilers/adapters can self-certify against the contract:

- `voce conformance --target <id>` runs the S68 fixtures, compiles via the named target, diffs semantic summaries, emits a contract-versioned pass/fail report.
- If S68 has not landed when S79 starts, A5 ships the runner skeleton + report envelope and is wired to fixtures when S68 completes (tracked as a dependency, not a blocker for A1–A4).

---

## Part B — Differentiators: what Zero structurally cannot do

These exist *because* Voce is a UI IR with no source text. They are the "stand out" half.

### B1. Semantic capability surface, not stdlib surface

`voce skills` reports **accessibility roles, compile targets, style packs, and the validity envelope per target** — the agent learns "this build can emit accessible SwiftUI with the `editorial` style pack and these 27 node types," not "here are stdlib functions." A general-purpose language's capability manifest cannot express target-specific UI semantics; Voce's is the actual design space.

### B2. Self-correcting fix loop (`voce fix --until-clean --plan`)

S67 ships single JSON Patch fixes. B1 of Zero's repair metadata is "fix plans"; Voce goes further: a **multi-step fix plan** object (ordered patches with per-step rationale and the code each resolves) plus an apply→re-validate→repeat loop that converges or reports the irreducible residue. Output is contract-versioned so an agent can drive it headlessly:

```json
{
  "plan": [
    { "step": 1, "code": "STR002", "rationale": "node_id required for references",
      "patch": [{ "op": "add", "path": "/root/children/0/node_id", "value": "hero" }] }
  ],
  "converges": true,
  "residual_codes": []
}
```

### B3. "Agent-authorability" lint (extends S72)

Zero's principle is "one obvious way to express most things." Apply it as a *validator pass over the IR design space*: flag schema regions where the same UI outcome is expressible multiple equivalent ways (ambiguity that degrades AI authorship reliability). Output feeds the S72 schema audit and produces concrete schema-tightening recommendations. This is a meta-capability — Voce auditing its own authorability for agents.

### B4. Contract-as-only-interface guarantee

Because there is no source text, document and test the invariant that **every fact an agent needs is reachable through the contract** (skills + graph + diagnostics). Add a CI "contract completeness" check: pick representative agent tasks (add a form field, fix an a11y error, retarget to iOS) and assert each is achievable using only contract outputs — no source inspection, because there is no source. Zero cannot make this guarantee; an agent can always fall back to reading `.zero` files.

---

## Acceptance Criteria

1. `voce skills --json`, `voce doctor --json`, `voce graph --json` exist, generated by reflection (no hand-maintained lists), each emitting `contract_version`.
2. JSON Schemas for all five envelopes published under `docs/schema/contract/v1/`; CI validates real command output against them.
3. MCP server exposes `skills` / `doctor` / `graph` as tools + resources sharing the CLI's code path.
4. `voce fix --until-clean --plan` produces a convergent, contract-versioned multi-step plan; integration test proves convergence and residue reporting.
5. `voce conformance` runner exists and is wired to S68 fixtures (or skeleton + envelope if S68 pending).
6. Agent-authorability lint pass emits at least the known S72 ambiguity findings as structured output.
7. Contract-completeness CI check passes for the three representative agent tasks.
8. Docs page at `voce-ir.xyz/docs/contract` describes the contract, stability policy, and versioning rules.
9. `cargo test --workspace && cargo clippy --workspace -- -D warnings` green; zero new runtime deps in compiled output.

## Out of Scope

- New compile targets or node types (schema work is S72/S83).
- The S68 fixture authoring itself (consumed here, not created).
- Telemetry/analytics (S78) — the contract is pull, not push.
- IDE surfacing of the contract (S86).

## Risk Notes

- **Reflection cost:** `skills` must be generated from real registries. If a registry isn't introspectable today (e.g., compiler targets are not enumerated in code), a small registration refactor is in scope — budget for it.
- **Contract stability is a promise.** Once published, codes/check-IDs/envelope shapes are an API. The deprecate-then-remove policy must land *with* v1, not after.
- **Scope:** Part A is the must-ship. Part B differentiators can split to a fast-follow (S79b) if the sprint runs long — but B2 (fix loop) is small given S67 and should stay.
