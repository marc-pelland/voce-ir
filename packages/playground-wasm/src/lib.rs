//! Voce IR WASM bindings — exposes validator and compiler to the browser.
//!
//! Used by the web playground to validate and compile IR without a server.

use wasm_bindgen::prelude::*;

/// Fail-closed validation guard for the compile entrypoints (C7): IR with
/// error-severity diagnostics is structurally invalid, so rendering it is
/// undefined. Direct WASM callers otherwise bypassed validation entirely (the
/// CLI already validates first). Warnings still compile. Returns a JSON error
/// string to short-circuit, or `None` to proceed.
fn compile_error_guard(ir_json: &str) -> Option<String> {
    match voce_validator::validate(ir_json) {
        Ok(val) if val.has_errors() => Some(
            serde_json::json!({
                "ok": false,
                "html": "",
                "sizeBytes": 0,
                "error": format!(
                    "IR has {} validation error(s); fix them before compiling",
                    val.error_count()
                ),
            })
            .to_string(),
        ),
        _ => None,
    }
}

/// Validate IR JSON. Returns a JSON string with validation results.
///
/// Result shape: `{ "valid": bool, "errors": [...], "warnings": [...] }`
#[wasm_bindgen]
pub fn validate(ir_json: &str) -> String {
    match voce_validator::validate(ir_json) {
        Ok(result) => {
            let errors: Vec<serde_json::Value> = result
                .diagnostics
                .iter()
                .filter(|d| matches!(d.severity, voce_validator::errors::Severity::Error))
                .map(diagnostic_to_json)
                .collect();
            let warnings: Vec<serde_json::Value> = result
                .diagnostics
                .iter()
                .filter(|d| matches!(d.severity, voce_validator::errors::Severity::Warning))
                .map(diagnostic_to_json)
                .collect();

            serde_json::json!({
                "valid": !result.has_errors(),
                "errorCount": result.error_count(),
                "warningCount": result.warning_count(),
                "errors": errors,
                "warnings": warnings,
            })
            .to_string()
        }
        Err(e) => serde_json::json!({
            "valid": false,
            "errorCount": 1,
            "warningCount": 0,
            "errors": [{ "code": "PARSE", "message": e, "path": "" }],
            "warnings": [],
        })
        .to_string(),
    }
}

/// Validate IR JSON with per-pass telemetry (S67). Returns a JSON string
/// containing the same fields as `validate()` plus a `passes` array, where
/// each entry is `{ name, durationUs, errors, warnings, codes }`. Used by
/// site-hero (and future MCP / playground consumers) to surface real per-pass
/// data instead of synthesizing it.
#[wasm_bindgen]
pub fn validate_verbose(ir_json: &str) -> String {
    match voce_validator::validate(ir_json) {
        Ok(result) => {
            let errors: Vec<serde_json::Value> = result
                .diagnostics
                .iter()
                .filter(|d| matches!(d.severity, voce_validator::errors::Severity::Error))
                .map(diagnostic_to_json)
                .collect();
            let warnings: Vec<serde_json::Value> = result
                .diagnostics
                .iter()
                .filter(|d| matches!(d.severity, voce_validator::errors::Severity::Warning))
                .map(diagnostic_to_json)
                .collect();
            let passes: Vec<serde_json::Value> = result
                .passes
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "name": p.name,
                        "durationUs": p.duration_us,
                        "errors": p.error_count,
                        "warnings": p.warning_count,
                        "codes": p.codes,
                    })
                })
                .collect();

            serde_json::json!({
                "valid": !result.has_errors(),
                "errorCount": result.error_count(),
                "warningCount": result.warning_count(),
                "errors": errors,
                "warnings": warnings,
                "passes": passes,
            })
            .to_string()
        }
        Err(e) => serde_json::json!({
            "valid": false,
            "errorCount": 1,
            "warningCount": 0,
            "errors": [{ "code": "PARSE", "message": e, "path": "" }],
            "warnings": [],
            "passes": [],
        })
        .to_string(),
    }
}

/// Compile IR JSON to HTML (DOM target). Returns a JSON string with the result.
///
/// Result shape: `{ "ok": bool, "html": string, "sizeBytes": number, "error"?: string }`
#[wasm_bindgen]
pub fn compile_dom(ir_json: &str) -> String {
    if let Some(err) = compile_error_guard(ir_json) {
        return err;
    }
    match voce_compiler_dom::compile(ir_json, &voce_compiler_dom::CompileOptions::default()) {
        Ok(result) => serde_json::json!({
            "ok": true,
            "html": result.html,
            "sizeBytes": result.size_bytes,
        })
        .to_string(),
        Err(e) => serde_json::json!({
            "ok": false,
            "html": "",
            "sizeBytes": 0,
            "error": format!("{e:#}"),
        })
        .to_string(),
    }
}

/// Compile IR JSON to email HTML. Returns a JSON string with the result.
#[wasm_bindgen]
pub fn compile_email(ir_json: &str) -> String {
    if let Some(err) = compile_error_guard(ir_json) {
        return err;
    }
    match voce_compiler_email::compile_email(ir_json) {
        Ok(result) => serde_json::json!({
            "ok": true,
            "html": result.html,
            "sizeBytes": result.size_bytes,
        })
        .to_string(),
        Err(e) => serde_json::json!({
            "ok": false,
            "html": "",
            "sizeBytes": 0,
            "error": format!("{e:#}"),
        })
        .to_string(),
    }
}

/// Inspect IR JSON — returns a summary.
#[wasm_bindgen]
pub fn inspect(ir_json: &str) -> String {
    match serde_json::from_str::<voce_validator::ir::VoceIr>(ir_json) {
        Ok(ir) => {
            let summary = voce_validator::inspect::summarize(&ir);
            serde_json::json!({
                "ok": true,
                "nodeCount": summary.total_nodes,
                "maxDepth": summary.max_depth,
                "nodeCounts": summary.node_counts,
                "schemaVersion": summary.schema_version,
                "hasRoutes": summary.has_routes,
                "hasI18n": summary.has_i18n,
                "hasTheme": summary.has_theme,
                "hasMetadata": summary.has_metadata,
            })
            .to_string()
        }
        Err(e) => serde_json::json!({
            "ok": false,
            "error": format!("{e}"),
        })
        .to_string(),
    }
}

fn diagnostic_to_json(d: &voce_validator::errors::Diagnostic) -> serde_json::Value {
    serde_json::json!({
        "code": d.code,
        "message": d.message,
        "path": d.node_path,
        "hint": d.hint,
        "pass": d.pass,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // IR with a dangling animation target -> REF005 error.
    const INVALID_IR: &str = r#"{ "root": { "node_id": "root", "children": [
        { "value_type": "AnimationTransition", "value": { "node_id": "a",
            "target_node_id": "missing", "properties": [], "duration": { "ms": 300 },
            "reduced_motion": { "strategy": "Remove" } } }
    ] } }"#;

    const VALID_IR: &str = r#"{ "root": { "node_id": "root", "children": [
        { "value_type": "TextNode", "value": { "node_id": "t", "content": "Hi" } }
    ] } }"#;

    #[test]
    fn compile_dom_fails_closed_on_validation_errors() {
        let out = compile_dom(INVALID_IR);
        assert!(
            out.contains("\"ok\":false"),
            "should refuse invalid IR: {out}"
        );
        assert!(
            out.contains("validation error"),
            "should explain why: {out}"
        );
    }

    #[test]
    fn compile_dom_compiles_valid_ir() {
        let out = compile_dom(VALID_IR);
        assert!(
            out.contains("\"ok\":true"),
            "valid IR should compile: {out}"
        );
    }
}
