//! Security validation pass.
//!
//! OWASP-informed checks. Mutations require CSRF, auth routes
//! need redirects, HTTPS enforced.

use crate::errors::{CodeMeta, Confidence, Diagnostic, Severity, ValidationResult};
use crate::index::NodeIndex;
use crate::ir::{ChildNode, VoceIr};
use crate::passes::ValidationPass;

pub struct SecurityPass;

const CODES: &[CodeMeta] = &[
    CodeMeta {
        code: "SEC001",
        summary: "Protected route is missing a redirect for unauthorized users",
        hint: "Routes with `requires_auth: true` need `redirect_on_fail` set. \
               Typical value is `/login` so unauthorized visitors land on the \
               sign-in page instead of seeing a permission error.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "SEC002",
        summary: "Action is missing an explicit allowed-origins list",
        hint: "ActionNodes posting to external endpoints need `allowed_origins`. \
               List specific domains; wildcard origins (`*`) defeat CORS protection.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "SEC003",
        summary: "Resource URL uses http:// — should use https:// for security",
        hint: "Change the URL to https://. Modern browsers block mixed content on \
               secure pages, so http:// resources won't load anyway.",
        fix_confidence: Some(Confidence::Suggested),
    },
    CodeMeta {
        code: "SEC004",
        summary: "Password field is missing the appropriate autocomplete attribute",
        hint: "Set `autocomplete: NewPassword` on signup/reset forms, or \
               `CurrentPassword` on login forms. Password managers depend on \
               this hint to fill the right credential.",
        fix_confidence: Some(Confidence::Suggested),
    },
    // ── S70 Day 2: SEC005-SEC009 ──────────────────────────────────────
    CodeMeta {
        code: "SEC005",
        summary: "ActionNode endpoint uses http:// — must be relative or https://",
        hint: "Action endpoints carry user data over the wire — http:// leaks payloads to anyone on the path. \
               Either change the URL to https://, or use a relative path (\"/api/...\") if the action targets the \
               same origin as the page.",
        fix_confidence: Some(Confidence::Suggested),
    },
    CodeMeta {
        code: "SEC006",
        summary: "URL field uses a dangerous scheme (javascript:, vbscript:, data:)",
        hint: "javascript: and vbscript: URLs execute code in the page's origin — a classic XSS vector. data: URLs \
               can carry executable HTML. Use https:// or a relative path. If you need to encode an inline image, \
               put it on MediaNode.src (only the image scheme is honored).",
        fix_confidence: None,
    },
    CodeMeta {
        code: "SEC007",
        summary: "MediaNode src is an external HTTP URL — should be HTTPS or self-hosted",
        hint: "External images leak the user's IP and referrer to a third party every page load, and downgrade \
               the page's security posture if served over http://. Pin to your own CDN, self-host, or upgrade to \
               https:// if the third-party host supports it.",
        fix_confidence: Some(Confidence::Suggested),
    },
    CodeMeta {
        code: "SEC008",
        summary: "Link target attribute has an unexpected value",
        hint: "Valid HTML target values are _self, _blank, _parent, _top, or a named frame. The compiler emits \
               rel=\"noopener noreferrer\" automatically for target=\"_blank\" — but only if the value is exactly \
               \"_blank\". Typos like \"blank\" or \"_BLANK\" silently bypass that defense.",
        fix_confidence: Some(Confidence::Suggested),
    },
    CodeMeta {
        code: "SEC009",
        summary: "StructuredData.properties_json contains a script-tag breakout",
        hint: "JSON-LD lands inside <script type=\"application/ld+json\">…</script>. A literal `</script` in the \
               properties body terminates the tag early and lets attacker-controlled content execute. Either \
               remove the </script substring or pre-escape with \\u003c.",
        fix_confidence: None,
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

        // SEC009: structured-data JSON-LD breakout. Runs against the page
        // metadata regardless of whether children were validated above.
        if let Some(ref meta) = root.metadata {
            if let Some(ref sds) = meta.structured_data {
                for (i, sd) in sds.iter().enumerate() {
                    if let Some(props) = sd.get("properties_json").and_then(|v| v.as_str()) {
                        if contains_script_close(props) {
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                code: "SEC009".to_string(),
                                message:
                                    "StructuredData.properties_json contains '</script' — JSON-LD breakout"
                                        .to_string(),
                                node_path: format!(
                                    "/root/metadata/structured_data/{i}/properties_json"
                                ),
                                pass: self.name().to_string(),
                                hint: None,
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Case-insensitive search for the literal substring `</script` (the prefix
/// that terminates a script tag). Allowing optional whitespace before the
/// closing > matches what real browsers accept.
fn contains_script_close(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    lower.contains("</script")
}

/// True if `url` uses a scheme known to execute code in the page's origin.
fn is_dangerous_scheme(url: &str) -> bool {
    let trimmed = url.trim_start().to_ascii_lowercase();
    trimmed.starts_with("javascript:")
        || trimmed.starts_with("vbscript:")
        // data: URIs that aren't strictly images can carry executable HTML.
        // Restrictive: any non-image-prefixed data: URL is suspicious in href.
        || (trimmed.starts_with("data:") && !trimmed.starts_with("data:image/"))
}

/// True if the URL targets an external host over plain http://.
fn is_external_http(url: &str) -> bool {
    url.trim_start().to_ascii_lowercase().starts_with("http://")
}

/// Valid values for the HTML `target` attribute. Anything else (including
/// case variants of these) is suspect — most often a typo that defeats the
/// rel="noopener" auto-emission for `_blank`.
const VALID_TARGETS: &[&str] = &["_self", "_blank", "_parent", "_top"];

fn is_valid_target(target: &str) -> bool {
    // _self/_blank/_parent/_top are case-sensitive in HTML5; anything else
    // is either a named frame (any non-keyword string is allowed) OR a
    // typo. We can't tell those apart here; flag anything that LOOKS LIKE
    // a keyword token but isn't an exact match.
    if VALID_TARGETS.contains(&target) {
        return true;
    }
    // A leading underscore signals "intended a keyword" — flag mismatches.
    if target.starts_with('_') {
        return false;
    }
    // Otherwise treat as a named frame and accept.
    true
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

                    // SEC005: ActionNode endpoint must be relative or HTTPS.
                    // SEC006: Endpoint must not use a dangerous URL scheme.
                    if let Some(ref source) = action.source {
                        if let Some(endpoint) = source.get("endpoint").and_then(|v| v.as_str()) {
                            if is_external_http(endpoint) {
                                result.diagnostics.push(Diagnostic {
                                    severity: Severity::Error,
                                    code: "SEC005".to_string(),
                                    message: format!(
                                        "ActionNode endpoint uses http://: {endpoint}"
                                    ),
                                    node_path: format!("{path}/source/endpoint"),
                                    pass: self.name().to_string(),
                                    hint: None,
                                });
                            }
                            if is_dangerous_scheme(endpoint) {
                                result.diagnostics.push(Diagnostic {
                                    severity: Severity::Error,
                                    code: "SEC006".to_string(),
                                    message: format!(
                                        "ActionNode endpoint uses a dangerous URL scheme: {endpoint}"
                                    ),
                                    node_path: format!("{path}/source/endpoint"),
                                    pass: self.name().to_string(),
                                    hint: None,
                                });
                            }
                        }
                    }
                }
            }

            // SEC003: MediaNode src should use HTTPS
            // SEC006: Reject javascript:/vbscript:/data: URLs in image src
            // SEC007: External HTTP image source — degrade signal, suggest CDN
            if child.type_name() == "MediaNode" {
                if let Some(media) = child.as_type::<crate::ir::MediaNode>() {
                    if let Some(ref src) = media.src {
                        if src.starts_with("http://") {
                            // SEC003 still fires for back-compat; SEC007 adds the
                            // "external HTTP image" lens with the CDN guidance.
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Warning,
                                code: "SEC003".to_string(),
                                message: "MediaNode src uses HTTP — use HTTPS for security"
                                    .to_string(),
                                node_path: path.clone(),
                                pass: self.name().to_string(),
                                hint: None,
                            });
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Warning,
                                code: "SEC007".to_string(),
                                message:
                                    "External HTTP image — pin to a trusted CDN, self-host, or upgrade to HTTPS"
                                        .to_string(),
                                node_path: format!("{path}/src"),
                                pass: self.name().to_string(),
                                hint: None,
                            });
                        }
                        if is_dangerous_scheme(src) {
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                code: "SEC006".to_string(),
                                message: format!(
                                    "MediaNode src uses a dangerous URL scheme: {src}"
                                ),
                                node_path: format!("{path}/src"),
                                pass: self.name().to_string(),
                                hint: None,
                            });
                        }
                    }
                }
            }

            // SEC006: TextNode/Surface href must not use a dangerous scheme.
            // SEC008: Linked nodes with target attributes — verify the value
            // is one of the HTML keywords or a non-leading-underscore frame
            // name (catches typos like "_blank2" / "blank" that bypass the
            // compiler's auto rel="noopener noreferrer" emission).
            let type_name = child.type_name();
            if type_name == "TextNode" || type_name == "Surface" {
                let raw = &child.value;
                if let Some(href) = raw.get("href").and_then(|v| v.as_str()) {
                    if is_dangerous_scheme(href) {
                        result.diagnostics.push(Diagnostic {
                            severity: Severity::Error,
                            code: "SEC006".to_string(),
                            message: format!("href uses a dangerous URL scheme: {href}"),
                            node_path: format!("{path}/href"),
                            pass: self.name().to_string(),
                            hint: None,
                        });
                    }
                }
                if let Some(target) = raw.get("target").and_then(|v| v.as_str()) {
                    if !is_valid_target(target) {
                        result.diagnostics.push(Diagnostic {
                            severity: Severity::Warning,
                            code: "SEC008".to_string(),
                            message: format!(
                                "target=\"{target}\" is not a recognized HTML keyword — typo? \
                                 The compiler only auto-emits rel=\"noopener noreferrer\" for exactly \"_blank\""
                            ),
                            node_path: format!("{path}/target"),
                            pass: self.name().to_string(),
                            hint: None,
                        });
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
