# Contributing to Voce IR

Thank you for your interest in contributing to Voce IR! This guide covers everything you need to get started.

## Development Setup

### Prerequisites

- **Rust 1.85+** (edition 2024 support)
- **flatc** (FlatBuffers compiler) -- `brew install flatbuffers` or [build from source](https://flatbuffers.dev/)
- **Node.js 20+** (for TypeScript packages)
- **wasm-pack** (for playground WASM builds) -- `cargo install wasm-pack`

### Building

```bash
git clone https://github.com/marc-pelland/voce-ir.git
cd voce-ir

# Build all Rust crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Lint (must pass with zero warnings)
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --check
```

### TypeScript packages

```bash
cd packages/ai-bridge && npm install && npm run build
cd packages/inspector && npm install && npm run build
```

## Code Conventions

### Rust

- **Edition 2024** (latest stable). Exception: schema crate uses edition 2021 for FlatBuffers codegen compatibility.
- **Error handling:** `thiserror` for library errors with typed variants, `anyhow` for CLI entry points only.
- **CLI args:** `clap` derive API.
- **No `unwrap()` in library code** -- propagate errors with `?`.
- **Naming:** `snake_case` for files/functions, `PascalCase` for types/enums.
- **Every public function and type** has a `///` doc comment.
- **Format:** `cargo fmt` (default settings).
- **Lint:** `cargo clippy -- -D warnings` (zero warnings policy).

### Testing

- Unit tests: in-file `#[cfg(test)]` modules.
- Integration tests: `packages/validator/tests/`.
- Snapshot tests: `insta` crate for compiled output snapshots.
- Benchmarks: `criterion` crate in `benches/` directories.
- Every new node type needs valid + invalid test fixtures in `tests/`.

### Commit Messages

```
feat(scope): description
fix(scope): description
test(scope): description
docs(scope): description
refactor(scope): description
```

Scopes: `schema`, `validator`, `compiler-dom`, `compiler-*`, `adapter-*`, `ai-bridge`, `inspector`, `playground`, `cli`, `docs`

## Working Pattern

When implementing a new feature, follow this order:

1. **Schema** -- Add/modify `.fbs` files in `packages/schema/schemas/` (if new types needed)
2. **Regenerate** -- Run `./scripts/regenerate-schema.sh`
3. **Validate** -- Add a validation pass in `packages/validator/src/passes/`
4. **Compile** -- Add compiler support in the relevant `packages/compiler-*/`
5. **Test** -- Write tests: valid IR fixture, invalid IR fixture, compiler snapshot
6. **Verify** -- `cargo test --workspace && cargo clippy --workspace -- -D warnings`

## Adding a New Compile Target

1. Create a new crate: `packages/compiler-<target>/`
2. Add it to the workspace `Cargo.toml` members list
3. Implement a `compile_<target>(json: &str) -> Result<TargetResult>` function
4. Add integration tests in `packages/validator/tests/cross_target_tests.rs`
5. Document in `docs/site/src/architecture/compilers.md`

## Adding a New Validation Rule

1. Choose the appropriate pass in `packages/validator/src/passes/`
2. Add the check with a new error code following the prefix convention (e.g., `A11Y006`)
3. Add an invalid fixture in `tests/schema/invalid/` that triggers the error
4. Add a test in `packages/validator/tests/validation_tests.rs`
5. Document the rule in `docs/site/src/architecture/validation.md`

## Pull Request Process

1. Fork and create a feature branch
2. Make your changes
3. Ensure all checks pass: `cargo test --workspace && cargo clippy --workspace -- -D warnings && cargo fmt --check`
4. Write a clear PR description explaining what and why
5. Submit the PR

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md).

## Questions?

Open a [discussion](https://github.com/marc-pelland/voce-ir/discussions) or [issue](https://github.com/marc-pelland/voce-ir/issues).
