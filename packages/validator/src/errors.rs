//! Typed validation error definitions.
//!
//! Every validation diagnostic includes a severity, error code, human-readable
//! message, node path, and the pass that produced it. Error codes are prefixed
//! by category (e.g., `A11Y001`, `SEC003`, `SEO002`).

use std::fmt;

/// Severity level for validation results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Blocks compilation. Must be fixed.
    Error,
    /// Reported but compilation proceeds.
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
        }
    }
}

/// A single validation diagnostic.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity level.
    pub severity: Severity,
    /// Error code (e.g., "A11Y001", "SEC003", "STR001").
    pub code: String,
    /// Human-readable message describing the issue and how to fix it.
    pub message: String,
    /// Path to the offending node (e.g., "/root/children/2/semantic").
    pub node_path: String,
    /// Validation pass that produced this diagnostic.
    pub pass: String,
    /// Optional fix suggestion.
    pub hint: Option<String>,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] {}: {} (at {})",
            self.severity, self.pass, self.code, self.message, self.node_path
        )
    }
}

/// Static metadata for a single diagnostic code. Each pass declares the codes
/// it can emit so consumers (CLI `--list-codes`, MCP tool descriptions, the
/// docs site) can enumerate the rule catalogue without parsing source files.
#[derive(Debug, Clone, Copy)]
pub struct CodeMeta {
    /// Stable diagnostic code (e.g., "STR001"). Matches `Diagnostic.code`.
    pub code: &'static str,
    /// One-line summary of what the rule checks. Plain English, ≤ 120 chars.
    pub summary: &'static str,
    /// Action-oriented next step. Names specific schema fields where possible.
    /// Surfaced on every emitted Diagnostic via the engine's hint-injection
    /// step. Plain English, target ≤ 280 chars.
    pub hint: &'static str,
}

/// Per-pass execution metadata. Populated by the engine for every pass that
/// runs. Surfaced via `voce validate --verbose-passes` so consumers (the
/// site-hero visualization, the MCP server, the AI bridge) can show real
/// per-pass timing and outcome instead of synthesizing it.
#[derive(Debug, Clone)]
pub struct PassResult {
    /// Pass name as returned by `ValidationPass::name()`.
    pub name: String,
    /// Wall-clock time the pass took, in microseconds.
    pub duration_us: u64,
    /// Number of Error-severity diagnostics this pass emitted.
    pub error_count: usize,
    /// Number of Warning-severity diagnostics this pass emitted.
    pub warning_count: usize,
    /// Distinct error codes this pass emitted (deduplicated, in order seen).
    pub codes: Vec<String>,
}

/// Result of running all validation passes on an IR blob.
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// All diagnostics from all passes.
    pub diagnostics: Vec<Diagnostic>,
    /// Per-pass execution metadata, in execution order. Empty if the engine
    /// didn't record it (e.g., older callers using `validate()` directly).
    pub passes: Vec<PassResult>,
}

impl ValidationResult {
    /// Returns true if any diagnostic has Error severity.
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }

    /// Count of Error-severity diagnostics.
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    /// Count of Warning-severity diagnostics.
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }
}
