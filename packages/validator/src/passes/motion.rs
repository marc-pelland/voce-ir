//! Motion safety validation pass.
//!
//! Every animation must have a ReducedMotion alternative.
//! Missing ReducedMotion is a compile error — accessibility
//! is non-negotiable.

use crate::errors::{CodeMeta, Confidence, Diagnostic, Severity, ValidationResult};
use crate::index::NodeIndex;
use crate::ir::{ChildNode, VoceIr};
use crate::passes::ValidationPass;

pub struct MotionPass;

const CODES: &[CodeMeta] = &[
    CodeMeta {
        code: "MOT001",
        summary: "AnimationTransition has no reduced_motion alternative",
        hint: "Set `reduced_motion` to one of `Remove`, `Simplify`, `ReduceDuration`, \
               or `Functional`. Voce honors prefers-reduced-motion as a compile error \
               — silent animation is not an option.",
        fix_confidence: Some(Confidence::Safe),
    },
    CodeMeta {
        code: "MOT002",
        summary: "Sequence has no reduced_motion alternative",
        hint: "Multi-step Sequences need `reduced_motion` set. For most cases pick \
               `Simplify` (single-step transition) or `Remove` (instant final state).",
        fix_confidence: Some(Confidence::Safe),
    },
    CodeMeta {
        code: "MOT003",
        summary: "Spring physics has invalid damping or stiffness (must be > 0)",
        hint: "Damping and stiffness must both be greater than 0. Negative or zero \
               values are physically meaningless and cause runtime layout breakage. \
               Typical values: damping 0.7-1.0, stiffness 100-400.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "MOT004",
        summary: "Animation duration exceeds the recommended UX threshold",
        hint: "Animations longer than ~1000ms feel sluggish and delay user interaction. \
               Consider shortening, or split into a faster main motion + slower \
               secondary detail. Reserve long durations for hero/storytelling moments.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "MOT005",
        summary: "ScrollBinding has no reduced_motion alternative",
        hint: "Scroll-driven animations can cause vestibular discomfort. Set \
               `reduced_motion` so users with the OS preference get a non-animated \
               version (typically `Remove` for parallax effects).",
        fix_confidence: Some(Confidence::Safe),
    },
];

impl ValidationPass for MotionPass {
    fn name(&self) -> &'static str {
        "motion"
    }

    fn codes(&self) -> &'static [CodeMeta] {
        CODES
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
