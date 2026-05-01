//! Output formatting for validation results.
//!
//! Supports two modes: colored terminal output (default) and
//! machine-readable JSON (for CI/tooling integration).

use colored::Colorize;

use crate::errors::{Diagnostic, FixPatch, Severity, ValidationResult};
use crate::fixes::build_fix;

/// Canonical docs URL pattern for a validator rule. Pages may not exist yet
/// for every code; the URL is stable so deep-links continue to work as the
/// docs site grows. S67 Day 5.
pub const DOCS_BASE: &str = "https://voce-ir.xyz/docs/validator-rules";

/// Build the docs URL for a diagnostic code (e.g. "STR002" → ".../STR002").
pub fn docs_url(code: &str) -> String {
    format!("{DOCS_BASE}/{code}")
}

/// Project a Diagnostic to JSON, including the auto-fix proposal when one
/// is available for that code (S67 Day 4). Centralizing here keeps the two
/// JSON output paths in sync.
fn diagnostic_to_json(d: &Diagnostic) -> serde_json::Value {
    let fix_value = build_fix(d).map(fix_to_json);
    serde_json::json!({
        "severity": d.severity.to_string(),
        "code": d.code,
        "message": d.message,
        "path": d.node_path,
        "pass": d.pass,
        "hint": d.hint,
        "docs_url": docs_url(&d.code),
        "fix": fix_value,
    })
}

fn fix_to_json(fix: FixPatch) -> serde_json::Value {
    let ops: Vec<serde_json::Value> = fix
        .operations
        .into_iter()
        .map(|op| {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "op".to_string(),
                serde_json::Value::String(op.op.to_string()),
            );
            obj.insert("path".to_string(), serde_json::Value::String(op.path));
            if let Some(v) = op.value {
                obj.insert("value".to_string(), v);
            }
            serde_json::Value::Object(obj)
        })
        .collect();
    serde_json::json!({
        "type": "json-patch",
        "confidence": fix.confidence.to_string(),
        "operations": ops,
        "preview": fix.preview,
    })
}

/// Print validation result as colored terminal output.
pub fn print_terminal(file: &str, result: &ValidationResult) {
    if result.diagnostics.is_empty() {
        println!("{} {file}", "✓".green().bold());
        println!("  {} issues", "0".green());
        return;
    }

    println!("{file}");
    println!();

    for diag in &result.diagnostics {
        let severity_label = match diag.severity {
            Severity::Error => "ERROR".red().bold(),
            Severity::Warning => " WARN".yellow().bold(),
        };

        println!(
            "  {severity_label}  {}  {}",
            diag.code.dimmed(),
            diag.message
        );
        println!("         {} {}", "at".dimmed(), diag.node_path.dimmed());
        println!("         {} {}", "in".dimmed(), diag.pass.dimmed());
        if let Some(ref hint) = diag.hint {
            println!("         {} {}", "fix:".cyan(), hint);
        }
        println!();
    }

    let errors = result.error_count();
    let warnings = result.warning_count();
    let total = errors + warnings;

    let summary = if errors > 0 {
        format!(
            "{total} issues ({errors} error{}, {warnings} warning{})",
            if errors != 1 { "s" } else { "" },
            if warnings != 1 { "s" } else { "" },
        )
        .red()
        .bold()
        .to_string()
    } else {
        format!(
            "{total} issues ({warnings} warning{})",
            if warnings != 1 { "s" } else { "" },
        )
        .yellow()
        .to_string()
    };

    println!("  {summary}");
}

/// Print validation result as JSON to stdout.
pub fn print_json(file: &str, result: &ValidationResult) -> Result<(), serde_json::Error> {
    let output = serde_json::json!({
        "file": file,
        "valid": !result.has_errors(),
        "errors": result.error_count(),
        "warnings": result.warning_count(),
        "diagnostics": result.diagnostics.iter().map(diagnostic_to_json).collect::<Vec<_>>(),
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

/// Print validation result as JSON with per-pass execution metadata.
/// Used by `voce validate --format json --verbose-passes`. Output is a
/// superset of `print_json` — adds a `passes` array describing each pass's
/// timing, error/warning counts, and the distinct codes it emitted.
pub fn print_json_verbose(file: &str, result: &ValidationResult) -> Result<(), serde_json::Error> {
    let passes: Vec<_> = result
        .passes
        .iter()
        .map(|p| {
            serde_json::json!({
                "name": p.name,
                "duration_us": p.duration_us,
                "errors": p.error_count,
                "warnings": p.warning_count,
                "codes": p.codes,
            })
        })
        .collect();

    let output = serde_json::json!({
        "file": file,
        "valid": !result.has_errors(),
        "errors": result.error_count(),
        "warnings": result.warning_count(),
        "diagnostics": result.diagnostics.iter().map(diagnostic_to_json).collect::<Vec<_>>(),
        "passes": passes,
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
