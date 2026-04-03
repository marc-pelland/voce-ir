# Sprint 10 — Documentation & Release

**Status:** Planned
**Goal:** Write all Phase 1 documentation, tag v0.1.0, and update project tracking files. After this sprint, the project has comprehensive docs for every schema node type, every error code, the CLI, and the architecture. Phase 1 is complete.
**Depends on:** Sprint 09 (examples, polished diagnostics, 30+ tests)

---

## Deliverables

1. `ARCHITECTURE.md` — system architecture and design decisions
2. `docs/SCHEMA.md` — every node type documented with fields, examples, and constraints
3. `docs/CLI.md` — full CLI reference for all subcommands
4. `docs/ERROR_CODES.md` — all 50+ error codes with explanations and fix guidance
5. `CHANGELOG.md` — v0.1.0 release notes
6. Updated `docs/ROADMAP.md` — Phase 1 marked complete, Phase 2 detailed
7. Updated `CLAUDE.md` — Phase 1 tasks checked off, Phase 2 current task section
8. `v0.1.0` git tag

---

## Tasks

### 1. ARCHITECTURE.md

Write at the repository root. Cover:

- **System overview** — what Voce IR is, the pipeline (intent -> IR -> compiler -> output)
- **Crate structure** — `voce-schema`, `voce-validator`, `voce-compiler-dom`, role of each
- **IR format** — FlatBuffers binary as wire format, JSON canonical as development format
- **Validation architecture** — pass-based engine, serde IR model vs FlatBuffers, NodeIndex
- **Pass ordering** — why structural runs first, dependency graph between passes
- **Design decisions** — why FlatBuffers (not protobuf, not custom binary), why serde IR model for validation, why accessibility is a compile error, why single-file HTML output
- **Data flow diagrams** — IR lifecycle from creation to compiled output
- **Extension points** — how to add a new node type, how to add a new validation pass, how to add a new compiler target
- **Performance considerations** — NodeIndex O(1) lookups, pass short-circuiting, binary format size advantages

Target: 200-300 lines, thorough but not padded.

### 2. docs/SCHEMA.md

Document every node type in the schema. For each node:

- **Name and purpose** — one-line description
- **Fields** — table with field name, type, required/optional, description
- **Constraints** — validation rules that apply (reference error codes)
- **Example** — minimal JSON snippet showing the node in use
- **Relationships** — what nodes it can contain or reference

Node types to document (25+):
- Layout: ViewRoot, Container, Surface, TextNode, MediaNode
- State: StateMachine, DataNode, ComputeNode, EffectNode, ContextNode
- Motion: Transition, Sequence, GestureHandler, ScrollBinding, PhysicsBody
- Navigation: RouteMap, RouteTransition
- Accessibility: SemanticNode, LiveRegion, FocusTrap, ReducedMotion
- Theming: ThemeNode, PersonalizationSlot, ResponsiveRule
- Data: ActionNode, SubscriptionNode, AuthContextNode, ContentSlot, RichTextNode
- Forms: FormNode, FormField, ValidationRule, CrossFieldValidation, FormSubmission
- SEO: PageMetadata, OpenGraphData, StructuredData
- i18n: LocalizedString, MessageParameter, FormatOptions, MessageCatalog

Target: 500-800 lines. This is the primary reference for anyone building IR.

### 3. docs/CLI.md

Full CLI reference:

```
voce validate <file> [--format terminal|json] [--no-color] [--pass <name>] [--warn-as-error]
voce inspect <file>
voce json2bin <input> [-o <output>]
voce bin2json <input> [-o <output>]
```

For each subcommand:
- Description and purpose
- All flags and arguments with defaults
- Exit codes and their meaning
- Example invocations with sample output
- Common workflows (validate before commit, CI integration, debugging a specific pass)

Target: 100-150 lines.

### 4. docs/ERROR_CODES.md

Document all 50+ error codes organized by pass:

| Code | Pass | Severity | Message | Fix |
|------|------|----------|---------|-----|

