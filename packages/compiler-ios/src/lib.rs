//! Voce IR iOS Compiler — compiles IR to SwiftUI.
//!
//! Maps IR nodes to SwiftUI views:
//! - Container → VStack/HStack/ZStack/LazyVGrid
//! - Surface → Rectangle with modifiers
//! - TextNode → Text with font/color modifiers
//! - MediaNode → AsyncImage
//! - StateMachine → @State with enum
//! - GestureHandler → onTapGesture/DragGesture
//! - SemanticNode → accessibilityLabel/accessibilityRole
//! - ThemeNode → Environment ColorScheme



use anyhow::{Context, Result};
use serde_json::Value;

/// Compile IR JSON to SwiftUI source code.
pub fn compile_swiftui(json: &str) -> Result<SwiftUiResult> {
    let doc: Value = serde_json::from_str(json).context("Failed to parse IR")?;
    let mut swift = String::with_capacity(4096);

    swift.push_str("import SwiftUI\n\n");
    swift.push_str("struct VoceView: View {\n");
    swift.push_str("    var body: some View {\n");

    if let Some(root) = doc.get("root") {
        if let Some(children) = root.get("children").and_then(|v| v.as_array()) {
            swift.push_str("        VStack(spacing: 0) {\n");
            for child in children {
                emit_view(&mut swift, child, 12);
            }
            swift.push_str("        }\n");
        }
    }

    swift.push_str("    }\n");
    swift.push_str("}\n\n");
    swift.push_str("#Preview {\n    VoceView()\n}\n");

    let size = swift.len();
    Ok(SwiftUiResult { swift, size_bytes: size })
}

pub struct SwiftUiResult {
    pub swift: String,
    pub size_bytes: usize,
}

fn emit_view(swift: &mut String, child: &Value, indent: usize) {
    let pad = " ".repeat(indent);
    let type_name = child.get("value_type").and_then(|v| v.as_str()).unwrap_or("");
    let value = child.get("value").cloned().unwrap_or(Value::Null);

    match type_name {
        "Container" => {
            let direction = value.get("direction").and_then(|v| v.as_str()).unwrap_or("Column");
            let stack = match direction {
                "Row" | "RowReverse" => "HStack",
                _ => "VStack",
            };
            let gap = value.get("gap").and_then(|g| g.get("value")).and_then(|v| v.as_f64()).unwrap_or(0.0);
            swift.push_str(&format!("{pad}{stack}(spacing: {gap:.0}) {{\n"));

            if let Some(children) = value.get("children").and_then(|v| v.as_array()) {
                for c in children { emit_view(swift, c, indent + 4); }
            }

            swift.push_str(&format!("{pad}}}\n"));
            emit_padding(swift, &value, &pad);
        }
        "TextNode" => {
            let content = value.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let heading = value.get("heading_level").and_then(|v| v.as_i64()).unwrap_or(0);
            let weight = value.get("font_weight").and_then(|v| v.as_str()).unwrap_or("Regular");
            let size = value.get("font_size").and_then(|f| f.get("value")).and_then(|v| v.as_f64()).unwrap_or(16.0);

            swift.push_str(&format!("{pad}Text(\"{content}\")\n"));
            swift.push_str(&format!("{pad}    .font(.system(size: {size:.0}, weight: .{wt}))\n",
                wt = swift_weight(weight)));
            if let Some(color) = value.get("color") {
                swift.push_str(&format!("{pad}    .foregroundColor({})\n", swift_color(color)));
            }
            if heading == 1 {
                swift.push_str(&format!("{pad}    .accessibilityAddTraits(.isHeader)\n"));
            }
        }
        "Surface" => {
            swift.push_str(&format!("{pad}ZStack {{\n"));
            if let Some(fill) = value.get("fill") {
                swift.push_str(&format!("{pad}    RoundedRectangle(cornerRadius: 8)\n"));
                swift.push_str(&format!("{pad}        .fill({})\n", swift_color(fill)));
            }
            if let Some(children) = value.get("children").and_then(|v| v.as_array()) {
                for c in children { emit_view(swift, c, indent + 4); }
            }
            swift.push_str(&format!("{pad}}}\n"));
            emit_padding(swift, &value, &pad);

            // Accessibility
            if let Some(sem_id) = value.get("semantic_node_id").and_then(|v| v.as_str()) {
                swift.push_str(&format!("{pad}.accessibilityLabel(\"{sem_id}\")\n"));
            }
        }
        "MediaNode" => {
            let src = value.get("src").and_then(|v| v.as_str()).unwrap_or("");
            let alt = value.get("alt").and_then(|v| v.as_str()).unwrap_or("");
            swift.push_str(&format!("{pad}AsyncImage(url: URL(string: \"{src}\")) {{ image in\n"));
            swift.push_str(&format!("{pad}    image.resizable().aspectRatio(contentMode: .fill)\n"));
            swift.push_str(&format!("{pad}}} placeholder: {{\n"));
            swift.push_str(&format!("{pad}    ProgressView()\n"));
            swift.push_str(&format!("{pad}}}\n"));
            swift.push_str(&format!("{pad}.accessibilityLabel(\"{alt}\")\n"));
        }
        _ => {
            // Non-visual nodes (StateMachine, GestureHandler, etc.) handled separately
        }
    }
}

fn emit_padding(swift: &mut String, value: &Value, pad: &str) {
    if let Some(p) = value.get("padding") {
        let top = p.get("top").and_then(|v| v.get("value")).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let leading = p.get("left").and_then(|v| v.get("value")).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let bottom = p.get("bottom").and_then(|v| v.get("value")).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let trailing = p.get("right").and_then(|v| v.get("value")).and_then(|v| v.as_f64()).unwrap_or(0.0);
        swift.push_str(&format!("{pad}.padding(EdgeInsets(top: {top:.0}, leading: {leading:.0}, bottom: {bottom:.0}, trailing: {trailing:.0}))\n"));
    }
}

fn swift_weight(weight: &str) -> &str {
    match weight {
        "Bold" => "bold",
        "SemiBold" => "semibold",
        "Medium" => "medium",
        "Light" => "light",
        "Thin" => "thin",
        "Black" => "black",
        _ => "regular",
    }
}

fn swift_color(color: &Value) -> String {
    let r = color.get("r").and_then(|v| v.as_f64()).unwrap_or(0.0) / 255.0;
    let g = color.get("g").and_then(|v| v.as_f64()).unwrap_or(0.0) / 255.0;
    let b = color.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0) / 255.0;
    format!("Color(red: {r:.3}, green: {g:.3}, blue: {b:.3})")
}
