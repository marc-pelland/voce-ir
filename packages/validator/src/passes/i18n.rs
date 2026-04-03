//! i18n validation pass.
//!
//! Validates internationalization completeness: localized strings
//! have valid keys, default values, and consistent usage.

use crate::errors::{Diagnostic, Severity, ValidationResult};
use crate::index::NodeIndex;
use crate::ir::{ChildNode, VoceIr};
use crate::passes::ValidationPass;

pub struct I18nPass;

impl ValidationPass for I18nPass {
    fn name(&self) -> &'static str {
        "i18n"
    }

    fn run(&self, ir: &VoceIr, _index: &NodeIndex, result: &mut ValidationResult) {
        let root = match &ir.root {
            Some(r) => r,
            None => return,
        };

        // Collect all TextNodes to check consistency
        let mut has_localized = false;
        let mut has_plain = false;

        if let Some(ref children) = root.children {
            self.check_children(
                children,
                "/root/children",
                &mut has_localized,
                &mut has_plain,
                result,
            );
        }

        // I18N001: If any TextNode uses localized_content, warn about inconsistency
        if has_localized && has_plain {
            result.diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                code: "I18N001".to_string(),
                message: "Some TextNodes use localized_content while others use plain content — consider using localized_content consistently for i18n support".to_string(),
                node_path: "/root".to_string(),
                pass: self.name().to_string(),
                    hint: None,
            });
        }
    }
}

impl I18nPass {
    fn check_children(
        &self,
        children: &[ChildNode],
        parent_path: &str,
        has_localized: &mut bool,
        has_plain: &mut bool,
        result: &mut ValidationResult,
    ) {
        for (i, child) in children.iter().enumerate() {
            let path = format!("{parent_path}/{i}");

            if child.type_name() == "TextNode" {
                if let Some(text) = child.as_type::<crate::ir::TextNode>() {
                    if let Some(ref loc) = text.localized_content {
                        *has_localized = true;

                        // I18N002: Key must be non-empty
                        if loc.message_key.as_ref().is_none_or(|k| k.is_empty()) {
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                code: "I18N002".to_string(),
                                message: "LocalizedString must have a non-empty message_key"
                                    .to_string(),
                                node_path: path.clone(),
                                pass: self.name().to_string(),
                                hint: None,
                            });
                        }

                        // I18N003: Default value should be present
                        if loc.default_value.as_ref().is_none_or(|v| v.is_empty()) {
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Warning,
                                code: "I18N003".to_string(),
                                message: "LocalizedString should have a default_value for fallback"
                                    .to_string(),
                                node_path: path.clone(),
                                pass: self.name().to_string(),
                                hint: None,
                            });
                        }
                    } else if text.content.is_some() {
                        *has_plain = true;
                    }
                }
            }

            if let Some(grandchildren) = child.children() {
                self.check_children(
                    &grandchildren,
                    &format!("{path}/children"),
                    has_localized,
                    has_plain,
                    result,
                );
            }
        }
    }
}
