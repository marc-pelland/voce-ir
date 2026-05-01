//! Auto-fix proposals for validator diagnostics.
//!
//! Pure function over `Diagnostic` — does not mutate the diagnostic itself.
//! Returns `None` when no auto-fix is offered for a code. Returns
//! `Some(FixPatch)` with operations describing how to fix the issue, plus a
//! confidence level (`Safe` / `Suggested` / `Risky`) so callers know whether
//! the fix can be applied without review.
//!
//! New fixes go here as new codes need them. Each fix is a small free
//! function for testability and to keep the dispatch table flat.

use serde_json::json;

use crate::errors::{Confidence, Diagnostic, FixPatch, PatchOp};

/// Compute an auto-fix for a diagnostic, if one is available for its code.
pub fn build_fix(diag: &Diagnostic) -> Option<FixPatch> {
    match diag.code.as_str() {
        "STR002" => Some(fix_str002(&diag.node_path)),
        "STR004" => Some(fix_str004(&diag.node_path)),
        "MOT001" | "MOT002" | "MOT005" => Some(fix_motion_reduced(&diag.node_path, &diag.code)),
        "FRM004" => Some(fix_frm004(&diag.node_path)),
        "FRM009" => Some(fix_frm009(&diag.node_path)),
        "A11Y005" => Some(fix_a11y005(&diag.node_path)),
        "SEC003" => Some(fix_sec003(&diag.node_path)),
        "SEC004" => Some(fix_sec004(&diag.node_path)),
        "SEO001" => Some(fix_seo001(&diag.node_path)),
        "SEO007" => Some(fix_seo007(&diag.node_path)),
        "STA002" => Some(fix_sta002(&diag.node_path)),
        _ => None,
    }
}

// ── STR002: missing node_id ─────────────────────────────────────────────────

fn fix_str002(node_path: &str) -> FixPatch {
    let suggested = path_to_id(node_path);
    FixPatch {
        confidence: Confidence::Safe,
        operations: vec![PatchOp {
            op: "add",
            path: format!("{node_path}/node_id"),
            value: Some(json!(suggested)),
        }],
        preview: format!("Add node_id \"{suggested}\" to {node_path}"),
    }
}

// ── STR004: Container missing direction ─────────────────────────────────────

fn fix_str004(node_path: &str) -> FixPatch {
    FixPatch {
        confidence: Confidence::Safe,
        operations: vec![PatchOp {
            op: "add",
            path: format!("{node_path}/direction"),
            value: Some(json!("Column")),
        }],
        preview: format!("Set direction: \"Column\" on {node_path} (safe default)"),
    }
}

// ── MOT001/MOT002/MOT005: missing reduced_motion ────────────────────────────

fn fix_motion_reduced(node_path: &str, code: &str) -> FixPatch {
    FixPatch {
        confidence: Confidence::Safe,
        operations: vec![PatchOp {
            op: "add",
            path: format!("{node_path}/reduced_motion"),
            value: Some(json!("Remove")),
        }],
        preview: format!(
            "Set reduced_motion: \"Remove\" on {node_path} ({code}) — strict accessibility default"
        ),
    }
}

// ── FRM004: email field needs Email validation rule ─────────────────────────

fn fix_frm004(node_path: &str) -> FixPatch {
    let value = json!({
        "rule_type": "Email",
        "message": "Please enter a valid email address"
    });
    FixPatch {
        confidence: Confidence::Safe,
        operations: vec![PatchOp {
            op: "add",
            path: format!("{node_path}/validations/-"),
            value: Some(value),
        }],
        preview: format!("Append Email validation rule to {node_path}/validations"),
    }
}

// ── FRM009: form field missing label ────────────────────────────────────────

fn fix_frm009(node_path: &str) -> FixPatch {
    FixPatch {
        confidence: Confidence::Suggested,
        operations: vec![PatchOp {
            op: "add",
            path: format!("{node_path}/label"),
            value: Some(json!("Label")),
        }],
        preview: format!("Add placeholder label \"Label\" to {node_path} — replace with real text"),
    }
}

// ── A11Y005: form field needs aria_label ────────────────────────────────────

fn fix_a11y005(node_path: &str) -> FixPatch {
    FixPatch {
        confidence: Confidence::Suggested,
        operations: vec![PatchOp {
            op: "add",
            path: format!("{node_path}/aria_label"),
            value: Some(json!("")),
        }],
        preview: format!("Add empty aria_label to {node_path} — replace with descriptive text"),
    }
}

// ── SEC003: http:// → https:// ──────────────────────────────────────────────

fn fix_sec003(node_path: &str) -> FixPatch {
    // The actual replacement requires reading the current value to swap http
    // for https — emit as a "replace" op that the consumer must populate. The
    // preview makes the substitution explicit.
    FixPatch {
        confidence: Confidence::Suggested,
        operations: vec![PatchOp {
            op: "replace",
            path: node_path.to_string(),
            value: None, // consumer fills this with the original value's https variant
        }],
        preview: format!(
            "Replace value at {node_path}: change http:// to https:// (verify the resource is reachable over TLS first)"
        ),
    }
}

// ── SEC004: password field autocomplete ─────────────────────────────────────

