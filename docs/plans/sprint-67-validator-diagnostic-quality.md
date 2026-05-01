# Sprint 67 — Validator Diagnostic Quality

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Make the validator's output the system's strongest teaching surface. Today it returns `{valid, errors, warnings, diagnostics}` — a verdict. After this sprint it returns per-pass timing, actionable hints with documented fix suggestions, machine-readable JSON Patch fixes for safe auto-correction, and stable diagnostic codes linked to docs. Every consumer (CLI, MCP, REPL, web) immediately benefits.

**Depends on:** validator (S06–S07), schema (S02–S05). Independent of S65/S66 — they consume what this produces.

---

## Motivation

S61 surfaced this gap (F-001): the validator emits a global verdict but no per-pass detail. The cinematic "9 passes light up" visualization had to fake per-pass status. More importantly: the conversational pillars from S65/S66 require the validator to teach the model *why* something failed and *how* to fix it. Today's diagnostics carry only a `code`, `message`, and `node_path`. The model gets generic "STR002: NONE node is missing a node_id" — no context, no next step.

The fix: structured per-pass output, an actionable `hint` field on every diagnostic, machine-readable JSON Patch fixes for the common cases, and a `voce fix` command that previews/applies them. This turns the validator from a gatekeeper into a teacher.

---

## Deliverables

### 1. Per-pass output (`voce validate --verbose-passes`)

New JSON shape (additive to existing `--format json`):

```json
{
  "valid": false,
  "errors": 2,
  "warnings": 1,
  "diagnostics": [...],
  "passes": [
    {
      "name": "structural",
      "duration_us": 47,
      "rules_checked": 12,
      "diagnostics": ["STR002"]
    },
    {
      "name": "references",
      "duration_us": 31,
      "rules_checked": 9,
      "diagnostics": []
    },
    ...
  ]
}
```

Engine instrumentation: each pass's `run()` is wrapped with a `Duration::now()` timer; the engine records timing per pass and surfaces `rules_checked` (the number of rule-evaluations performed, not just diagnostics emitted). Output format is opt-in via `--verbose-passes` so existing consumers don't break.

### 2. `hint` field on every diagnostic

Currently `Diagnostic.hint` exists as `Option<String>` but is mostly `None`. Populate it for every error code. Examples:

| Code | Today | After |
| --- | --- | --- |
| STR002 | `"NONE node is missing a node_id"` | hint: `"Add `\"node_id\": \"<unique-name>\"` to the node. Voce IR uses node_ids for cross-references and debugging."` |
| A11Y001 | `"Interactive node has no SemanticNode"` | hint: `"Add a `SemanticNode` with `role` matching the node's purpose (`button`, `link`, `checkbox`, etc.) and reference it via `semantic_node_id`."` |
| SEC001 | `"Mutation action has no CSRF token"` | hint: `"Set `csrf_protected: true` on the ActionNode, OR add a `csrf_token_field` referencing a hidden form input."` |

All 46 error codes get hints. Each hint references either a schema field or a concrete code change — no vague advice.

### 3. JSON Patch fix proposals

For codes where the fix is unambiguous, attach a `fix` field:

```json
{
  "code": "STR002",
  "severity": "error",
  "message": "NONE node is missing a node_id",
  "node_path": "/root/children/2",
  "hint": "Add a node_id...",
  "fix": {
    "type": "json-patch",
    "operations": [
      { "op": "add", "path": "/root/children/2/node_id", "value": "node-2" }
    ],
    "confidence": "safe",
    "preview": "Adds node_id \"node-2\" to /root/children/2"
  }
}
```

`confidence` ∈ `{safe, suggested, risky}`:
- `safe` — purely additive, no semantic change (e.g., adding a missing node_id)
- `suggested` — opinionated default that may be wrong (e.g., picking a `role` for a SemanticNode)
- `risky` — substantive change (e.g., restructuring the node tree)

Initial coverage target: every STR* error gets a `safe` fix. Every REF* and FRM* gets at least `suggested`. A11y/SEC/SEO get `suggested` where deterministic.

### 4. New CLI: `voce fix`

```
voce fix <ir-file> [--apply | --dry-run] [--confidence safe|suggested|risky]
```

- `--dry-run` (default) — print the proposed patch, don't write
- `--apply` — apply patches at or below the given confidence threshold; default threshold is `safe`
- `--confidence` — change the threshold

Output: a unified diff of the IR JSON before/after, plus the validation result on the patched IR. Iteratively re-runs validation after applying patches to catch cascading issues.

### 5. Diagnostic code → docs URL

Every diagnostic gets a `docs_url` field pointing to the validator rule reference page. The mdBook `docs/` site (S55) gains a `validator-rules/` section with one page per code. URL pattern: `https://voce-ir.xyz/docs/validator-rules/STR002`.

