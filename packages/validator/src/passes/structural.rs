//! Structural completeness pass.
//!
//! Checks that required fields are present, node IDs are unique,
//! and the basic tree structure is valid.

use crate::errors::{Diagnostic, Severity, ValidationResult};
use crate::index::NodeIndex;
use crate::ir::{ChildNode, VoceIr};
use crate::passes::ValidationPass;

pub struct StructuralPass;

impl ValidationPass for StructuralPass {
    fn name(&self) -> &'static str {
        "structural"
    }

    fn run(&self, ir: &VoceIr, index: &NodeIndex, result: &mut ValidationResult) {
        // STR001: ViewRoot must exist
        if ir.root.is_none() {
            result.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                code: "STR001".to_string(),
                message: "Document is missing a root ViewRoot".to_string(),
                node_path: "/".to_string(),
                pass: self.name().to_string(),
                hint: None,
            });
            return; // Can't continue without a root
        }

        let root = ir.root.as_ref().unwrap();

        // STR001b: ViewRoot must have a node_id
        if root.node_id.is_none() || root.node_id.as_deref() == Some("") {
            result.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                code: "STR001".to_string(),
                message: "ViewRoot is missing a node_id".to_string(),
                node_path: "/root".to_string(),
                pass: self.name().to_string(),
                hint: None,
            });
        }

        // STR002: Duplicate node IDs
        for (id, path1, path2) in &index.duplicates {
            result.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                code: "STR002".to_string(),
                message: format!("Duplicate node_id \"{id}\" found at {path1} and {path2}"),
                node_path: path2.clone(),
                pass: self.name().to_string(),
                hint: None,
            });
        }

        // Walk children for per-node structural checks
        if let Some(ref children) = root.children {
            self.check_children(children, "/root/children", result);
        }
    }
}

impl StructuralPass {
    fn check_children(
        &self,
        children: &[ChildNode],
        parent_path: &str,
        result: &mut ValidationResult,
    ) {
        for (i, child) in children.iter().enumerate() {
            let path = format!("{parent_path}/{i}");

            // Check for missing node_id
            if child.node_id().is_none() {
                result.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    code: "STR002".to_string(),
                    message: format!("{} node is missing a node_id", child.type_name()),
                    node_path: path.clone(),
                    pass: "structural".to_string(),
                    hint: None,
                });
            }

            // STR004: TextNode must have content
            if child.type_name() == "TextNode" {
                if let Some(text) = child.as_type::<crate::ir::TextNode>() {
                    let has_content = text.content.as_ref().is_some_and(|c| !c.is_empty());
                    let has_binding = text.content_binding.is_some();
                    let has_localized = text.localized_content.is_some();

                    if !has_content && !has_binding && !has_localized {
                        result.diagnostics.push(Diagnostic {
                            severity: Severity::Error,
                            code: "STR004".to_string(),
                            message:
                                "TextNode has no content, content_binding, or localized_content"
                                    .to_string(),
                            node_path: path.clone(),
                            pass: "structural".to_string(),
                            hint: None,
                        });
                    }
                }
            }

            // STR005: MediaNode must have src
            if child.type_name() == "MediaNode" {
                if let Some(media) = child.as_type::<crate::ir::MediaNode>() {
                    if media.src.as_ref().is_none_or(|s| s.is_empty()) {
                        result.diagnostics.push(Diagnostic {
                            severity: Severity::Error,
                            code: "STR005".to_string(),
                            message: "MediaNode is missing src".to_string(),
                            node_path: path.clone(),
                            pass: "structural".to_string(),
                            hint: None,
                        });
                    }
                }
            }

            // Recurse into children
            if let Some(grandchildren) = child.children() {
                self.check_children(&grandchildren, &format!("{path}/children"), result);
            }
        }
    }
}
