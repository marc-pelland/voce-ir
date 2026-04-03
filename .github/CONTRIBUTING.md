# Contributing to Voce IR

Thank you for your interest in contributing to Voce IR. This document covers the process for contributing to the project.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/voce-ir.git`
3. Install prerequisites: Rust (stable), FlatBuffers compiler (`flatc`), Node.js 20+
4. Build: `cargo build --workspace`
5. Test: `cargo test --workspace`

## Development Workflow

1. Create a branch from `main`: `git checkout -b feat/your-feature`
2. Make your changes following the conventions in `CLAUDE.md`
3. Run the full check suite:
   ```bash
   cargo fmt --check
   cargo clippy --workspace -- -D warnings
   cargo test --workspace
   ```
4. Commit with a clear message: `feat(validator): add keyboard equivalence check for GestureHandler`
5. Push and open a Pull Request

## Commit Message Format

```
type(scope): description

feat(schema):     New schema types or fields
feat(validator):  New validation passes or rules
feat(compiler):   New compiler features or codegen
fix(scope):       Bug fixes
docs:             Documentation changes
test:             Test additions or fixes
refactor(scope):  Code restructuring without behavior change
```

## Adding a New IR Node Type

1. Define the type in the appropriate `.fbs` schema file in `packages/schema/schemas/`
2. Regenerate bindings: `flatc --rust -o packages/schema/src/generated/ packages/schema/schemas/*.fbs`
3. Add validation logic in `packages/validator/src/passes/`
4. Add compiler support in `packages/compiler-dom/src/codegen/`
5. Write test cases in `tests/schema/valid/` and `tests/schema/invalid/`
6. Add a compiler snapshot test in `tests/compiler/snapshots/`
7. Update documentation if the node type is user-facing

## Adding a New Compile Target

New compile targets are separate packages in the `packages/` directory. Create a new package:

```bash
cargo init packages/compiler-your-target
```

Your compiler must:
- Accept a validated IR blob (use `packages/schema` for deserialization)
- Accept a DeviceProfile
- Emit output appropriate to the target
- Pass the integration test suite in `tests/integration/`

## Code of Conduct

We are committed to providing a welcoming and inclusive experience for everyone. Please be respectful and constructive in all interactions.

## License

By contributing, you agree that your contributions will be licensed under the Apache 2.0 License.
