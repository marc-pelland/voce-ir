//! Report generation — compilation quality metrics.
//!
//! Combines validation results, compilation output analysis, and IR
//! summary into a single human-readable or JSON report.

use crate::errors::ValidationResult;
use crate::inspect;
use crate::ir::VoceIr;

/// A full compilation report.
#[derive(Debug)]
pub struct CompilationReport {
    pub file: String,
    pub validation_errors: usize,
    pub validation_warnings: usize,
    pub total_nodes: usize,
    pub node_types: usize,
    pub state_machines: usize,
    pub has_metadata: bool,
    pub has_theme: bool,
    pub has_i18n: bool,
    pub compiled_size: Option<usize>,
}

/// Generate a report from IR and validation results.
pub fn generate_report(
    file: &str,
    ir: &VoceIr,
    validation: &ValidationResult,
    compiled_size: Option<usize>,
) -> CompilationReport {
    let summary = inspect::summarize(ir);

    CompilationReport {
        file: file.to_string(),
        validation_errors: validation.error_count(),
        validation_warnings: validation.warning_count(),
        total_nodes: summary.total_nodes,
        node_types: summary.node_counts.len(),
        state_machines: summary.state_machines.len(),
        has_metadata: summary.has_metadata,
        has_theme: summary.has_theme,
        has_i18n: summary.has_i18n,
        compiled_size,
    }
}

/// Print the report to stdout.
pub fn print_report(report: &CompilationReport) {
    println!("voce report: {}", report.file);
    println!();
    println!("  Validation:");
    println!("    Errors:   {}", report.validation_errors);
    println!("    Warnings: {}", report.validation_warnings);
    let status = if report.validation_errors == 0 {
        "PASS"
    } else {
        "FAIL"
    };
    println!("    Status:   {status}");
    println!();
    println!("  Structure:");
    println!("    Nodes:          {}", report.total_nodes);
    println!("    Node types:     {}", report.node_types);
    println!("    State machines: {}", report.state_machines);
    println!();
    println!("  Features:");
    println!(
        "    SEO metadata: {}",
        if report.has_metadata { "yes" } else { "no" }
    );
    println!(
        "    Theme:        {}",
        if report.has_theme { "yes" } else { "no" }
    );
    println!(
        "    i18n:         {}",
        if report.has_i18n { "yes" } else { "no" }
    );

    if let Some(size) = report.compiled_size {
        println!();
        println!("  Output:");
        println!("    Size: {} bytes ({:.1} KB)", size, size as f64 / 1024.0);
        let under_target = size < 10_000;
        println!(
            "    <10KB target: {}",
            if under_target { "PASS" } else { "FAIL" }
        );
    }
}

/// Print report as JSON.
pub fn print_report_json(report: &CompilationReport) {
    let json = serde_json::json!({
        "file": report.file,
        "validation": {
            "errors": report.validation_errors,
            "warnings": report.validation_warnings,
            "status": if report.validation_errors == 0 { "pass" } else { "fail" },
        },
        "structure": {
            "total_nodes": report.total_nodes,
            "node_types": report.node_types,
            "state_machines": report.state_machines,
        },
        "features": {
            "seo_metadata": report.has_metadata,
            "theme": report.has_theme,
            "i18n": report.has_i18n,
        },
        "output": {
            "size_bytes": report.compiled_size,
            "under_10kb": report.compiled_size.map(|s| s < 10_000),
        },
    });
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}
