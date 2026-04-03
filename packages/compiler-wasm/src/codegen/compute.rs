//! ComputeNode → WASM compilation.
//!
//! Pure functions that transform inputs to outputs. Compiled to WASM
//! functions that take f64 parameters and return f64.

use serde_json::Value;

/// Compile a ComputeNode to a WAT function + export.
///
/// ComputeNodes have an `expression` field with simple math operations.
/// The compiler maps these to WASM instructions.
pub fn compile_compute_node(value: &Value, func_name: &str) -> (String, String) {
    let expression = value
        .get("expression")
        .and_then(|v| v.as_str())
        .unwrap_or("0");

    let inputs = value
        .get("inputs")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|i| i.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // Build parameter list
    let params: String = inputs
        .iter()
        .map(|name| format!("(param ${name} f64)"))
        .collect::<Vec<_>>()
        .join(" ");

    // Compile expression to WAT instructions
    let body = compile_expression(expression, &inputs);

    let func = format!(
        "  ;; ComputeNode: {func_name}\n  (func ${func_name} {params} (result f64)\n    {body}\n  )\n"
    );

    let export = format!("  (export \"{func_name}\" (func ${func_name}))");

    (func, export)
}

/// Compile a simple expression to WAT instructions.
///
/// Supports: +, -, *, /, variable references, numeric literals.
/// Complex expressions would need a proper parser; this handles
/// common patterns like "price * quantity" or "a + b".
fn compile_expression(expr: &str, inputs: &[String]) -> String {
    let trimmed = expr.trim();

    // Simple binary operation: "a + b", "price * quantity", etc.
    for op in ["*", "+", "-", "/"] {
        if let Some(pos) = trimmed.find(op) {
            let left = trimmed[..pos].trim();
            let right = trimmed[pos + 1..].trim();

            let left_wat = compile_operand(left, inputs);
            let right_wat = compile_operand(right, inputs);

            let wasm_op = match op {
                "+" => "f64.add",
                "-" => "f64.sub",
                "*" => "f64.mul",
                "/" => "f64.div",
                _ => "f64.add",
            };

            return format!("({wasm_op} {left_wat} {right_wat})");
        }
    }

    // Single operand (variable or literal)
    compile_operand(trimmed, inputs)
}

fn compile_operand(operand: &str, inputs: &[String]) -> String {
    // Check if it's an input variable
    if inputs.iter().any(|i| i == operand) {
        return format!("(local.get ${operand})");
    }

    // Try parsing as a number
    if let Ok(num) = operand.parse::<f64>() {
        return format!("(f64.const {num})");
    }

    // Fallback: treat as variable name with dots (field path)
    let clean = operand.replace('.', "_");
    if inputs.iter().any(|i| i == &clean) {
        return format!("(local.get ${clean})");
    }

    // Unknown — return 0
    "(f64.const 0)".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_multiply() {
        let wat = compile_expression("price * quantity", &["price".into(), "quantity".into()]);
        assert!(wat.contains("f64.mul"));
        assert!(wat.contains("local.get $price"));
        assert!(wat.contains("local.get $quantity"));
    }

    #[test]
    fn simple_add() {
        let wat = compile_expression("a + b", &["a".into(), "b".into()]);
        assert!(wat.contains("f64.add"));
    }

    #[test]
    fn literal_constant() {
        let wat = compile_expression("42", &[]);
        assert!(wat.contains("f64.const 42"));
    }
}
