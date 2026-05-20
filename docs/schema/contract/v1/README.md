# Voce Agent Contract — JSON Schemas (v1)

The agent contract is the set of versioned, machine-consumable
envelopes Voce emits for AI agents, MCP clients, and third-party
tooling. **The schemas in this directory ARE the contract.** If the
shipped output doesn't match the schema, the binary is wrong.

## Current envelopes (v1.0.0)

| Envelope | Produced by | Schema |
| --- | --- | --- |
| `skills` | `voce skills --json` | `skills.schema.json` |
| `graph` | `voce graph <file> --json` | `graph.schema.json` |
| `doctor` | `voce doctor --json` | `doctor.schema.json` |

### Planned for v1.x

| Envelope | Produced by | Status |
| --- | --- | --- |
| `validator` | `voce validate --format json` | Pre-dates A4; schema-ization deferred to follow-up slice. Shape is stable today via S67 conventions. |
| `perf-report` | `voce compile --perf-report` | Pre-dates A4; same. |

Both will land as additive minors once their underlying types receive
`JsonSchema` derives (mechanical; deferred only to keep this slice
complete and tested rather than sprawling). |

## Versioning policy (semver)

`contract_version` is **independent of `voce_version` and
`voce-schema_version`** — it tracks the *contract envelope shapes
only*. Every envelope carries `contract_version` so a consumer always
knows what to expect.

- **Major bump (2.0.0)** — required for any breaking change:
  removing a field, renaming a field, narrowing a field's type,
  removing or renaming a diagnostic code or check ID, removing or
  renaming a CLI command or compile-target ID.
- **Minor bump (1.1.0)** — required for any additive change:
  new optional field, new enum variant, new envelope, new diagnostic
  code, new compile target. Consumers built against 1.0.x must
  continue to parse 1.1.x output.
- **Patch bump (1.0.1)** — wording-only fixes (e.g. a `hint` text
  edit) that do not change shape or semantics.

## Stability of identifiers

The following identifiers are part of the contract and follow the
versioning above:

- **Diagnostic codes** (`A11Y007`, `STR001`, etc.) — once shipped, a
  code's meaning is fixed. Removals or rename = major.
- **CLI subcommand names** (`skills`, `graph`, `validate`, …) — same.
- **Compile target IDs** (`dom`, `email`, …) — same.
- **Reference-edge `kind`s** in `graph` output — same.
- **Doctor check IDs** (`DOC-TOOLCHAIN-NNN`, `DOC-VOCE-NNN`) — same.

The historical gap in `A11Y00*` (no `A11Y002`) is intentional and
permanent — codes are never reassigned, even when retired.

## Drift gate (how it stays honest)

`packages/validator/tests/contract_schemas.rs` runs two assertions per
envelope on every `cargo test`:

1. **Drift** — regenerate the schema from the live struct (via
   `schemars`) and compare to the committed file. Any change to the
   contract structs fails the test, surfacing the diff in code review
   and forcing a conscious version-bump decision.
2. **Live conformance** — build a real envelope in-process and
   validate it against the committed schema (via `jsonschema`). The
   published contract is therefore never aspirational.

To intentionally update a schema after a contract change:

```sh
UPDATE_CONTRACT_SCHEMAS=1 cargo test -p voce-validator --test contract_schemas
```

Review the diff. Bump `contract_version` per the rules above. Commit.

## Deprecation policy

A field, enum variant, code, or ID may be marked `deprecated` in any
minor release. **Removal requires waiting at least one major release
afterwards.** No identifier is ever silently changed.
