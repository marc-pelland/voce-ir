//! Application manifest — human-readable summary of what was built.
//!
//! Generated from IR analysis. Describes pages, state machines,
//! data sources, forms, accessibility features, and design decisions.

use crate::inspect;
use crate::ir::VoceIr;

/// Generate and print an application manifest from the IR.
pub fn print_manifest(file: &str, ir: &VoceIr) {
    let summary = inspect::summarize(ir);

    println!("# Application Manifest");
    println!();
    println!("Source: {file}");
    println!("Schema: v{}", summary.schema_version);
    if let Some(ref lang) = summary.language {
        println!("Language: {lang}");
    }
    println!();

    // Pages
    println!("## Pages");
    println!();
    if summary.has_routes {
        println!("  {} route(s) defined", summary.route_count);
    } else {
        println!("  Single page (no RouteMap)");
    }
    println!();

    // Node breakdown
    println!("## Structure");
    println!();
    println!("  Total nodes: {}", summary.total_nodes);
    println!("  Node types:  {}", summary.node_counts.len());

    let mut counts: Vec<_> = summary.node_counts.iter().collect();
    counts.sort_by(|a, b| b.1.cmp(a.1));
    for (type_name, count) in &counts {
        println!("    {type_name}: {count}");
    }
    println!();

    // State machines
    if !summary.state_machines.is_empty() {
        println!("## State Machines");
        println!();
        for sm in &summary.state_machines {
            println!(
                "  - {} ({} states, {} transitions)",
                sm.name, sm.state_count, sm.transition_count
            );
        }
        println!();
    }

    // Features
    println!("## Features");
    println!();
    println!(
        "  SEO metadata: {}",
        if summary.has_metadata { "yes" } else { "no" }
    );
    println!(
        "  Theme:        {}",
        if summary.has_theme { "yes" } else { "no" }
    );
    println!(
        "  i18n:         {}",
        if summary.has_i18n { "yes" } else { "no" }
    );
    println!(
        "  Routes:       {}",
        if summary.has_routes {
            format!("{}", summary.route_count)
        } else {
            "none".to_string()
        }
    );
}
