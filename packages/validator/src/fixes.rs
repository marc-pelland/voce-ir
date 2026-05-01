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

use serde_json::{Value, json};

use crate::errors::{Confidence, Diagnostic, FixPatch, PatchOp};

/// Apply a single JSON Patch operation to a mutable JSON value. Subset of
/// RFC 6902 supporting `add` (object set, array append via `/-`, array index
/// insert), `replace` (object/array index), and `remove` (object key, array
/// index). Returns Err with a descriptive message on invalid path or shape.
pub fn apply_op(root: &mut Value, op: &PatchOp) -> Result<(), String> {
    let segs = parse_pointer(&op.path)?;
    match op.op {
        "add" => apply_add(root, &segs, op.value.clone()),
        "replace" => apply_replace(root, &segs, op.value.clone()),
        "remove" => apply_remove(root, &segs),
        other => Err(format!("unsupported patch op: {other}")),
    }
}

fn parse_pointer(path: &str) -> Result<Vec<String>, String> {
    if path.is_empty() {
        return Ok(Vec::new());
    }
    if !path.starts_with('/') {
        return Err(format!("invalid JSON Pointer (must start with /): {path}"));
    }
    // RFC 6901: ~1 → /, ~0 → ~. Decode after splitting.
    Ok(path[1..]
        .split('/')
        .map(|s| s.replace("~1", "/").replace("~0", "~"))
        .collect())
}

fn apply_add(root: &mut Value, segs: &[String], value: Option<Value>) -> Result<(), String> {
    let value = value.ok_or_else(|| "add op missing value".to_string())?;
    if segs.is_empty() {
        *root = value;
        return Ok(());
    }
    let (last, parents) = segs.split_last().unwrap();
    let parent = walk_to(root, parents)?;
    // F-026: if the parent is a union wrapper and we're adding a regular
    // field, the field belongs on the inner value (e.g. node_id, direction).
    let parent = auto_descend_wrapper(parent, last);
    match parent {
        Value::Object(map) => {
            map.insert(last.clone(), value);
            Ok(())
        }
        Value::Array(arr) => {
            if last == "-" {
                arr.push(value);
                Ok(())
            } else {
                let idx: usize = last
                    .parse()
                    .map_err(|_| format!("array index expected, got {last:?}"))?;
                if idx > arr.len() {
                    return Err(format!(
                        "array index {idx} out of bounds (len {})",
                        arr.len()
                    ));
                }
                arr.insert(idx, value);
                Ok(())
            }
        }
        _ => Err(format!("cannot add into non-object/array at {parents:?}")),
    }
}

fn apply_replace(root: &mut Value, segs: &[String], value: Option<Value>) -> Result<(), String> {
    let value = value.ok_or_else(|| "replace op missing value".to_string())?;
    if segs.is_empty() {
        *root = value;
        return Ok(());
    }
    let (last, parents) = segs.split_last().unwrap();
    let parent = walk_to(root, parents)?;
    let parent = auto_descend_wrapper(parent, last);
    match parent {
        Value::Object(map) => {
            if !map.contains_key(last) {
                return Err(format!("replace: key {last:?} not found"));
            }
            map.insert(last.clone(), value);
            Ok(())
        }
        Value::Array(arr) => {
            let idx: usize = last
                .parse()
                .map_err(|_| format!("array index expected, got {last:?}"))?;
            if idx >= arr.len() {
                return Err(format!(
                    "array index {idx} out of bounds (len {})",
                    arr.len()
                ));
            }
            arr[idx] = value;
            Ok(())
        }
        _ => Err(format!("cannot replace in non-object/array")),
    }
}

fn apply_remove(root: &mut Value, segs: &[String]) -> Result<(), String> {
    if segs.is_empty() {
        return Err("cannot remove root".to_string());
    }
    let (last, parents) = segs.split_last().unwrap();
    let parent = walk_to(root, parents)?;
    let parent = auto_descend_wrapper(parent, last);
    match parent {
        Value::Object(map) => {
            if map.remove(last).is_none() {
                return Err(format!("remove: key {last:?} not found"));
            }
            Ok(())
        }
        Value::Array(arr) => {
            let idx: usize = last
                .parse()
                .map_err(|_| format!("array index expected, got {last:?}"))?;
            if idx >= arr.len() {
                return Err(format!(
                    "array index {idx} out of bounds (len {})",
                    arr.len()
                ));
            }
            arr.remove(idx);
            Ok(())
        }
        _ => Err("cannot remove from non-object/array".to_string()),
    }
}

