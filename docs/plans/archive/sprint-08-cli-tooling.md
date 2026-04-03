# Sprint 08 — CLI & Tooling

**Status:** Planned
**Goal:** Wire the validator into the CLI and implement all four subcommands: `voce validate`, `voce inspect`, `voce json2bin`, `voce bin2json`. After this sprint, a user can validate IR from the command line with colored output, inspect IR structure, and convert between JSON and FlatBuffer binary formats.
**Depends on:** Sprint 07 (all 9 validation passes operational)

---

## Deliverables

1. `voce validate` — run all passes, colored terminal output, `--format json` for CI
2. `voce inspect` — human-readable summary of an IR file (node counts, types, tree depth)
3. `voce json2bin` — convert `.voce.json` to `.voce` FlatBuffer binary
4. `voce bin2json` — convert `.voce` FlatBuffer binary to `.voce.json`
5. Serialization bridge module (JSON <-> FlatBuffer conversion)
6. Output formatter module (colored terminal + JSON output)
7. Exit codes: 0 (valid), 1 (validation errors), 2 (tool error / bad input)
8. CLI integration tests using `assert_cmd`
9. 10+ integration tests

---

## Tasks

### 1. Output Formatter (`formatter.rs`)

Create a formatter module that renders `ValidationResult` in two modes:

**Terminal mode (default):**
```
voce validate: examples/landing-page.voce.json

  ERROR  STR004  TextNode must have non-empty content
         at /children/3/content
         in pass: structural

  WARN   SEO002  Description should be 50-160 characters (got 22)
         at /metadata/description
         in pass: seo

  2 issues (1 error, 1 warning)
```

- Errors in red, warnings in yellow, pass names in dim
- Use `termcolor` or `colored` crate for portable color support
- Detect `--no-color` flag and `NO_COLOR` env variable

**JSON mode (`--format json`):**
```json
{
  "file": "examples/landing-page.voce.json",
  "valid": false,
  "errors": 1,
  "warnings": 1,
  "diagnostics": [
    {
      "severity": "error",
      "code": "STR004",
      "message": "TextNode must have non-empty content",
      "path": "/children/3/content",
      "pass": "structural"
    }
  ]
}
```

### 2. `voce validate` Implementation

Wire `main.rs` validate subcommand to:
1. Read file from path argument
2. Detect format (`.voce.json` = JSON, `.voce` = binary, error otherwise)
3. If binary, convert to serde IR via serialization bridge
4. If JSON, deserialize directly
5. Build NodeIndex
6. Run all 9 passes
7. Format and print results
8. Exit with code 0 (no errors), 1 (has errors), or 2 (file not found / parse error)

CLI flags:
- `--format <terminal|json>` — output format (default: terminal)
- `--no-color` — disable colored output
- `--pass <name>` — run only a specific pass (optional, for debugging)
- `--warn-as-error` — treat warnings as errors (exit 1 if any warnings)

### 3. `voce inspect` Implementation

Print a human-readable summary of an IR file:

```
voce inspect: examples/landing-page.voce.json

  Format:     JSON canonical
  Root:       ViewRoot "Landing Page"
  Nodes:      47 total
  Tree depth: 6 levels

  Node types:
    Container      12
    TextNode       15
    MediaNode       4
    Surface         3
    GestureHandler  5
    DataNode        3
    StateMachine    2
    FormNode        1
    SemanticNode    2

  State machines: 2
    - nav-toggle (3 states, 4 transitions)
    - form-flow (4 states, 6 transitions)

  Routes: 0 (no RouteMap)
  i18n:   not used
  a11y:   15/15 interactive nodes have SemanticNode
```

### 4. Serialization Bridge (`bridge.rs`)

Conversion between JSON canonical format and FlatBuffer binary:

- `json_to_flatbuffer(json: &str) -> Result<Vec<u8>>` — parse JSON, build FlatBuffer
- `flatbuffer_to_json(bytes: &[u8]) -> Result<String>` — read FlatBuffer, emit JSON
- Uses the FlatBuffers generated code from `voce-schema` for binary operations
- Uses the serde IR model for JSON operations
- Round-trip guarantee: `json -> fb -> json` produces semantically identical output (field order may differ)

