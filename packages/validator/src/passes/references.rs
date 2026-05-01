//! Reference resolution pass.
//!
//! Validates that all cross-node references (string IDs) point to
//! existing nodes in the IR tree.

use crate::errors::{CodeMeta, Diagnostic, Severity, ValidationResult};
use crate::index::NodeIndex;
use crate::ir::{ChildNode, VoceIr};
use crate::passes::ValidationPass;

pub struct ReferencesPass;

const CODES: &[CodeMeta] = &[
    CodeMeta {
        code: "REF001",
        summary: "Referenced node_id does not exist in this document",
    },
    CodeMeta {
        code: "REF005",
        summary: "Reference target type does not match the expected type",
    },
    CodeMeta {
        code: "REF006",
        summary: "Reference target was found but is not reachable from root",
    },
    CodeMeta {
        code: "REF007",
        summary: "Cyclic reference detected between nodes",
    },
    CodeMeta {
        code: "REF008",
        summary: "ContextNode references an undefined provider",
    },
    CodeMeta {
        code: "REF009",
        summary: "DataBinding references an unknown node or field",
    },
];

impl ValidationPass for ReferencesPass {
    fn name(&self) -> &'static str {
        "references"
    }

    fn codes(&self) -> &'static [CodeMeta] {
        CODES
    }

    fn run(&self, ir: &VoceIr, index: &NodeIndex, result: &mut ValidationResult) {
        let root = match &ir.root {
            Some(r) => r,
            None => return,
        };

        // Check semantic_node_id references on the root's children
        if let Some(ref children) = root.children {
            self.check_children(children, "/root/children", index, result);
        }

        // Semantic nodes are referenced by visual nodes via semantic_node_id
        // (checked in the children walk above). No outbound refs to check here.
    }
}

impl ReferencesPass {
    fn check_children(
        &self,
        children: &[ChildNode],
        parent_path: &str,
        index: &NodeIndex,
        result: &mut ValidationResult,
    ) {
        for (i, child) in children.iter().enumerate() {
            let path = format!("{parent_path}/{i}");

            // REF001: semantic_node_id must resolve
            if let Some(ref sem_id) = child.semantic_node_id() {
                if !sem_id.is_empty() && !index.contains(sem_id) {
                    result.diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        code: "REF001".to_string(),
                        message: format!(
                            "semantic_node_id \"{sem_id}\" does not reference an existing node"
                        ),
                        node_path: path.clone(),
                        pass: self.name().to_string(),
                        hint: None,
                    });
                }
            }

            // Type-specific reference checks
            match child.type_name() {
                "AnimationTransition" => {
                    if let Some(anim) = child.as_type::<crate::ir::AnimationTransition>() {
                        self.check_ref(
                            &anim.target_node_id,
                            "target_node_id",
                            "REF005",
                            &path,
                            index,
                            result,
                        );
                    }
                }
                "GestureHandler" => {
                    if let Some(gh) = child.as_type::<crate::ir::GestureHandler>() {
                        self.check_ref(
                            &gh.target_node_id,
                            "target_node_id",
                            "REF005",
                            &path,
                            index,
                            result,
                        );
                    }
                }
                "ScrollBinding" => {
                    if let Some(sb) = child.as_type::<crate::ir::ScrollBinding>() {
                        self.check_ref(
                            &sb.target_node_id,
                            "target_node_id",
                            "REF006",
                            &path,
                            index,
                            result,
                        );
                    }
                }
                "PhysicsBody" => {
                    if let Some(pb) = child.as_type::<crate::ir::PhysicsBody>() {
                        self.check_ref(
                            &pb.target_node_id,
                            "target_node_id",
                            "REF005",
                            &path,
                            index,
                            result,
                        );
                    }
                }
                "LiveRegion" => {
                    if let Some(lr) = child.as_type::<crate::ir::LiveRegion>() {
                        self.check_ref(
                            &lr.target_node_id,
                            "target_node_id",
                            "REF005",
                            &path,
                            index,
                            result,
                        );
                    }
                }
                "FocusTrap" => {
                    if let Some(ft) = child.as_type::<crate::ir::FocusTrap>() {
                        self.check_ref(
                            &ft.container_node_id,
                            "container_node_id",
                            "REF007",
                            &path,
                            index,
                            result,
                        );
                        // initial_focus is a warning, not error
                        if let Some(ref focus_id) = ft.initial_focus_node_id {
                            if !focus_id.is_empty() && !index.contains(focus_id) {
                                result.diagnostics.push(Diagnostic {
                                    severity: Severity::Warning,
                                    code: "REF008".to_string(),
                                    message: format!(
                                        "initial_focus_node_id \"{focus_id}\" does not reference an existing node"
                                    ),
                                    node_path: path.clone(),
                                    pass: self.name().to_string(),
                    hint: None,
                                });
                            }
                        }
                    }
                }
                "SubscriptionNode" => {
                    if let Some(sub) = child.as_type::<crate::ir::SubscriptionNode>() {
                        self.check_ref(
                            &sub.target_data_node_id,
                            "target_data_node_id",
                            "REF005",
                            &path,
                            index,
                            result,
                        );
                    }
                }
                "ComputeNode" => {
                    if let Some(cn) = child.as_type::<crate::ir::ComputeNode>() {
                        if let Some(ref inputs) = cn.inputs {
                            for (j, input) in inputs.iter().enumerate() {
                                if let Some(ref src_id) = input.source_node_id {
                                    if !src_id.is_empty() && !index.contains(src_id) {
                                        result.diagnostics.push(Diagnostic {
                                            severity: Severity::Error,
                                            code: "REF009".to_string(),
                                            message: format!(
                                                "ComputeInput source_node_id \"{src_id}\" does not exist"
                                            ),
                                            node_path: format!("{path}/inputs/{j}"),
                                            pass: self.name().to_string(),
                    hint: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }

            // Recurse
            if let Some(grandchildren) = child.children() {
                self.check_children(&grandchildren, &format!("{path}/children"), index, result);
            }
        }
    }

    /// Check a single optional reference field.
    fn check_ref(
        &self,
        field: &Option<String>,
        field_name: &str,
        code: &str,
        path: &str,
        index: &NodeIndex,
        result: &mut ValidationResult,
    ) {
        if let Some(target_id) = field {
            if !target_id.is_empty() && !index.contains(target_id) {
                result.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    code: code.to_string(),
                    message: format!(
                        "{field_name} \"{target_id}\" does not reference an existing node"
                    ),
                    node_path: path.to_string(),
                    pass: self.name().to_string(),
                    hint: None,
                });
            }
        }
    }
}
