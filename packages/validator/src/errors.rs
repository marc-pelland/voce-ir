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

/// Result of running all validation passes on an IR blob.
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// All diagnostics from all passes.
    pub diagnostics: Vec<Diagnostic>,
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
