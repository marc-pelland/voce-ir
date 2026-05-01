//! Security validation pass.
//!
//! OWASP-informed checks. Mutations require CSRF, auth routes
//! need redirects, HTTPS enforced.

use crate::errors::{CodeMeta, Diagnostic, Severity, ValidationResult};
use crate::index::NodeIndex;
use crate::ir::{ChildNode, VoceIr};
use crate::passes::ValidationPass;

pub struct SecurityPass;

const CODES: &[CodeMeta] = &[
    CodeMeta {
        code: "SEC001",
        summary: "Protected route is missing a redirect for unauthorized users",
    },
    CodeMeta {
        code: "SEC002",
        summary: "Action is missing an explicit allowed-origins list",
    },
    CodeMeta {
        code: "SEC003",
        summary: "Resource URL uses http:// — should use https:// for security",
    },
    CodeMeta {
        code: "SEC004",
        summary: "Password field is missing the appropriate autocomplete attribute",
    },
];

impl ValidationPass for SecurityPass {
    fn name(&self) -> &'static str {
        "security"
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

        // SEC001: Check route guards
        if let Some(ref routes) = ir.routes {
            if let Some(ref entries) = routes.routes {
                for (i, entry) in entries.iter().enumerate() {
                    self.check_route_guard(entry, &format!("/routes/{i}"), result);
                }
            }
        }
    }
}

impl SecurityPass {
    fn check_children(
        &self,
        children: &[ChildNode],
        parent_path: &str,
        result: &mut ValidationResult,
    ) {
        for (i, child) in children.iter().enumerate() {
            let path = format!("{parent_path}/{i}");

            // SEC002: ActionNode mutations must have CSRF
            if child.type_name() == "ActionNode" {
                if let Some(action) = child.as_type::<crate::ir::ActionNode>() {
                    let method = action.method.as_deref().unwrap_or("GET");
                    let is_mutation = matches!(method, "POST" | "PUT" | "DELETE" | "PATCH");
                    let has_csrf = action.csrf_protected.unwrap_or(false);

                    if is_mutation && !has_csrf {
                        result.diagnostics.push(Diagnostic {
                            severity: Severity::Error,
                            code: "SEC002".to_string(),
                            message: format!(
                                "ActionNode with method {method} must have csrf_protected: true"
                            ),
                            node_path: path.clone(),
                            pass: self.name().to_string(),
                            hint: None,
                        });
                    }
                }
            }

            // SEC003: MediaNode src should use HTTPS
            if child.type_name() == "MediaNode" {
                if let Some(media) = child.as_type::<crate::ir::MediaNode>() {
                    if let Some(ref src) = media.src {
                        if src.starts_with("http://") {
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Warning,
                                code: "SEC003".to_string(),
                                message: "MediaNode src uses HTTP — use HTTPS for security"
                                    .to_string(),
                                node_path: path.clone(),
                                pass: self.name().to_string(),
                                hint: None,
                            });
                        }
                    }
                }
            }

            // SEC004: Password fields should have autocomplete
            if child.type_name() == "FormNode" {
                if let Some(form) = child.as_type::<crate::ir::FormNode>() {
                    if let Some(ref fields) = form.fields {
                        for (j, field) in fields.iter().enumerate() {
                            if field.field_type.as_deref() == Some("Password") {
                                let has_autocomplete = field
                                    .autocomplete
                                    .as_ref()
                                    .is_some_and(|a| a == "NewPassword" || a == "CurrentPassword");
                                if !has_autocomplete {
                                    result.diagnostics.push(Diagnostic {
                                        severity: Severity::Warning,
                                        code: "SEC004".to_string(),
                                        message: "Password field should have autocomplete: NewPassword or CurrentPassword".to_string(),
                                        node_path: format!("{path}/fields/{j}"),
                                        pass: self.name().to_string(),
                    hint: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            if let Some(grandchildren) = child.children() {
                self.check_children(&grandchildren, &format!("{path}/children"), result);
            }
        }
    }

    fn check_route_guard(
        &self,
        entry: &crate::ir::RouteEntry,
        path: &str,
        result: &mut ValidationResult,
    ) {
        if let Some(ref guard) = entry.guard {
            if guard.requires_auth && guard.redirect_on_fail.as_ref().is_none_or(|r| r.is_empty()) {
                result.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    code: "SEC001".to_string(),
                    message: "Protected route (requires_auth) must specify redirect_on_fail"
                        .to_string(),
                    node_path: path.to_string(),
                    pass: self.name().to_string(),
                    hint: None,
                });
            }
        }

        // Check nested routes
        if let Some(ref children) = entry.children {
            for (i, child_entry) in children.iter().enumerate() {
                self.check_route_guard(child_entry, &format!("{path}/children/{i}"), result);
            }
        }
    }
}
