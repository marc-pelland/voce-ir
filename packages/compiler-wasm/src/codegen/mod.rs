//! WAT code generation — compiles IR nodes to WebAssembly Text Format.
//!
//! StateMachine → WASM function with state table lookup.
//! ComputeNode → WASM pure function with typed parameters.

pub mod compute;
pub mod state_machine;

use anyhow::{Context, Result};
use serde_json::Value;

/// Result of WASM compilation.
pub struct WasmCompileResult {
    /// WAT (WebAssembly Text Format) source.
    pub wat: String,
    /// JavaScript interop bridge code.
    pub js_bridge: String,
    /// Number of functions compiled.
    pub function_count: usize,
}

/// Compile IR JSON to WAT + JS bridge.
///
/// Scans the IR for StateMachine and ComputeNode nodes, compiles each
/// to a WASM function, and generates a JS bridge for calling them.
pub fn compile_to_wat(json: &str) -> Result<WasmCompileResult> {
    let doc: Value = serde_json::from_str(json).context("Failed to parse IR JSON")?;

    let mut wat_functions = Vec::new();
    let mut exports = Vec::new();
    let mut js_bridge = String::new();

    // Walk IR tree
    if let Some(root) = doc.get("root") {
        if let Some(children) = root.get("children").and_then(|v| v.as_array()) {
            walk_for_wasm(children, &mut wat_functions, &mut exports);
        }
    }

    let function_count = wat_functions.len();

    // Assemble WAT module
    let wat = if wat_functions.is_empty() {
        "(module)".to_string()
    } else {
        let mut module = String::from("(module\n");
        // Memory for state storage
        module.push_str("  (memory (export \"memory\") 1)\n");
        // Functions
        for func in &wat_functions {
            module.push_str(func);
            module.push('\n');
        }
        // Exports
        for export in &exports {
            module.push_str(export);
            module.push('\n');
        }
        module.push(')');
        module
    };

    // Generate JS bridge
    js_bridge.push_str(&crate::interop::generate_bridge(&exports));

    Ok(WasmCompileResult {
        wat,
        js_bridge,
        function_count,
    })
}

fn walk_for_wasm(
    children: &[Value],
    functions: &mut Vec<String>,
    exports: &mut Vec<String>,
) {
    for child in children {
        let type_name = child
            .get("value_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let value = child.get("value").cloned().unwrap_or(Value::Null);

        match type_name {
            "StateMachine" => {
                let id = value
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("sm");
                let func_name = id.replace('-', "_");

                let (func_wat, export_wat) =
                    state_machine::compile_state_machine(&value, &func_name);
                functions.push(func_wat);
                exports.push(export_wat);
            }
            "ComputeNode" => {
                let id = value
                    .get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("compute");
                let func_name = id.replace('-', "_");

                let (func_wat, export_wat) =
                    compute::compile_compute_node(&value, &func_name);
                functions.push(func_wat);
                exports.push(export_wat);
            }
            _ => {
                // Recurse into containers
                if let Some(gc) = value.get("children").and_then(|v| v.as_array()) {
                    walk_for_wasm(gc, functions, exports);
                }
            }
        }
    }
}