fn fix_sec004(node_path: &str) -> FixPatch {
    FixPatch {
        confidence: Confidence::Suggested,
        operations: vec![PatchOp {
            op: "add",
            path: format!("{node_path}/autocomplete"),
            value: Some(json!("CurrentPassword")),
        }],
        preview: format!(
            "Set autocomplete: \"CurrentPassword\" on {node_path} (use \"NewPassword\" for signup/reset forms)"
        ),
    }
}

// ── SEO001: page missing title ──────────────────────────────────────────────

fn fix_seo001(node_path: &str) -> FixPatch {
    FixPatch {
        confidence: Confidence::Suggested,
        operations: vec![PatchOp {
            op: "add",
            path: format!("{node_path}/title"),
            value: Some(json!("Untitled")),
        }],
        preview: format!(
            "Add placeholder title \"Untitled\" to {node_path} — replace with real page title"
        ),
    }
}

// ── SEO007: og:image missing ────────────────────────────────────────────────

fn fix_seo007(node_path: &str) -> FixPatch {
    FixPatch {
        confidence: Confidence::Suggested,
        operations: vec![PatchOp {
            op: "add",
            path: format!("{node_path}/image"),
            value: Some(json!("https://example.com/og-image.png")),
        }],
        preview: format!(
            "Add placeholder og:image at {node_path} — replace with a real 1200×630 image URL"
        ),
    }
}

// ── STA002: StateMachine missing initial state ─────────────────────────────

fn fix_sta002(node_path: &str) -> FixPatch {
    FixPatch {
        confidence: Confidence::Suggested,
        operations: vec![PatchOp {
            op: "add",
            path: format!("{node_path}/states/0/initial"),
            value: Some(json!(true)),
        }],
        preview: format!(
            "Mark the first state at {node_path}/states/0 as initial (verify it's the right starting state)"
        ),
    }
}

// ── helpers ─────────────────────────────────────────────────────────────────

/// Convert a JSON-Pointer-style node_path into a slug usable as a node_id.
/// "/root/children/2/value" → "root-children-2-value".
fn path_to_id(node_path: &str) -> String {
    let trimmed = node_path.trim_start_matches('/');
    if trimmed.is_empty() {
        "node".to_string()
    } else {
        trimmed.replace('/', "-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Severity;

    fn diag(code: &str, path: &str) -> Diagnostic {
        Diagnostic {
            severity: Severity::Error,
            code: code.to_string(),
            message: String::new(),
            node_path: path.to_string(),
            pass: String::new(),
            hint: None,
        }
    }

    #[test]
    fn str002_emits_safe_add_with_path_slug() {
        let fix = build_fix(&diag("STR002", "/root/children/2/value")).unwrap();
        assert_eq!(fix.confidence, Confidence::Safe);
        assert_eq!(fix.operations.len(), 1);
        assert_eq!(fix.operations[0].op, "add");
        assert_eq!(fix.operations[0].path, "/root/children/2/value/node_id");
        assert_eq!(
            fix.operations[0].value,
            Some(json!("root-children-2-value"))
        );
    }

    #[test]
    fn str004_adds_column_direction() {
        let fix = build_fix(&diag("STR004", "/root/children/0/value")).unwrap();
        assert_eq!(fix.confidence, Confidence::Safe);
        assert_eq!(fix.operations[0].path, "/root/children/0/value/direction");
        assert_eq!(fix.operations[0].value, Some(json!("Column")));
    }

    #[test]
    fn motion_codes_emit_reduced_motion_remove() {
        for code in &["MOT001", "MOT002", "MOT005"] {
            let fix = build_fix(&diag(code, "/root/animations/0")).unwrap();
            assert_eq!(fix.confidence, Confidence::Safe);
            assert_eq!(fix.operations[0].path, "/root/animations/0/reduced_motion");
            assert_eq!(fix.operations[0].value, Some(json!("Remove")));
        }
    }

    #[test]
    fn frm004_appends_email_rule_to_validations() {
        let fix = build_fix(&diag("FRM004", "/root/forms/0/fields/1")).unwrap();
        assert_eq!(fix.confidence, Confidence::Safe);
        assert_eq!(
            fix.operations[0].path,
            "/root/forms/0/fields/1/validations/-"
        );
    }

    #[test]
    fn unknown_code_returns_none() {
        assert!(build_fix(&diag("UNKNOWN999", "/root")).is_none());
    }

    /// Every code with a non-None fix_confidence in the registry must produce
    /// a fix from build_fix(), and every code build_fix() returns Some for must
    /// declare a fix_confidence in the registry. Catches drift between the
    /// pass CodeMeta entries and the fixes.rs dispatch table.
    #[test]
    fn catalog_and_dispatch_table_agree() {
        use crate::passes;
        for pass in passes::all_passes() {
            for meta in pass.codes() {
                let d = diag(meta.code, "/root");
                let has_fix = build_fix(&d).is_some();
                let declared = meta.fix_confidence.is_some();
                assert_eq!(
                    has_fix, declared,
                    "code {} drift: catalog declares fix_confidence={:?} but \
                     build_fix returns {} — keep both in sync.",
                    meta.code,
                    meta.fix_confidence,
                    if has_fix { "Some(...)" } else { "None" }
                );
            }
        }
    }

    #[test]
    fn path_to_id_handles_root_and_paths() {
        assert_eq!(path_to_id(""), "node");
        assert_eq!(path_to_id("/"), "node");
        assert_eq!(path_to_id("/root"), "root");
        assert_eq!(path_to_id("/root/children/0"), "root-children-0");
    }
}
