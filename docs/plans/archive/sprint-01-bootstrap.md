# Sprint 01 — Project Bootstrap

**Status:** Ready for review
**Goal:** A working Rust workspace with FlatBuffers toolchain, CI, directory structure, and the `.voce/` memory skeleton. After this sprint, `cargo build --workspace` succeeds and the project structure matches the architecture.
**Depends on:** Nothing (first sprint)

---

## Deliverables

1. Rust workspace with 3 packages (schema, validator, compiler-dom)
2. FlatBuffers compiler (`flatc`) installed and verified
3. GitHub Actions CI (build, test, lint, format)
4. Complete directory structure matching CLAUDE.md architecture
5. `.voce/` memory directory structure with templates
6. `voce` CLI skeleton (binary that prints help, no functionality yet)

---

## Tasks

### 1. Initialize Rust Workspace

**File: `Cargo.toml` (workspace root)**

**Toolchain version decision:** Rust Edition 2024 is stable and offers `async fn` in traits and improved diagnostics. Use it for forward-compatibility. If any crate has issues with edition 2024, fall back to 2021 for that crate only.

```toml
[workspace]
resolver = "2"
members = [
    "packages/schema",
    "packages/validator",
    "packages/compiler-dom",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "Apache-2.0"
repository = "https://github.com/marc-pelland/voce-ir"
rust-version = "1.85"

[workspace.dependencies]
flatbuffers = "24.12"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
thiserror = "2.0"
anyhow = "1.0"
clap = { version = "4.5", features = ["derive"] }
insta = { version = "1.40", features = ["yaml"] }
```

**Note on edition 2024:** If `flatbuffers` generated code has issues with edition 2024, the schema package can override to `edition = "2021"` in its own `Cargo.toml` while the rest of the workspace uses 2024.

**Packages to initialize:**

```bash
cargo init packages/schema --lib --name voce-schema
cargo init packages/validator --name voce-validator
cargo init packages/compiler-dom --name voce-compiler-dom
```

**Package dependencies:**

`packages/schema/Cargo.toml`:
- `flatbuffers` (workspace)
- `serde`, `serde_json` (workspace) — for JSON canonical format

`packages/validator/Cargo.toml`:
- `voce-schema = { path = "../schema" }`
- `thiserror` (workspace) — for typed validation errors
- `anyhow` (workspace) — for CLI error handling
- `clap` (workspace) — for CLI argument parsing
- `serde_json` (workspace)

`packages/compiler-dom/Cargo.toml`:
- `voce-schema = { path = "../schema" }`
- `voce-validator = { path = "../validator" }` — validate before compiling
- `anyhow`, `clap` (workspace)

### 2. Create Directory Structure

```
voce-ir/
├── Cargo.toml
├── CLAUDE.md
├── README.md
├── LICENSE
├── packages/
│   ├── schema/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs              # Re-exports, module declarations
│   │   │   └── generated/          # Auto-generated from .fbs (initially empty)
│   │   │       └── .gitkeep
│   │   └── schemas/                # FlatBuffers schema files
│   │       └── .gitkeep
│   ├── validator/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs             # CLI entry point (voce binary)
│   │       ├── lib.rs              # Validation engine
│   │       ├── passes/
│   │       │   └── mod.rs          # Pass registry
│   │       └── errors.rs           # Typed error definitions
│   └── compiler-dom/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs             # Will be a library, CLI in validator
│           └── lib.rs              # Compiler engine
├── tests/
│   ├── schema/
│   │   ├── valid/                  # Valid IR test fixtures
│   │   │   └── .gitkeep
│   │   └── invalid/                # Invalid IR test fixtures
│   │       └── .gitkeep
│   ├── compiler/
│   │   └── snapshots/
│   │       └── .gitkeep
│   └── integration/
│       └── .gitkeep
├── examples/
│   ├── intents/                    # (intent, IR) pairs for Phase 3 RAG
│   │   └── .gitkeep
│   └── landing-page/
│       └── .gitkeep
├── style-packs/                    # Design pattern libraries (Phase 3)
│   └── .gitkeep
├── docs/
│   └── (existing docs)
└── .github/
    └── workflows/
        └── ci.yml
```

