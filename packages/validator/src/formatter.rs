//! Output formatting for validation results.
//!
//! Supports two modes: colored terminal output (default) and
//! machine-readable JSON (for CI/tooling integration).

use colored::Colorize;

use crate::errors::{Severity, ValidationResult};

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
        "diagnostics": result.diagnostics.iter().map(|d| {
            serde_json::json!({
                "severity": d.severity.to_string(),
                "code": d.code,
                "message": d.message,
                "path": d.node_path,
                "pass": d.pass,
            })
        }).collect::<Vec<_>>(),
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
