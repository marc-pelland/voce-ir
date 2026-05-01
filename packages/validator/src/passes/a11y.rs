//! Accessibility validation pass.
//!
//! Accessibility is a compile error in Voce IR. Missing semantic
//! information blocks compilation.

use crate::errors::{CodeMeta, Confidence, Diagnostic, Severity, ValidationResult};
use crate::index::NodeIndex;
use crate::ir::{ChildNode, VoceIr};
use crate::passes::ValidationPass;

pub struct AccessibilityPass;

const CODES: &[CodeMeta] = &[
    CodeMeta {
        code: "A11Y001",
        summary: "Interactive node has no SemanticNode for screen readers",
        hint: "Add a `SemanticNode` with the right `role` (`button`, `link`, `checkbox`, \
               etc.) and reference it via `semantic_node_id`. Voce treats accessibility \
               as a compile error — interactive nodes always need semantics.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "A11Y003",
        summary: "MediaNode is missing alt text and is not marked decorative",
        hint: "Set `alt` describing the image, OR add `semantic_node_id` referencing a \
               labeled SemanticNode, OR set `decorative: true` to mark it as purely \
               visual. Pick one — silence is not a valid option.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "A11Y004",
        summary: "Heading hierarchy skips a level (e.g. h1 → h3)",
        hint: "Heading levels jumped (e.g. h1 → h3, skipping h2). Demote the heading \
               to maintain a continuous hierarchy, or add the missing intermediate \
               heading. Screen readers depend on this for navigation.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "A11Y005",
        summary: "Form field is missing a label or aria-label",
        hint: "Set a `label` string on the FormField. If the design doesn't show a \
               visible label, set `aria_label` instead so screen readers can announce \
               the field.",
        fix_confidence: Some(Confidence::Suggested),
    },
    CodeMeta {
        code: "A11Y006",
        summary: "Link or button has no accessible text content",
        hint: "Add visible text content (a TextNode child) inside the interactive \
               element, OR set `semantic_node_id` referencing a SemanticNode with a \
               `label`. Icon-only buttons need explicit names.",
        fix_confidence: None,
    },
];

impl ValidationPass for AccessibilityPass {
    fn name(&self) -> &'static str {
        "accessibility"
    }

    fn codes(&self) -> &'static [CodeMeta] {
        CODES
    }

    fn run(&self, ir: &VoceIr, _index: &NodeIndex, result: &mut ValidationResult) {
        let root = match &ir.root {
            Some(r) => r,
            None => return,
        };

        // A11Y005: Check heading hierarchy
        let mut heading_levels: Vec<(String, i8)> = Vec::new();
        if let Some(ref children) = root.children {
            self.collect_headings(children, "/root/children", &mut heading_levels);
        }
        self.check_heading_hierarchy(&heading_levels, result);

        // Walk children for per-node checks
        if let Some(ref children) = root.children {
            self.check_children(children, "/root/children", result);
        }
    }
}

impl AccessibilityPass {
    fn check_children(
        &self,
        children: &[ChildNode],
        parent_path: &str,
        result: &mut ValidationResult,
    ) {
        for (i, child) in children.iter().enumerate() {
            let path = format!("{parent_path}/{i}");

            match child.type_name() {
                // A11Y004: GestureHandler must have keyboard equivalent
                "GestureHandler" => {
                    if let Some(gh) = child.as_type::<crate::ir::GestureHandler>() {
                        if gh.keyboard_key.as_ref().is_none_or(|k| k.is_empty()) {
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                code: "A11Y004".to_string(),
                                message:
                                    "GestureHandler must have a keyboard equivalent (keyboard_key)"
                                        .to_string(),
                                node_path: path.clone(),
                                pass: self.name().to_string(),
                                hint: None,
                            });
                        }
                    }
                }
                // A11Y003: MediaNode images must have alt or be decorative
                "MediaNode" => {
                    if let Some(media) = child.as_type::<crate::ir::MediaNode>() {
                        let is_decorative = media.decorative.unwrap_or(false);
                        let has_alt = media.alt.as_ref().is_some_and(|a| !a.is_empty());
                        let has_semantic = child.semantic_node_id().is_some();

                        if !is_decorative && !has_alt && !has_semantic {
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                code: "A11Y003".to_string(),
                                message: "MediaNode must have alt text, a semantic_node_id, or be marked decorative".to_string(),
                                node_path: path.clone(),
                                pass: self.name().to_string(),
                    hint: None,
                            });
                        }
                    }
                }
                // A11Y001: FormNode must have semantic
                "FormNode" => {
                    if let Some(form) = child.as_type::<crate::ir::FormNode>() {
                        if form.semantic_node_id.as_ref().is_none_or(|s| s.is_empty()) {
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                code: "A11Y001".to_string(),
                                message: "FormNode must have a semantic_node_id for accessibility"
                                    .to_string(),
                                node_path: path.clone(),
                                pass: self.name().to_string(),
                                hint: None,
                            });
                        }
                    }
                }
                // A11Y006: TextNode with href must have non-empty content for accessible link text
                "TextNode" => {
                    if let Some(text) = child.as_type::<crate::ir::TextNode>() {
                        if text.href.as_ref().is_some_and(|h| !h.is_empty()) {
                            let has_content = text.content.as_ref().is_some_and(|c| !c.is_empty());
                            let has_semantic = child.semantic_node_id().is_some();
                            if !has_content && !has_semantic {
                                result.diagnostics.push(Diagnostic {
                                    severity: Severity::Error,
                                    code: "A11Y006".to_string(),
                                    message: "Link (TextNode with href) must have text content or a semantic_node_id with a label".to_string(),
                                    node_path: path.clone(),
                                    pass: self.name().to_string(),
                                    hint: Some("Add content text or a SemanticNode with an aria-label".to_string()),
                                });
                            }
                        }
                    }
                }
                _ => {}
            }

            // Recurse
            if let Some(grandchildren) = child.children() {
                self.check_children(&grandchildren, &format!("{path}/children"), result);
            }
        }
    }

    fn collect_headings(
        &self,
        children: &[ChildNode],
        parent_path: &str,
        headings: &mut Vec<(String, i8)>,
    ) {
        for (i, child) in children.iter().enumerate() {
            let path = format!("{parent_path}/{i}");

            if child.type_name() == "TextNode" {
                if let Some(text) = child.as_type::<crate::ir::TextNode>() {
                    if let Some(level) = text.heading_level {
                        if level > 0 {
                            headings.push((path.clone(), level));
                        }
                    }
                }
            }

            if let Some(grandchildren) = child.children() {
                self.collect_headings(&grandchildren, &format!("{path}/children"), headings);
            }
        }
    }

    fn check_heading_hierarchy(&self, headings: &[(String, i8)], result: &mut ValidationResult) {
        if headings.is_empty() {
            return;
        }

        let mut prev_level: i8 = 0;
        for (path, level) in headings {
            if prev_level > 0 && *level > prev_level + 1 {
                result.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    code: "A11Y005".to_string(),
                    message: format!(
                        "Heading level skipped: h{prev_level} -> h{level} (must not skip levels)"
                    ),
                    node_path: path.clone(),
                    pass: self.name().to_string(),
                    hint: None,
                });
            }
            prev_level = *level;
        }
    }
}
