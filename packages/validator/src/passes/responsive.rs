//! Responsive/reflow validation pass.
//!
//! Warns when a layout is likely to overflow small screens (WCAG 1.4.10
//! reflow risk): a large fixed pixel `width` with no `max_width` and no
//! ResponsiveRule override to reduce it at a breakpoint. Warning-only — the
//! compiler emits fluid defaults that mitigate most cases, so this is a nudge,
//! not a hard failure.

use std::collections::HashSet;

use crate::errors::{CodeMeta, Diagnostic, Severity, ValidationResult};
use crate::index::NodeIndex;
use crate::ir::{ChildNode, VoceIr};
use crate::passes::ValidationPass;

pub struct ResponsivePass;

/// Fixed widths at or below this fit a small phone (with room to spare), so we
/// don't warn on them. Above it, a fixed px width will overflow narrow screens.
const OVERFLOW_THRESHOLD_PX: f64 = 600.0;

const CODES: &[CodeMeta] = &[CodeMeta {
    code: "RSP001",
    summary: "Fixed pixel width may overflow small screens",
    hint: "A large fixed `width` (in Px) with no `max_width` and no ResponsiveRule \
           override will overflow narrow viewports (WCAG 1.4.10). Use a percentage / \
           fluid width, add a `max_width`, or add a ResponsiveRule that reduces the \
           width at a small breakpoint.",
    fix_confidence: None,
}];

impl ValidationPass for ResponsivePass {
    fn name(&self) -> &'static str {
        "responsive"
    }

    fn codes(&self) -> &'static [CodeMeta] {
        CODES
    }

    fn run(&self, ir: &VoceIr, _index: &NodeIndex, result: &mut ValidationResult) {
        let Some(root) = &ir.root else {
            return;
        };
        let Some(children) = &root.children else {
            return;
        };
        // Nodes an author already made responsive via a ResponsiveRule override
        // are exempt from the fixed-width warning.
        let mut handled: HashSet<String> = HashSet::new();
        collect_responsive_targets(children, &mut handled);
        check_children(children, "/root/children", &handled, result);
    }
}

/// Collect every node id targeted by a ResponsiveRule override.
fn collect_responsive_targets(children: &[ChildNode], handled: &mut HashSet<String>) {
    for c in children {
        if c.type_name() == "ResponsiveRule" {
            if let Some(sets) = c
                .value
                .get("responsive_overrides")
                .and_then(|v| v.as_array())
            {
                for set in sets {
                    if let Some(ovs) = set.get("overrides").and_then(|v| v.as_array()) {
                        for ov in ovs {
                            if let Some(t) = ov.get("target_node_id").and_then(|v| v.as_str()) {
                                handled.insert(t.to_string());
                            }
                        }
                    }
                }
            }
        }
        if let Some(grandchildren) = c.children() {
            collect_responsive_targets(&grandchildren, handled);
        }
    }
}

fn check_children(
    children: &[ChildNode],
    path: &str,
    handled: &HashSet<String>,
    result: &mut ValidationResult,
) {
    for (i, c) in children.iter().enumerate() {
        let node_path = format!("{path}/{i}");
        if matches!(c.type_name(), "Container" | "Surface") {
            if let Some(width) = c.value.get("width") {
                let unit = width.get("unit").and_then(|v| v.as_str()).unwrap_or("Px");
                let value = width.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let has_max = c.value.get("max_width").is_some();
                let id = c
                    .value
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if unit == "Px"
                    && value > OVERFLOW_THRESHOLD_PX
                    && !has_max
                    && !handled.contains(id)
                {
                    result.diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        code: "RSP001".to_string(),
                        message: format!(
                            "Node \"{id}\" has a fixed width of {value:.0}px with no max_width or \
                             responsive override; it may overflow small screens"
                        ),
                        node_path: node_path.clone(),
                        pass: "responsive".to_string(),
                        hint: None,
                    });
                }
            }
        }
        if let Some(grandchildren) = c.children() {
            check_children(&grandchildren, &node_path, handled, result);
        }
    }
}