For each code:
- The rule it enforces
- Why this rule exists (link to WCAG, OWASP, or design decision)
- How to fix it (concrete example of before/after IR)
- Related codes

Organize by pass:
- Structural (STR001-STR008)
- References (REF001-REF010)
- State Machine (STA001-STA005)
- Accessibility (A11Y001-A11Y009)
- Security (SEC001-SEC006)
- SEO (SEO001-SEO008)
- Forms (FRM001-FRM008)
- i18n (I18N001-I18N005)
- Motion (MOT001-MOT006)

Total: 8 + 10 + 5 + 9 + 6 + 8 + 8 + 5 + 6 = 65 error codes.

Target: 400-600 lines.

### 5. CHANGELOG.md

Create at repository root. Follow [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
# Changelog

## [0.1.0] - 2026-XX-XX

### Added
- FlatBuffers IR schema with 25+ node types across 9 domains
- JSON canonical format for development and validation
- 9 validation passes: structural, references, state_machine, a11y, security, seo, forms, i18n, motion
- 65 error codes with actionable diagnostics
- CLI: voce validate, voce inspect, voce json2bin, voce bin2json
- Reference landing page IR example
- 5 intent-IR training pairs for Phase 3 RAG
- 30+ test cases
- Comprehensive documentation: ARCHITECTURE.md, SCHEMA.md, CLI.md, ERROR_CODES.md
```

### 6. Update ROADMAP.md

- Mark Phase 1 milestones as complete with dates
- Add Phase 2 details (DOM compiler, preview server, device profiles)
- Update timeline estimates based on Phase 1 actuals

### 7. Update CLAUDE.md

- Check off all Phase 1 tasks in the "Current task status" section
- Update "Current Phase" to Phase 2
- Add Phase 2 task checklist
- Update any conventions or architecture notes that evolved during Phase 1

### 8. Tag v0.1.0

```bash
git tag -a v0.1.0 -m "Phase 1: Specification & Foundation complete"
```

Pre-tag checklist:
- All tests pass
- All clippy warnings resolved
- All docs written and reviewed
- Examples validate cleanly
- CHANGELOG.md has the release date filled in

---

## Files to Create / Modify

### Create
- `ARCHITECTURE.md`
- `docs/SCHEMA.md`
- `docs/CLI.md`
- `docs/ERROR_CODES.md`
- `CHANGELOG.md`

### Modify
- `docs/ROADMAP.md` — Phase 1 complete, Phase 2 detailed
- `CLAUDE.md` — Phase 1 checked off, Phase 2 current
- `README.md` — update with installation instructions, quick start, link to docs

---

## Acceptance Criteria

- [ ] `ARCHITECTURE.md` covers system overview, crate structure, validation architecture, design decisions, extension points
- [ ] `docs/SCHEMA.md` documents all 25+ node types with fields, constraints, examples
- [ ] `docs/CLI.md` documents all 4 subcommands with flags, exit codes, examples
- [ ] `docs/ERROR_CODES.md` documents all 65 error codes with fix guidance
- [ ] `CHANGELOG.md` follows Keep a Changelog format with v0.1.0 entry
- [ ] `docs/ROADMAP.md` has Phase 1 marked complete
- [ ] `CLAUDE.md` has all Phase 1 tasks checked and Phase 2 section ready
- [ ] `README.md` has working quick-start instructions
- [ ] `v0.1.0` git tag created
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo fmt --check` passes
- [ ] All examples validate cleanly against the tagged release

---

## Notes

- SCHEMA.md is the most labor-intensive deliverable. Consider generating the field tables from the `.fbs` files to avoid drift, but hand-write the descriptions, constraints, and examples.
- ERROR_CODES.md is the second most labor-intensive. The messages and hints from Sprint 09's diagnostic polish should be reusable here.
- The v0.1.0 tag marks the schema and validator as stable for Phase 2 development. Breaking schema changes after this point require a migration path.
- Phase 2 (DOM compiler) can begin immediately after tagging. The compiler consumes validated IR, so the validator is a hard dependency.
