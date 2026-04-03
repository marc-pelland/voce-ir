//! JS interop bridge — generates JavaScript code for loading and calling
//! the compiled WASM module.

/// Generate JavaScript bridge code for WASM exports.
pub fn generate_bridge(exports: &[String]) -> String {
    if exports.is_empty() {
        return String::new();
    }

    let mut js = String::new();

    js.push_str("// Voce IR WASM Interop Bridge\n");
    js.push_str("let wasmInstance = null;\n\n");

    js.push_str("async function loadVoceWasm(wasmUrl) {\n");
    js.push_str("  const response = await fetch(wasmUrl);\n");
    js.push_str("  const bytes = await response.arrayBuffer();\n");
    js.push_str("  const { instance } = await WebAssembly.instantiate(bytes);\n");
    js.push_str("  wasmInstance = instance;\n");
    js.push_str("  // Initialize all state machines\n");

    // Call init functions for state machines
    for export in exports {
        if export.contains("_init\"") {
            // Extract function name from export string
            if let Some(name) = extract_export_name(export) {
                if name.ends_with("_init") {
                    js.push_str(&format!(
                        "  if (instance.exports.{name}) instance.exports.{name}();\n"
                    ));
                }
            }
        }
    }

    js.push_str("  return instance;\n");
    js.push_str("}\n\n");

    // Generate typed wrappers for each exported function
    for export in exports {
        if let Some(name) = extract_export_name(export) {
            js.push_str(&format!(
                "function wasm_{name}(...args) {{ return wasmInstance.exports.{name}(...args); }}\n"
            ));
        }
    }

    js
}

/// Extract the function name from a WAT export declaration.
fn extract_export_name(export: &str) -> Option<&str> {
    // Format: (export "name" (func $name))
    let start = export.find('"')? + 1;
    let end = export[start..].find('"')? + start;
    Some(&export[start..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_name_from_export() {
        let export = r#"  (export "sm_send" (func $sm_send))"#;
        assert_eq!(extract_export_name(export), Some("sm_send"));
    }

    #[test]
    fn bridge_includes_init_calls() {
        let exports = vec![
            r#"  (export "sm_send" (func $sm_send))"#.to_string(),
            r#"  (export "sm_init" (func $sm_init))"#.to_string(),
        ];
        let js = generate_bridge(&exports);
        assert!(js.contains("sm_init"));
        assert!(js.contains("loadVoceWasm"));
    }
}
