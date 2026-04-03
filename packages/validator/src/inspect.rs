//! IR inspection — human-readable summary of an IR file.
//!
//! Does not run validation passes. Only reads and summarizes structure.

use std::collections::HashMap;

use crate::ir::{ChildNode, VoceIr};

/// Summary of an IR document.
pub struct IrSummary {
    pub schema_version: String,
    pub language: Option<String>,
    pub total_nodes: usize,
    pub max_depth: usize,
    pub node_counts: HashMap<String, usize>,
    pub state_machines: Vec<StateMachineSummary>,
    pub has_routes: bool,
    pub route_count: usize,
    pub has_i18n: bool,
    pub has_theme: bool,
    pub has_metadata: bool,
}

pub struct StateMachineSummary {
    pub name: String,
    pub state_count: usize,
    pub transition_count: usize,
}

/// Build a summary from a deserialized IR document.
pub fn summarize(ir: &VoceIr) -> IrSummary {
    let mut summary = IrSummary {
        schema_version: format!("{}.{}", ir.schema_version_major, ir.schema_version_minor),
        language: None,
        total_nodes: 0,
        max_depth: 0,
        node_counts: HashMap::new(),
        state_machines: Vec::new(),
        has_routes: ir.routes.is_some(),
        route_count: ir
            .routes
            .as_ref()
            .and_then(|r| r.routes.as_ref())
            .map_or(0, |r| r.len()),
        has_i18n: ir.i18n.is_some(),
        has_theme: ir.theme.is_some(),
        has_metadata: false,
    };

    if let Some(ref root) = ir.root {
        summary.total_nodes += 1; // ViewRoot
        *summary
            .node_counts
            .entry("ViewRoot".to_string())
            .or_default() += 1;
        summary.language = root.document_language.clone();
        summary.has_metadata = root.metadata.is_some();

        if let Some(ref sems) = root.semantic_nodes {
            summary.total_nodes += sems.len();
            *summary
                .node_counts
                .entry("SemanticNode".to_string())
                .or_default() += sems.len();
        }

        if let Some(ref children) = root.children {
            count_nodes(children, 1, &mut summary);
        }
    }

    summary
}

fn count_nodes(children: &[ChildNode], depth: usize, summary: &mut IrSummary) {
    if depth > summary.max_depth {
        summary.max_depth = depth;
    }

    for child in children {
        summary.total_nodes += 1;
        let type_name = child.type_name().to_string();
        *summary.node_counts.entry(type_name.clone()).or_default() += 1;

        // Collect StateMachine details
        if type_name == "StateMachine" {
            if let Some(sm) = child.as_type::<crate::ir::StateMachine>() {
                summary.state_machines.push(StateMachineSummary {
                    name: sm
                        .name
                        .or(sm.node_id)
                        .unwrap_or_else(|| "<unnamed>".to_string()),
                    state_count: sm.states.as_ref().map_or(0, |s| s.len()),
                    transition_count: sm.transitions.as_ref().map_or(0, |t| t.len()),
                });
            }
        }

        if let Some(grandchildren) = child.children() {
            count_nodes(&grandchildren, depth + 1, summary);
        }
    }
}

/// Print the summary to stdout.
pub fn print_summary(file: &str, summary: &IrSummary) {
    println!("voce inspect: {file}");
    println!();
    println!("  Schema:     v{}", summary.schema_version);
    if let Some(ref lang) = summary.language {
        println!("  Language:   {lang}");
    }
    println!("  Nodes:      {} total", summary.total_nodes);
    println!("  Tree depth: {} levels", summary.max_depth);
    println!();

    // Node type counts (sorted by count descending)
    let mut counts: Vec<_> = summary.node_counts.iter().collect();
    counts.sort_by(|a, b| b.1.cmp(a.1));

    println!("  Node types:");
    for (type_name, count) in &counts {
        println!("    {type_name:<24} {count}");
    }
    println!();

    // State machines
    if !summary.state_machines.is_empty() {
        println!("  State machines: {}", summary.state_machines.len());
        for sm in &summary.state_machines {
            println!(
                "    - {} ({} states, {} transitions)",
                sm.name, sm.state_count, sm.transition_count
            );
        }
        println!();
    }

    // Features
    println!(
        "  Routes:   {}",
        if summary.has_routes {
            format!("{}", summary.route_count)
        } else {
            "none".to_string()
        }
    );
    println!(
        "  i18n:     {}",
        if summary.has_i18n {
            "configured"
        } else {
            "not used"
        }
    );
    println!(
        "  Theme:    {}",
        if summary.has_theme { "yes" } else { "none" }
    );
    println!(
        "  Metadata: {}",
        if summary.has_metadata { "yes" } else { "none" }
    );
}