### 6. `--list-passes` and `--list-codes` CLI flags

Stable, machine-readable enumerations so consumers (the front-end's `VALIDATION_PASSES` const, MCP tool descriptions, docs generator) can stay in sync without parsing source files. Replaces the source-file-parsing approach in `verify-pass-list.mjs` from S61 with a real API.

```
$ voce validate --list-passes
{ "passes": ["structural", "references", "state-machine", "accessibility", "security", "seo", "forms", "i18n", "motion"] }

$ voce validate --list-codes
{ "codes": [
  { "code": "STR001", "pass": "structural", "severity": "error", "summary": "..." },
  ...
] }
```

### 7. Severity escalation config

A `.voce/validator.toml` (or section in an existing config) lets a project promote warnings to errors:

```toml
[severity]
SEO007 = "error"   # we require og:image on this project
A11Y005 = "error"  # alt text required everywhere
```

Validator reads this config and applies overrides before emitting diagnostics. Useful for projects with stricter requirements than Voce's defaults.

### 8. Tests

- Unit tests for every populated `hint` field (assert presence, length cap)
- Unit tests for every fix patch (apply patch to a fixture, re-validate, expect green or fewer diagnostics)
- Integration test for `voce fix --dry-run` and `--apply` paths
- Snapshot tests for the new `--verbose-passes` JSON shape
- CI step: `voce validate --list-codes | jq` produces stable JSON

---

## Acceptance Criteria

- [ ] `voce validate --verbose-passes` emits per-pass timing + rules-checked + per-pass diagnostic codes
- [ ] All 46 error codes have non-null `hint` fields
- [ ] Every STR* code has a `safe` JSON Patch fix
- [ ] `voce fix --dry-run <file>` prints a coherent diff for fixable IRs
- [ ] `voce fix --apply <file>` writes the patched IR and re-validates
- [ ] Every diagnostic carries a `docs_url`
- [ ] mdBook gains `validator-rules/<code>.md` pages for every code
- [ ] `voce validate --list-passes` and `--list-codes` emit stable JSON
- [ ] `.voce/validator.toml` severity overrides work
- [ ] Existing snapshot tests refreshed; consumer code (site-hero, mcp-server) updated to use the new shapes where useful
- [ ] No regression in existing CLI invocations — additive only

---

## Risks

1. **Hint quality degrades over time.** Hints are written once and rarely revisited. Mitigation: a `hint-style.md` doc with examples, plus a CI check that rejects diagnostics with hints under 30 chars or over 300 chars.
2. **Fix patches can be wrong.** A `safe` patch that turns out to be wrong is a real regression. Each patch needs a test that asserts the patched IR validates AND has the same semantic intent (not just structural validity). For ambiguous cases, prefer `suggested` over `safe`.
3. **`--verbose-passes` becomes the default expected shape.** Once consumers depend on it, removing it is breaking. Document stability tier (probably "stable" once shipped).
4. **Bundle growth.** Adding hints + docs_urls to every diagnostic increases binary size. Consider lazy-loading the hint text from a static table rather than inlining.

---

## Out of Scope

- Auto-fix for animation safety, state machine, or i18n issues (too contextual; would require additional input from the user)
- Multi-language hint messages — English only for v1
- A "rule severity" UI in the playground — defer
- Compiler-side fix proposals (e.g., "this IR validates but compiles to a 500KB bundle, here's how to slim it")