### 3. Install and Verify FlatBuffers

```bash
# macOS
brew install flatbuffers

# Verify
flatc --version
# Should be 24.x

# Test: create a minimal schema, compile, verify it works
echo 'namespace voce; table Ping { message: string; }' > /tmp/test.fbs
flatc --rust -o /tmp/ /tmp/test.fbs
# Should produce /tmp/test_generated.rs
```

Document the required `flatc` version in CLAUDE.md and README.

### 4. Unified CLI Skeleton

The `voce` binary lives in the validator package (since validation is the core operation). Other subcommands (compile, deploy) will be added in later sprints.

**`packages/validator/src/main.rs`:**

```rust
use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "voce")]
#[command(about = "Voce IR — AI-native UI intermediate representation toolchain")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate an IR file
    Validate {
        /// Path to the IR file (.voce or .voce.json)
        file: std::path::PathBuf,
    },
    /// Inspect an IR file (human-readable summary)
    Inspect {
        /// Path to the IR file
        file: std::path::PathBuf,
    },
    /// Convert JSON canonical format to binary FlatBuffer
    Json2bin {
        /// Input JSON file
        input: std::path::PathBuf,
        /// Output binary file
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },
    /// Convert binary FlatBuffer to JSON canonical format
    Bin2json {
        /// Input binary file
        input: std::path::PathBuf,
        /// Output JSON file
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { file } => {
            println!("voce validate: {} (not yet implemented)", file.display());
        }
        Commands::Inspect { file } => {
            println!("voce inspect: {} (not yet implemented)", file.display());
        }
        Commands::Json2bin { input, output: _ } => {
            println!("voce json2bin: {} (not yet implemented)", input.display());
        }
        Commands::Bin2json { input, output: _ } => {
            println!("voce bin2json: {} (not yet implemented)", input.display());
        }
    }

    Ok(())
}
```

Set the binary name in `packages/validator/Cargo.toml`:
```toml
[[bin]]
name = "voce"
path = "src/main.rs"
```

### 5. GitHub Actions CI

**`.github/workflows/ci.yml`:**

```yaml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --check
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo test --workspace
      - run: cargo build --release --workspace
```

### 6. `.voce/` Memory Structure Templates

Create the memory directory structure with template files:

```
.voce/
├── brief.yaml.template          # Project brief template
├── decisions/
│   └── .gitkeep
├── sessions/
│   └── .gitkeep
├── memory/
│   └── .gitkeep
└── snapshots/
    └── .gitkeep
```

**`.voce/brief.yaml.template`:**
```yaml
# Voce IR Project Brief
# Copy this to brief.yaml and fill in your project details.
# The AI uses this as the "north star" for all decisions.

project:
  name: ""
  version: 1
  
vision: ""

target_audience:
  primary: ""
  secondary: ""

success_criteria: []

non_negotiables: []

out_of_scope: []

style_direction:
  references: []
  feel: ""

technical_decisions:
  backend: ""
  auth: ""
  hosting: ""

features: []
```

### 7. Initial lib.rs Files

**`packages/schema/src/lib.rs`:**
```rust
//! Voce IR Schema — FlatBuffers type definitions and generated bindings.
//!
//! This crate contains the IR schema definitions and the Rust bindings
//! generated from FlatBuffers `.fbs` files.

// Generated bindings will be added in Sprint 02
// pub mod generated;
```

**`packages/validator/src/lib.rs`:**
```rust
//! Voce IR Validator — Reference IR validation engine.
//!
//! Validates IR blobs against structural, accessibility, security,
//! and other quality rules. Returns typed errors with node paths.

pub mod errors;
pub mod passes;
```