/// Auto-descend through a `{value_type, value}` ChildUnion wrapper when the
/// next segment is addressing the inner value. Validator diagnostics use the
/// abstract IR tree (without `/value` wrappers); JSON Pointer needs them.
/// F-026: bridges those two path conventions.
///
/// Returns the inner `value` if `current` is a wrapper and `seg` isn't
/// explicitly addressing the wrapper itself; otherwise returns `current`
/// unchanged.
fn auto_descend_wrapper<'a>(current: &'a mut Value, seg: &str) -> &'a mut Value {
    if seg == "value" || seg == "value_type" || seg == "ref_id" {
        return current;
    }
    let is_wrapper = matches!(
        current,
        Value::Object(m) if m.contains_key("value_type") && m.contains_key("value")
    );
    if !is_wrapper {
        return current;
    }
    match current {
        Value::Object(map) => map.get_mut("value").unwrap(),
        _ => unreachable!("checked above"),
    }
}

fn walk_to<'a>(root: &'a mut Value, segs: &[String]) -> Result<&'a mut Value, String> {
    let mut current = root;
    for seg in segs {
        // F-026: descend through union wrappers transparently.
        current = auto_descend_wrapper(current, seg);
        current = match current {
            Value::Object(map) => map
                .get_mut(seg)
                .ok_or_else(|| format!("path segment {seg:?} not found"))?,
            Value::Array(arr) => {
                let idx: usize = seg
                    .parse()
                    .map_err(|_| format!("array index expected, got {seg:?}"))?;
                arr.get_mut(idx)
                    .ok_or_else(|| format!("array index {idx} out of bounds"))?
            }
            _ => return Err(format!("cannot descend into non-object/array at {seg:?}")),
        };
    }
    Ok(current)
}

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

    #[test]
    fn apply_op_add_object_key() {
        let mut v = json!({ "root": { "children": [] } });
        let op = PatchOp {
            op: "add",
            path: "/root/node_id".to_string(),
            value: Some(json!("hero")),
        };
        apply_op(&mut v, &op).unwrap();
        assert_eq!(v["root"]["node_id"], json!("hero"));
    }

    #[test]
    fn apply_op_add_array_append() {
        let mut v = json!({ "items": [1, 2] });
        let op = PatchOp {
            op: "add",
            path: "/items/-".to_string(),
            value: Some(json!(3)),
        };
        apply_op(&mut v, &op).unwrap();
        assert_eq!(v["items"], json!([1, 2, 3]));
    }

    #[test]
    fn apply_op_replace_existing_key() {
        let mut v = json!({ "url": "http://example.com" });
        let op = PatchOp {
            op: "replace",
            path: "/url".to_string(),
            value: Some(json!("https://example.com")),
        };
        apply_op(&mut v, &op).unwrap();
        assert_eq!(v["url"], json!("https://example.com"));
    }

    #[test]
    fn apply_op_remove_object_key() {
        let mut v = json!({ "tmp": 1, "keep": 2 });
        let op = PatchOp {
            op: "remove",
            path: "/tmp".to_string(),
            value: None,
        };
        apply_op(&mut v, &op).unwrap();
        assert!(v.as_object().unwrap().get("tmp").is_none());
        assert_eq!(v["keep"], json!(2));
    }

    /// F-026: validator paths skip the union `value` wrapper. apply_op must
    /// auto-descend through `{value_type, value}` wrappers when the segment
    /// being addressed isn't `value`/`value_type` itself.
    #[test]
    fn apply_op_descends_through_union_wrapper() {
        let mut v = json!({
            "root": {
                "children": [
                    {
                        "value_type": "Container",
                        "value": {
                            "node_id": "container",
                            "children": [
                                { "ref_id": "ghost" }
                            ]
                        }
                    }
                ]
            }
        });
        let op = PatchOp {
            op: "add",
            // Validator-style path (no /value segment) — STR002 would emit
            // exactly this for the missing-node_id leaf.
            path: "/root/children/0/children/0/node_id".to_string(),
            value: Some(json!("auto-id")),
        };
        apply_op(&mut v, &op).unwrap();
        assert_eq!(
            v["root"]["children"][0]["value"]["children"][0]["node_id"],
            json!("auto-id")
        );
    }

    #[test]
    fn apply_op_invalid_path_errors() {
        let mut v = json!({});
        let op = PatchOp {
            op: "add",
            path: "no-leading-slash".to_string(),
            value: Some(json!(1)),
        };
        assert!(apply_op(&mut v, &op).is_err());
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
                    has_fix,
                    declared,
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