### 5. `voce json2bin` Implementation

```bash
voce json2bin input.voce.json -o output.voce
voce json2bin input.voce.json              # writes to input.voce (same name, different extension)
```

- Read JSON, validate it parses as valid IR structure (not full validation — just deserialization)
- Convert to FlatBuffer binary
- Write to output path
- Print file size comparison: `Converted: 12,847 bytes JSON -> 3,204 bytes binary (75% smaller)`

### 6. `voce bin2json` Implementation

```bash
voce bin2json input.voce -o output.voce.json
voce bin2json input.voce                   # writes to input.voce.json
```

- Read binary FlatBuffer
- Verify it is a valid FlatBuffer (magic bytes / verifier)
- Convert to JSON canonical format
- Write to output path with pretty-printing (2-space indent)

### 7. CLI Integration Tests

Use `assert_cmd` and `predicates` crates for end-to-end CLI testing.

Tests:
- `validate` on valid fixture returns exit code 0 and "valid" message
- `validate` on invalid fixture returns exit code 1 and expected error codes
- `validate --format json` returns valid JSON with expected structure
- `validate` on nonexistent file returns exit code 2
- `validate` on malformed JSON returns exit code 2 with parse error
- `inspect` on valid fixture prints node count and type summary
- `json2bin` produces a file that `bin2json` can round-trip
- `json2bin` on invalid JSON returns exit code 2
- `bin2json` on corrupted binary returns exit code 2
- `--no-color` flag produces output without ANSI escape codes

---

## Files to Create / Modify

### Create
- `packages/validator/src/formatter.rs` — output formatting (terminal + JSON)
- `packages/validator/src/bridge.rs` — JSON <-> FlatBuffer serialization bridge
- `packages/validator/src/inspect.rs` — IR inspection and summary logic
- `tests/integration/cli_validate.rs` — validate subcommand tests
- `tests/integration/cli_inspect.rs` — inspect subcommand tests
- `tests/integration/cli_convert.rs` — json2bin / bin2json tests

### Modify
- `packages/validator/src/main.rs` — implement all 4 subcommands
- `packages/validator/src/lib.rs` — add `pub mod formatter; pub mod bridge; pub mod inspect;`
- `packages/validator/Cargo.toml` — add `colored`, `assert_cmd`, `predicates` dependencies

---

## Acceptance Criteria

- [ ] `voce validate valid.voce.json` exits 0, prints success message
- [ ] `voce validate invalid.voce.json` exits 1, prints colored diagnostics
- [ ] `voce validate --format json invalid.voce.json` prints valid JSON to stdout
- [ ] `voce validate nonexistent.voce.json` exits 2 with file-not-found error
- [ ] `voce validate --pass structural` runs only the structural pass
- [ ] `voce validate --warn-as-error` exits 1 on warnings
- [ ] `voce inspect` prints node counts, types, tree depth, state machine summary
- [ ] `voce json2bin` converts JSON to binary and reports size comparison
- [ ] `voce bin2json` converts binary to pretty-printed JSON
- [ ] Round-trip: `json2bin` then `bin2json` produces semantically identical JSON
- [ ] `NO_COLOR` environment variable disables colored output
- [ ] 10+ integration tests passing with `assert_cmd`
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes

---

## Notes

- The `colored` crate respects `NO_COLOR` automatically. Also support `--no-color` flag explicitly for discoverability.
- Exit code 2 is used for tool-level errors (bad input, missing file, parse failure) to distinguish from validation errors (exit code 1). This follows the convention of tools like `rustfmt` and `eslint`.
- The serialization bridge is the critical new module. If FlatBuffer generated code is difficult to work with for building buffers programmatically, consider using `flatbuffers::FlatBufferBuilder` directly rather than the generated builder API.
- `voce inspect` does not run validation passes. It only reads and summarizes the IR structure. This makes it fast and useful for quick checks.