**`packages/validator/src/errors.rs`:**
```rust
//! Typed validation error definitions.

use thiserror::Error;

/// Severity level for validation results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Blocks compilation. Must be fixed.
    Error,
    /// Reported but compilation proceeds.
    Warning,
}

/// A single validation diagnostic.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity level
    pub severity: Severity,
    /// Error code (e.g., "A11Y001", "SEC003")
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Path to the offending node (e.g., "/children/2/semantic")
    pub node_path: String,
    /// Validation pass that produced this diagnostic
    pub pass: String,
}

/// Result of running all validation passes.
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub diagnostics: Vec<Diagnostic>,
}

impl ValidationResult {
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Error).count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Warning).count()
    }
}
```

**`packages/validator/src/passes/mod.rs`:**
```rust
//! Validation passes. Each pass checks one category of rules.
//!
//! Passes will be added in Sprints 06-07:
//! - structural: completeness, required fields
//! - types: type checking
//! - references: Ref<T> resolution
//! - state: state machine validation
//! - a11y: accessibility enforcement
//! - security: OWASP-informed checks
//! - seo: meta tag, heading hierarchy
//! - forms: label, ARIA, validation rules
//! - i18n: translation completeness
//! - motion: ReducedMotion coverage
```

**`packages/compiler-dom/src/lib.rs`:**
```rust
//! Voce IR DOM Compiler — Compiles validated IR to HTML output.
//!
//! Takes validated IR + device profile and emits minimal,
//! zero-framework HTML with inline styles, surgical JS, and
//! ARIA attributes.

// Compiler implementation starts in Sprint 11
```

---

## Acceptance Criteria

Before moving to Sprint 02, verify:

- [ ] `cargo build --workspace` succeeds with zero warnings
- [ ] `cargo test --workspace` succeeds (even with no tests yet)
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo fmt --check` passes
- [ ] `cargo run -p voce-validator -- validate --help` prints usage
- [ ] `flatc --version` returns 24.x
- [ ] CI workflow runs (push to GitHub to verify)
- [ ] Directory structure matches the layout above
- [ ] `.voce/brief.yaml.template` exists
- [ ] All `lib.rs` files have doc comments

---

## Toolchain Versions (Verified April 2026)

| Tool | Version | Notes |
| ---- | ------- | ----- |
| Rust | 1.85+ stable, edition 2024 | Edition 2024 has `async fn` in traits, improved diagnostics |
| Cargo | (bundled with Rust) | Workspace dependency inheritance, resolver v2 |
| FlatBuffers (`flatc`) | 24.x (date-based versioning) | Rust codegen functional but rougher than C++. May need edition 2021 per-crate |
| Clap | 4.5+ | Derive API stable and mature |
| Serde | 1.x | Undisputed standard, no challengers |
| Thiserror | 2.x | Recent major version — verify API |
| Insta | 1.40+ | Snapshot testing, YAML output |
| GitHub Actions | ubuntu-latest (24.04), `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2` | |

**Forward-thinking principle:** Use the latest stable versions. Don't use nightly or experimental features. If a new major version just released (< 2 weeks), wait for ecosystem settling before adopting. Verify each dependency compiles with edition 2024 during setup.

**Not needed yet (defer until required):**
- Tokio — the validator and compiler are CPU-bound, synchronous. Add async only when the preview server or AI bridge needs it (Phase 2-3)
- TypeScript 6 — evaluate once ecosystem support (ESLint, bundlers) stabilizes. Use 5.x for Phase 3 initial work if 6 is too new
- `rkyv` — zero-copy alternative to FlatBuffers, worth knowing about but not switching

## Notes

- The `voce` binary name may conflict with other packages — check with `which voce` before installing globally. The binary is in `target/release/voce` for local use.
- FlatBuffers 24.x is required. Earlier versions have different Rust codegen.
- The workspace uses resolver = "2" (required for edition 2021).
