# Contributing

This guide covers development setup, code conventions, the working pattern for
new features, and the process for adding a new compile target.

## Development Setup

### Prerequisites

- Rust 1.85+ (edition 2024 support required)
- FlatBuffers compiler (`flatc`) for schema regeneration
- Node.js 18+ and npm (for the TypeScript AI bridge)

### Clone and Build

```bash
git clone https://github.com/fireburnsup/voce-ir.git
cd voce-ir

# Build the entire workspace
cargo build --workspace

# Run all tests
cargo test --workspace

# Lint (must pass with zero warnings)
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --check
```

If all four commands pass, your environment is ready.

### Regenerating FlatBuffers Bindings

After editing any `.fbs` file in `packages/schema/schemas/`:

```bash
# Rust bindings
flatc --rust -o packages/schema/src/generated/ packages/schema/schemas/voce.fbs

# TypeScript bindings (for AI bridge)
flatc --ts -o packages/ai-bridge/src/generated/ packages/schema/schemas/voce.fbs
```

## Code Conventions

### Rust

- **Edition 2024** (latest stable). Fall back to edition 2021 per-crate only
  if a dependency requires it (e.g., FlatBuffers codegen).
- **Error handling:** `thiserror` for library error types, `anyhow` for CLI
  entry points. No `unwrap()` in library code -- propagate errors with `?`.
- **CLI arguments:** `clap` derive API.
- **Naming:** `snake_case` for files and functions, `PascalCase` for types and
  enums.
- **Documentation:** Every public function and type has a `///` doc comment.
- **Formatting:** `cargo fmt` with default settings. Run before every commit.
- **Linting:** `cargo clippy -- -D warnings` enforces a zero-warnings policy.
  CI will reject PRs with clippy warnings.

### Testing

- Unit tests live in-file under `#[cfg(test)]` modules.
- Integration tests live in `tests/`.
- Every new node type needs valid and invalid test IR in `tests/schema/`.
- Compiler output uses `insta` snapshot testing to catch regressions.

## Working Pattern

When implementing a new feature, follow this sequence:

1. **Schema** -- If new IR types are needed, add or modify `.fbs` files in
   `packages/schema/schemas/`. Update the `ChildUnion` in `voce.fbs` if
   adding a new node type.

2. **Bindings** -- Regenerate Rust and TypeScript bindings with `flatc`.

3. **Validation** -- Add a validation pass (or extend an existing one) in
   `packages/validator/src/passes/`. Implement the `ValidationPass` trait
   and register the pass in `all_passes()`.

4. **Compiler** -- Add codegen support in the relevant compiler crate(s).
   At minimum, the DOM compiler (`packages/compiler-dom/`) should handle
   the new node type.

5. **Tests** -- Write tests for each layer: schema validity, validator
   acceptance/rejection, and compiler snapshot output.

6. **Verify** -- Run the full suite before submitting:
   ```bash
   cargo test --workspace && cargo clippy --workspace -- -D warnings
   ```

7. **Commit** -- Use conventional commit messages:
   ```
   feat(validator): add FRM005 rule for phone field validation
   fix(compiler-dom): correct heading level output for nested sections
   ```

## Adding a New Compile Target

To add a new compiler (e.g., Flutter, React Native):

1. Create a new crate: `cargo new --lib packages/compiler-flutter`

2. Add it to the workspace `Cargo.toml` members list.

3. Depend on the schema crate for IR types and the validator's serde IR model
   for input deserialization.

4. Implement a `compile` function that accepts validated `VoceIr` and returns
   the target output (string, file bundle, or byte stream).

5. Follow the shared compiler patterns:
   - Zero runtime dependencies in output
   - Accessibility semantics must be preserved (never silently drop them)
   - Use `insta` for snapshot testing

6. Register the new target in the CLI's `--target` enum
   (`packages/cli/` or the relevant binary crate).

7. Add integration tests that compile the reference landing page IR and verify
   the output.

## Pull Request Process

1. Fork the repository and create a feature branch.
2. Follow the working pattern above.
3. Ensure `cargo test --workspace` and `cargo clippy --workspace -- -D warnings`
   both pass.
4. Write a clear PR description explaining what changed and why.
5. Link any related issues.

PRs that introduce new validation rules should include both positive tests
(valid IR that passes) and negative tests (invalid IR that triggers the
expected error code).

## Project Structure Reference

Key directories: `packages/schema/` (FlatBuffers schema + bindings),
`packages/validator/` (9-pass validator), `packages/compiler-*/` (7 compile
targets), `packages/ai-bridge/` (TypeScript AI layer), `packages/adapter-*/`
(4 deploy adapters), `tests/` (integration tests), `examples/` (reference IR).
