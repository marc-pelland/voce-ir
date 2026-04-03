//! Motion safety validation pass.
//!
//! Every animation must have a ReducedMotion alternative.
//! Missing ReducedMotion is a compile error — accessibility
//! is non-negotiable.

use crate::errors::{Diagnostic, Severity, ValidationResult};
use crate::index::NodeIndex;
use crate::ir::{ChildNode, VoceIr};
use crate::passes::ValidationPass;

pub struct MotionPass;

impl ValidationPass for MotionPass {
    fn name(&self) -> &'static str {
        "motion"
    }

    fn run(&self, ir: &VoceIr, _index: &NodeIndex, result: &mut ValidationResult) {
        let root = match &ir.root {
            Some(r) => r,
            None => return,
        };

        if let Some(ref children) = root.children {
            self.check_children(children, "/root/children", result);
        }
    }
}

impl MotionPass {
    fn check_children(
        &self,
        children: &[ChildNode],
        parent_path: &str,
        result: &mut ValidationResult,
    ) {
        for (i, child) in children.iter().enumerate() {
            let path = format!("{parent_path}/{i}");

            match child.type_name() {
                // MOT001: AnimationTransition must have ReducedMotion
                "AnimationTransition" => {
                    if let Some(anim) = child.as_type::<crate::ir::AnimationTransition>() {
                        if anim.reduced_motion.is_none() {
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                code: "MOT001".to_string(),
                                message:
                                    "AnimationTransition must have a reduced_motion alternative"
                                        .to_string(),
                                node_path: path.clone(),
                                pass: self.name().to_string(),
                                hint: None,
                            });
                        }

                        // MOT004: Duration warning
                        if let Some(ref dur) = anim.duration {
                            if let Some(ms) = dur.get("ms").and_then(|v| v.as_f64()) {
                                if ms > 5000.0 {
                                    result.diagnostics.push(Diagnostic {
                                        severity: Severity::Warning,
                                        code: "MOT004".to_string(),
                                        message: format!("Animation duration is {ms}ms — consider shorter durations for better UX"),
                                        node_path: path.clone(),
                                        pass: self.name().to_string(),
                    hint: None,
                                    });
                                }
                            }
                        }
                    }
                }
                // MOT002: Sequence must have ReducedMotion
                "Sequence" => {
                    if let Some(seq) = child.as_type::<crate::ir::Sequence>() {
                        if seq.reduced_motion.is_none() {
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                code: "MOT002".to_string(),
                                message: "Sequence must have a reduced_motion alternative"
                                    .to_string(),
                                node_path: path.clone(),
                                pass: self.name().to_string(),
                                hint: None,
                            });
                        }
                    }
                }
                // MOT005: ScrollBinding must have ReducedMotion
                "ScrollBinding" => {
                    if let Some(sb) = child.as_type::<crate::ir::ScrollBinding>() {
                        if sb.reduced_motion.is_none() {
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                code: "MOT005".to_string(),
                                message: "ScrollBinding must have a reduced_motion alternative"
                                    .to_string(),
                                node_path: path.clone(),
                                pass: self.name().to_string(),
                                hint: None,
                            });
                        }
                    }
                }
                // MOT003: PhysicsBody damping must be > 0
                "PhysicsBody" => {
                    let damping = child
                        .value
                        .get("damping")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(25.0);
                    if damping <= 0.0 {
                        result.diagnostics.push(Diagnostic {
                            severity: Severity::Error,
                            code: "MOT003".to_string(),
                            message:
                                "PhysicsBody damping must be > 0 to prevent infinite oscillation"
                                    .to_string(),
                            node_path: path.clone(),
                            pass: self.name().to_string(),
                            hint: None,
                        });
                    }
                }
                _ => {}
            }

            if let Some(grandchildren) = child.children() {
                self.check_children(&grandchildren, &format!("{path}/children"), result);
            }
        }
    }
}
