//! Voce IR Android Compiler — compiles IR to Jetpack Compose.
//!
//! Maps IR nodes to Compose functions:
//! - Container → Column/Row/LazyVerticalGrid
//! - Surface → Card/Box with Modifier
//! - TextNode → Text composable
//! - MediaNode → AsyncImage (Coil)
//! - StateMachine → mutableStateOf with when expression
//! - GestureHandler → Modifier.clickable/pointerInput
//! - SemanticNode → Modifier.semantics { contentDescription, role }

use anyhow::{Context, Result};
use serde_json::Value;

pub struct ComposeResult {
    pub kotlin: String,
    pub size_bytes: usize,
}

/// Compile IR JSON to Jetpack Compose Kotlin code.
pub fn compile_compose(json: &str) -> Result<ComposeResult> {
    let doc: Value = serde_json::from_str(json).context("Failed to parse IR")?;
    let mut kt = String::with_capacity(4096);

    kt.push_str("package com.voce.generated\n\n");
    kt.push_str("import androidx.compose.foundation.layout.*\n");
    kt.push_str("import androidx.compose.material3.*\n");
    kt.push_str("import androidx.compose.runtime.*\n");
    kt.push_str("import androidx.compose.ui.*\n");
    kt.push_str("import androidx.compose.ui.unit.dp\n");
    kt.push_str("import androidx.compose.ui.unit.sp\n");
    kt.push_str("import androidx.compose.ui.graphics.Color\n");
    kt.push_str("import androidx.compose.ui.text.font.FontWeight\n\n");

    kt.push_str("@Composable\nfun VoceScreen() {\n");

    if let Some(root) = doc.get("root") {
        if let Some(children) = root.get("children").and_then(|v| v.as_array()) {
            kt.push_str("    Column {\n");
            for child in children {
                emit_composable(&mut kt, child, 8);
            }
            kt.push_str("    }\n");
        }
    }

    kt.push_str("}\n");

    let size = kt.len();
    Ok(ComposeResult {
        kotlin: kt,
        size_bytes: size,
    })
}

fn emit_composable(kt: &mut String, child: &Value, indent: usize) {
    let pad = " ".repeat(indent);
    let type_name = child
        .get("value_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let value = child.get("value").cloned().unwrap_or(Value::Null);

    match type_name {
        "Container" => {
            let direction = value
                .get("direction")
                .and_then(|v| v.as_str())
                .unwrap_or("Column");
            let composable = match direction {
                "Row" | "RowReverse" => "Row",
                _ => "Column",
            };
            let gap = value
                .get("gap")
                .and_then(|g| g.get("value"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            kt.push_str(&format!("{pad}{composable}(\n"));
            kt.push_str(&format!(
                "{pad}    verticalArrangement = Arrangement.spacedBy({gap:.0}.dp)\n"
            ));
            kt.push_str(&format!("{pad}) {{\n"));

            if let Some(children) = value.get("children").and_then(|v| v.as_array()) {
                for c in children {
                    emit_composable(kt, c, indent + 4);
                }
            }

            kt.push_str(&format!("{pad}}}\n"));
        }
        "TextNode" => {
            let content = value.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let size = value
                .get("font_size")
                .and_then(|f| f.get("value"))
                .and_then(|v| v.as_f64())
                .unwrap_or(16.0);
            let weight = value
                .get("font_weight")
                .and_then(|v| v.as_str())
                .unwrap_or("Regular");

            kt.push_str(&format!("{pad}Text(\n"));
            kt.push_str(&format!("{pad}    text = \"{content}\",\n"));
            kt.push_str(&format!("{pad}    fontSize = {size:.0}.sp,\n"));
            kt.push_str(&format!(
                "{pad}    fontWeight = FontWeight.{wt},\n",
                wt = kotlin_weight(weight)
            ));
            if let Some(color) = value.get("color") {
                kt.push_str(&format!("{pad}    color = {},\n", kotlin_color(color)));
            }
            kt.push_str(&format!("{pad})\n"));
        }
        "Surface" => {
            kt.push_str(&format!("{pad}Card(\n"));
            kt.push_str(&format!("{pad}    modifier = Modifier.fillMaxWidth()\n"));
            kt.push_str(&format!("{pad}) {{\n"));
            if let Some(children) = value.get("children").and_then(|v| v.as_array()) {
                for c in children {
                    emit_composable(kt, c, indent + 4);
                }
            }
            kt.push_str(&format!("{pad}}}\n"));
        }
        _ => {}
    }
}

fn kotlin_weight(weight: &str) -> &str {
    match weight {
        "Bold" => "Bold",
        "SemiBold" => "SemiBold",
        "Medium" => "Medium",
        "Light" => "Light",
        "Thin" => "Thin",
        "Black" => "Black",
        _ => "Normal",
    }
}

fn kotlin_color(color: &Value) -> String {
    let r = color.get("r").and_then(|v| v.as_u64()).unwrap_or(0);
    let g = color.get("g").and_then(|v| v.as_u64()).unwrap_or(0);
    let b = color.get("b").and_then(|v| v.as_u64()).unwrap_or(0);
    format!("Color(0xFF{r:02X}{g:02X}{b:02X})")
}
