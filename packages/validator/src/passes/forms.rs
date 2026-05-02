//! Forms validation pass.
//!
//! Validates form structure, field names, labels, and submission config.

use std::collections::HashSet;

use crate::errors::{CodeMeta, Confidence, Diagnostic, Severity, ValidationResult};
use crate::index::NodeIndex;
use crate::ir::{ChildNode, VoceIr};
use crate::passes::ValidationPass;

pub struct FormsPass;

const CODES: &[CodeMeta] = &[
    CodeMeta {
        code: "FRM001",
        summary: "FormNode must have at least one field",
        hint: "Add at least one FormField to the `fields` array (text, email, \
               password, textarea, etc.). An empty form has nothing to submit.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "FRM002",
        summary: "FormNode is missing a submission configuration",
        hint: "Add `submission: { action_node_id, encoding, progressive }` to \
               the FormNode. Without it, submitted form data has nowhere to go.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "FRM003",
        summary: "Duplicate field name within the same FormNode",
        hint: "Two FormFields share the same `name`. Server-side handlers can't \
               distinguish them — rename one to be unique within the form.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "FRM004",
        summary: "Email field is missing an Email validation rule",
        hint: "Add `{ rule_type: \"Email\", message: \"...\" }` to the field's \
               `validations` array. The HTML input enforces format client-side, \
               but the validation rule produces server-side checks too.",
        fix_confidence: Some(Confidence::Safe),
    },
    CodeMeta {
        code: "FRM009",
        summary: "FormField has no label, breaking screen-reader accessibility",
        hint: "Set a `label` string on the FormField. Required for screen readers \
               and helpful for users skimming the form. Don't rely on placeholder \
               text — placeholders disappear on focus.",
        fix_confidence: Some(Confidence::Suggested),
    },
    CodeMeta {
        code: "FRM010",
        summary: "FormLayout.direction is not a valid LayoutDirection",
        hint: "`direction` must be one of \"Row\", \"Column\", \"RowReverse\", \
               or \"ColumnReverse\" (matching the LayoutDirection enum). \
               Anything else is rejected by the compiler. Omit the field to \
               keep the default Column layout.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "FRM011",
        summary: "FormLayout.button_alignment is not a valid FormButtonAlignment",
        hint: "`button_alignment` must be one of \"Start\", \"Center\", \"End\", \
               or \"Stretch\". Omit the field to keep the default Start \
               alignment, which matches the S61 baseline form CSS.",
        fix_confidence: None,
    },
];

const VALID_LAYOUT_DIRECTIONS: &[&str] = &["Row", "Column", "RowReverse", "ColumnReverse"];
const VALID_BUTTON_ALIGNMENTS: &[&str] = &["Start", "Center", "End", "Stretch"];

impl ValidationPass for FormsPass {
    fn name(&self) -> &'static str {
        "forms"
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

impl FormsPass {
    fn check_children(
        &self,
        children: &[ChildNode],
        parent_path: &str,
        result: &mut ValidationResult,
    ) {
        for (i, child) in children.iter().enumerate() {
            let path = format!("{parent_path}/{i}");

            if child.type_name() == "FormNode" {
                if let Some(form) = child.as_type::<crate::ir::FormNode>() {
                    self.check_form(&form, &path, result);
                }
            }

            if let Some(grandchildren) = child.children() {
                self.check_children(&grandchildren, &format!("{path}/children"), result);
            }
        }
    }

    fn check_form(&self, form: &crate::ir::FormNode, path: &str, result: &mut ValidationResult) {
        let fields = form.fields.as_deref().unwrap_or(&[]);

        // FRM001: Must have at least one field
        if fields.is_empty() {
            result.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                code: "FRM001".to_string(),
                message: "FormNode must have at least one field".to_string(),
                node_path: path.to_string(),
                pass: self.name().to_string(),
                hint: None,
            });
        }

        // FRM010 / FRM011: validate FormLayout enum values when present.
        // Optional fields, so absence is fine; only check declared values.
        if let Some(ref layout) = form.layout {
            if let Some(ref dir) = layout.direction {
                if !VALID_LAYOUT_DIRECTIONS.contains(&dir.as_str()) {
                    result.diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        code: "FRM010".to_string(),
                        message: format!(
                            "FormLayout.direction \"{dir}\" is not a valid LayoutDirection \
                             (expected one of: Row, Column, RowReverse, ColumnReverse)"
                        ),
                        node_path: format!("{path}/layout"),
                        pass: self.name().to_string(),
                        hint: None,
                    });
                }
            }

            if let Some(ref align) = layout.button_alignment {
                if !VALID_BUTTON_ALIGNMENTS.contains(&align.as_str()) {
                    result.diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        code: "FRM011".to_string(),
                        message: format!(
                            "FormLayout.button_alignment \"{align}\" is not a valid \
                             FormButtonAlignment (expected one of: Start, Center, End, Stretch)"
                        ),
                        node_path: format!("{path}/layout"),
                        pass: self.name().to_string(),
                        hint: None,
                    });
                }
            }
        }

        // FRM002: Must have submission config
        if form.submission.is_none() {
            result.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                code: "FRM002".to_string(),
                message: "FormNode must have a submission configuration".to_string(),
                node_path: path.to_string(),
                pass: self.name().to_string(),
                hint: None,
            });
        }

        // FRM003: Field names must be unique
        let mut seen_names: HashSet<String> = HashSet::new();
        for (j, field) in fields.iter().enumerate() {
            if let Some(ref name) = field.name {
                if !seen_names.insert(name.clone()) {
                    result.diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        code: "FRM003".to_string(),
                        message: format!("Duplicate field name \"{name}\" in form"),
                        node_path: format!("{path}/fields/{j}"),
                        pass: self.name().to_string(),
                        hint: None,
                    });
                }
            }

            // FRM009: Fields must have labels
            if field.label.as_ref().is_none_or(|l| l.is_empty()) {
                result.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    code: "FRM009".to_string(),
                    message: "FormField must have a label for accessibility".to_string(),
                    node_path: format!("{path}/fields/{j}"),
                    pass: self.name().to_string(),
                    hint: None,
                });
            }

            // FRM004: Email fields should have email validation
            if field.field_type.as_deref() == Some("Email") {
                let has_email_rule = field.validations.as_ref().is_some_and(|rules| {
                    rules
                        .iter()
                        .any(|r| r.rule_type.as_deref() == Some("Email"))
                });
                if !has_email_rule {
                    result.diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        code: "FRM004".to_string(),
                        message: "Email field should have an Email validation rule".to_string(),
                        node_path: format!("{path}/fields/{j}"),
                        pass: self.name().to_string(),
                        hint: None,
                    });
                }
            }
        }
    }
}
