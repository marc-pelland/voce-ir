//! IR ingestion — loads validated JSON IR into the compiler IR.
//!
//! Uses serde_json::Value directly to avoid depending on the validator crate.
//! The IR is assumed to be valid (validation runs before compilation).

use std::collections::HashMap;

use anyhow::{Context, Result};
use serde_json::Value;

use crate::compiler_ir::{
    CNode, CompiledAnimation, CompiledForm, CompiledFormField, CompiledGestureHandler,
    CompiledStateMachine, CompiledTransition, CompiledValidationRule, CompilerIr, DocumentMeta,
    NodeId, NodeKind, SemanticInfo,
};

/// Ingest a JSON IR string into the compiler IR.
pub fn ingest(json: &str) -> Result<CompilerIr> {
    let doc: Value = serde_json::from_str(json).context("Failed to parse IR JSON")?;

    let mut nodes: Vec<CNode> = Vec::new();
    let mut id_map: HashMap<String, NodeId> = HashMap::new();

    let root = doc.get("root").context("IR has no root ViewRoot")?;

    // Build document metadata
    let meta = build_meta(root, &doc);

    // Create root node
    let root_id = NodeId(nodes.len());
    let root_node_id = root
        .get("node_id")
        .and_then(|v| v.as_str())
        .unwrap_or("root")
        .to_string();
    id_map.insert(root_node_id.clone(), root_id);
    nodes.push(CNode {
        id: root_node_id,
        kind: NodeKind::ViewRoot {
            language: root
                .get("document_language")
                .and_then(|v| v.as_str())
                .map(String::from),
        },
        children: Vec::new(),
        semantic_node_id: None,
        styles: HashMap::new(),
    });

    // Collect semantic nodes for ARIA emission
    let mut semantic_map = HashMap::new();
    if let Some(sems) = root.get("semantic_nodes").and_then(|v| v.as_array()) {
        for sem in sems {
            if let Some(id) = sem.get("node_id").and_then(|v| v.as_str()) {
                semantic_map.insert(
                    id.to_string(),
                    SemanticInfo {
                        role: sem.get("role").and_then(|v| v.as_str()).map(String::from),
                        label: sem.get("label").and_then(|v| v.as_str()).map(String::from),
                        labelled_by: sem
                            .get("labelled_by")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        described_by: sem
                            .get("described_by")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        tab_index: sem
                            .get("tab_index")
                            .and_then(|v| v.as_i64())
                            .map(|v| v as i32),
                    },
                );
            }
        }
    }

    // Collect interactive elements during ingestion
    let mut state_machines = Vec::new();
    let mut gesture_handlers = Vec::new();
    let mut animations = Vec::new();
    let mut forms = Vec::new();

    // Ingest children recursively
    let child_ids = if let Some(children) = root.get("children").and_then(|v| v.as_array()) {
        ingest_children(
            children,
            &mut nodes,
            &mut id_map,
            &mut state_machines,
            &mut gesture_handlers,
            &mut animations,
            &mut forms,
        )
    } else {
        Vec::new()
    };
    nodes[root_id.0].children = child_ids;

    // Collect responsive rules from ResponsiveRule nodes
    let responsive_rules = collect_responsive_rules(&nodes);

    Ok(CompilerIr {
        nodes,
        root: root_id,
        id_map,
        meta,
        state_machines,
        gesture_handlers,
        animations,
        forms,
        semantic_map,
        responsive_rules,
    })
}

fn build_meta(root: &Value, doc: &Value) -> DocumentMeta {
    let mut meta = DocumentMeta {
        schema_version: format!(
            "{}.{}",
            doc.get("schema_version_major")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            doc.get("schema_version_minor")
                .and_then(|v| v.as_i64())
                .unwrap_or(1),
        ),
        language: root
            .get("document_language")
            .and_then(|v| v.as_str())
            .map(String::from),
        text_direction: root
            .get("text_direction")
            .and_then(|v| v.as_str())
            .unwrap_or("ltr")
            .to_lowercase(),
        ..Default::default()
    };

    if let Some(pm) = root.get("metadata") {
        meta.title = pm.get("title").and_then(|v| v.as_str()).map(String::from);
        meta.description = pm
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);
        meta.canonical_url = pm
            .get("canonical_url")
            .and_then(|v| v.as_str())
            .map(String::from);
        if let Some(og) = pm.get("open_graph") {
            meta.og_title = og.get("title").and_then(|v| v.as_str()).map(String::from);
            meta.og_description = og
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from);
            meta.og_image = og.get("image").and_then(|v| v.as_str()).map(String::from);
        }
        meta.csp_override = pm
            .get("content_security_policy")
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(String::from);
    }

    // Extract structured data
    if let Some(pm) = root.get("metadata") {
        if let Some(sd_arr) = pm.get("structured_data").and_then(|v| v.as_array()) {
            for sd in sd_arr {
                if let (Some(schema_type), Some(props)) = (
                    sd.get("schema_type").and_then(|v| v.as_str()),
                    sd.get("properties_json").and_then(|v| v.as_str()),
                ) {
                    // Wrap in JSON-LD format
                    let jsonld = format!(
                        "{{\"@context\":\"https://schema.org\",\"@type\":\"{schema_type}\",{props}}}"
                    );
                    meta.structured_data.push(jsonld);
                }
            }
        }
    }

    // Extract theme as CSS custom properties
    if let Some(theme) = doc.get("theme") {
        if let Some(colors) = theme.get("colors") {
            extract_theme_colors(colors, &mut meta.theme_vars);
        }
    }

    meta
}

/// Convert theme color fields to CSS custom properties.
fn extract_theme_colors(colors: &Value, vars: &mut Vec<(String, String)>) {
    if let Some(obj) = colors.as_object() {
        for (name, color) in obj {
            if let (Some(r), Some(g), Some(b)) = (
                color.get("r").and_then(|v| v.as_u64()),
                color.get("g").and_then(|v| v.as_u64()),
                color.get("b").and_then(|v| v.as_u64()),
            ) {
                let css_name = name.replace('_', "-");
                vars.push((format!("--voce-{css_name}"), format!("rgb({r},{g},{b})")));
            }
        }
    }
}

fn ingest_children(
    children: &[Value],
    nodes: &mut Vec<CNode>,
    id_map: &mut HashMap<String, NodeId>,
    state_machines: &mut Vec<CompiledStateMachine>,
    gesture_handlers: &mut Vec<CompiledGestureHandler>,
    animations: &mut Vec<CompiledAnimation>,
    forms: &mut Vec<CompiledForm>,
) -> Vec<NodeId> {
    let mut ids = Vec::new();

    for child in children {
        let type_name = child
            .get("value_type")
            .and_then(|v| v.as_str())
            .unwrap_or("NONE");
        let value = child.get("value").cloned().unwrap_or(Value::Null);
        let node_id_str = value
            .get("node_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let semantic = value
            .get("semantic_node_id")
            .and_then(|v| v.as_str())
            .map(String::from);

        let kind = match type_name {
            "Container" => NodeKind::Container {
                layout: value
                    .get("layout")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Stack")
                    .to_string(),
                direction: value
                    .get("direction")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Column")
                    .to_string(),
                main_align: value
                    .get("main_align")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Start")
                    .to_string(),
                cross_align: value
                    .get("cross_align")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Start")
                    .to_string(),
                gap: value
                    .get("gap")
                    .and_then(|g| g.get("value"))
                    .and_then(|v| v.as_f64())
                    .map(|v| format!("{v}px")),
                wrap: value.get("wrap").and_then(|v| v.as_bool()).unwrap_or(false),
            },
            "Surface" => NodeKind::Surface {
                decorative: value
                    .get("decorative")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                href: value.get("href").and_then(|v| v.as_str()).map(String::from),
                target: value
                    .get("target")
                    .and_then(|v| v.as_str())
                    .map(String::from),
            },
            "TextNode" => {
                let content = value
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let heading_level = value
                    .get("heading_level")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i8;
                let href = value.get("href").and_then(|v| v.as_str()).map(String::from);
                let target = value
                    .get("target")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let tag = if (1..=6).contains(&heading_level) {
                    format!("h{heading_level}")
                } else if href.is_some() {
                    "a".to_string()
                } else {
                    "p".to_string()
                };
                NodeKind::Text {
                    content,
                    heading_level,
                    tag,
                    href,
                    target,
                }
            }
            "MediaNode" => NodeKind::Media {
                src: value
                    .get("src")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                alt: value
                    .get("alt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                media_type: value
                    .get("media_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Image")
                    .to_string(),
                decorative: value
                    .get("decorative")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                above_fold: value
                    .get("above_fold")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            },
            "RichTextNode" => {
                let blocks = value
                    .get("blocks")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().map(parse_rich_text_block).collect())
                    .unwrap_or_default();
                NodeKind::RichText { blocks }
            }
            _ => {
                // Collect interactive data for JS emission
                if type_name == "StateMachine" {
                    if let Some(sm) = extract_state_machine(&value) {
                        state_machines.push(sm);
                    }
                } else if type_name == "AnimationTransition" {
                    if let Some(anim) = extract_animation(&value) {
                        animations.push(anim);
                    }
                } else if type_name == "FormNode" {
                    if let Some(form) = extract_form(&value) {
                        forms.push(form);
                    }
                } else if type_name == "GestureHandler" {
                    gesture_handlers.push(CompiledGestureHandler {
                        id: node_id_str.clone(),
                        target_node_id: value
                            .get("target_node_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        gesture_type: value
                            .get("gesture_type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Tap")
                            .to_string(),
                        trigger_event: value
                            .get("trigger_event")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        trigger_state_machine: value
                            .get("trigger_state_machine")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        keyboard_key: value
                            .get("keyboard_key")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                    });
                }

                NodeKind::NonVisual {
                    type_name: type_name.to_string(),
                    data: value.clone(),
                }
            }
        };

        let styles = extract_styles(&value, type_name);

        let handle = NodeId(nodes.len());
        if !node_id_str.is_empty() {
            id_map.insert(node_id_str.clone(), handle);
        }

        nodes.push(CNode {
            id: node_id_str,
            kind,
            children: Vec::new(),
            semantic_node_id: semantic,
            styles,
        });
        ids.push(handle);

        // Recurse
        if let Some(grandchildren) = value.get("children").and_then(|v| v.as_array()) {
            let child_ids = ingest_children(
                grandchildren,
                nodes,
                id_map,
                state_machines,
                gesture_handlers,
                animations,
                forms,
            );
            nodes[handle.0].children = child_ids;
        }
    }

    ids
}

/// Extract a FormNode for HTML + JS compilation.
fn extract_form(value: &Value) -> Option<CompiledForm> {
    let id = value.get("node_id")?.as_str()?.to_string();

    let fields = value
        .get("fields")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|f| {
                    let name = f.get("name")?.as_str()?.to_string();
                    let field_type = f
                        .get("field_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Text")
                        .to_string();
                    let label = f.get("label")?.as_str()?.to_string();
                    let placeholder = f
                        .get("placeholder")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let autocomplete = f
                        .get("autocomplete")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let description = f
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    let validations = f
                        .get("validations")
                        .and_then(|v| v.as_array())
                        .map(|rules| {
                            rules
                                .iter()
                                .filter_map(|r| {
                                    Some(CompiledValidationRule {
                                        rule_type: r.get("rule_type")?.as_str()?.to_string(),
                                        value: r
                                            .get("value")
                                            .and_then(|v| v.as_str())
                                            .map(String::from),
                                        message: r.get("message")?.as_str()?.to_string(),
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    let options: Vec<String> = f
                        .get("options")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|o| o.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();

                    let style = f.get("style").and_then(extract_form_field_style);

                    Some(CompiledFormField {
                        name,
                        field_type,
                        label,
                        placeholder,
                        autocomplete,
                        validations,
                        description,
                        options,
                        style,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Get action endpoint from submission
    let submission = value.get("submission");
    let action_endpoint = submission
        .and_then(|s| s.get("action_node_id"))
        .and_then(|v| v.as_str())
        .map(String::from);
    let progressive = submission
        .and_then(|s| s.get("progressive"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let layout = value.get("layout").and_then(extract_form_layout);

    Some(CompiledForm {
        id,
        fields,
        action_endpoint,
        action_method: "POST".to_string(),
        progressive,
        layout,
    })
}

/// Convert a FormFieldStyle JSON object to its compiler-IR mirror.
/// Returns `None` only if the input is unstructured (e.g., not an
/// object); otherwise the result is `Some` even if every field is empty
/// — that lets the emitter distinguish "author declared style" from
/// "author omitted style and accepts the default".
fn extract_form_field_style(value: &Value) -> Option<crate::compiler_ir::CompiledFormFieldStyle> {
    let obj = value.as_object()?;
    Some(crate::compiler_ir::CompiledFormFieldStyle {
        padding: obj.get("padding").and_then(edge_insets_to_css),
        border: obj.get("border").and_then(border_sides_to_css),
        corner_radius: obj.get("corner_radius").and_then(corner_radii_to_css),
        background: obj.get("background").and_then(color_to_css),
        text_color: obj.get("text_color").and_then(color_to_css),
        placeholder_color: obj.get("placeholder_color").and_then(color_to_css),
        font_family: obj
            .get("font_family")
            .and_then(|v| v.as_str())
            .map(String::from),
        font_size: obj.get("font_size").map(|v| length_to_css(Some(v))),
        font_weight: obj
            .get("font_weight")
            .and_then(|v| v.as_str())
            .map(String::from),
        line_height: obj.get("line_height").map(|v| length_to_css(Some(v))),
        focus_style: obj
            .get("focus_style")
            .and_then(extract_form_field_style)
            .map(Box::new),
        error_style: obj
            .get("error_style")
            .and_then(extract_form_field_style)
            .map(Box::new),
        disabled_style: obj
            .get("disabled_style")
            .and_then(extract_form_field_style)
            .map(Box::new),
    })
}

/// Convert a FormLayout JSON object to its compiler-IR mirror.
fn extract_form_layout(value: &Value) -> Option<crate::compiler_ir::CompiledFormLayout> {
    let obj = value.as_object()?;
    Some(crate::compiler_ir::CompiledFormLayout {
        direction: obj
            .get("direction")
            .and_then(|v| v.as_str())
            .map(String::from),
        gap: obj.get("gap").map(|v| length_to_css(Some(v))),
        max_width: obj.get("max_width").map(|v| length_to_css(Some(v))),
        padding: obj.get("padding").and_then(edge_insets_to_css),
        wrap: obj.get("wrap").and_then(|v| v.as_bool()).unwrap_or(false),
        button_alignment: obj
            .get("button_alignment")
            .and_then(|v| v.as_str())
            .map(String::from),
    })
}

/// Convert a FlatBuffers Color (`{r,g,b,a}` 0-255) to a CSS color
/// string. `a == 255` (or absent) emits `rgb()`; anything lower emits
/// `rgba()`.
fn color_to_css(value: &Value) -> Option<String> {
    let r = value.get("r").and_then(|v| v.as_u64())? as u8;
    let g = value.get("g").and_then(|v| v.as_u64())? as u8;
    let b = value.get("b").and_then(|v| v.as_u64())? as u8;
    let a = value.get("a").and_then(|v| v.as_u64()).unwrap_or(255) as u8;
    if a == 255 {
        Some(format!("rgb({r},{g},{b})"))
    } else {
        let af = f64::from(a) / 255.0;
        Some(format!("rgba({r},{g},{b},{af:.2})"))
    }
}

/// Convert an EdgeInsets table to a CSS `padding`/`margin` shorthand.
fn edge_insets_to_css(value: &Value) -> Option<String> {
    let top = length_to_css(value.get("top"));
    let right = length_to_css(value.get("right"));
    let bottom = length_to_css(value.get("bottom"));
    let left = length_to_css(value.get("left"));
    if top.is_empty() && right.is_empty() && bottom.is_empty() && left.is_empty() {
        return None;
    }
    let part = |s: String| if s.is_empty() { "0".to_string() } else { s };
    Some(format!(
        "{} {} {} {}",
        part(top),
        part(right),
        part(bottom),
        part(left)
    ))
}

/// Convert a CornerRadii table to a CSS `border-radius` shorthand.
fn corner_radii_to_css(value: &Value) -> Option<String> {
    let tl = length_to_css(value.get("top_left"));
    let tr = length_to_css(value.get("top_right"));
    let br = length_to_css(value.get("bottom_right"));
    let bl = length_to_css(value.get("bottom_left"));
    if tl.is_empty() && tr.is_empty() && br.is_empty() && bl.is_empty() {
        return None;
    }
    let part = |s: String| if s.is_empty() { "0".to_string() } else { s };
    Some(format!(
        "{} {} {} {}",
        part(tl),
        part(tr),
        part(br),
        part(bl)
    ))
}

/// Convert a BorderSides table to a CSS `border` shorthand using the
/// top side as the canonical value. Per-side variation isn't expressible
/// in a single shorthand; if all four sides are equal we collapse, else
/// we still emit the top side (with a TODO once `border-top`/`-right`/
/// etc. are wired through the emit pass).
fn border_sides_to_css(value: &Value) -> Option<String> {
    let top = value.get("top")?;
    let width = length_to_css(top.get("width"));
    let style = top
        .get("style")
        .and_then(|v| v.as_str())
        .unwrap_or("Solid")
        .to_lowercase();
    let color = top.get("color").and_then(color_to_css).unwrap_or_default();
    if width.is_empty() && color.is_empty() {
        return None;
    }
    Some(format!("{width} {style} {color}").trim().to_string())
}

/// Extract an AnimationTransition with compile-time spring solving.
fn extract_animation(value: &Value) -> Option<CompiledAnimation> {
    let id = value.get("node_id")?.as_str()?.to_string();
    let target_node_id = value.get("target_node_id")?.as_str()?.to_string();

    // Extract properties
    let properties = value
        .get("properties")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|p| {
                    let prop = p.get("property")?.as_str()?.to_string();
                    let from = p.get("from")?.as_str()?.to_string();
                    let to = p.get("to")?.as_str()?.to_string();
                    Some((prop, from, to))
                })
                .collect()
        })
        .unwrap_or_default();

    // Duration
    let duration_ms = value
        .get("duration")
        .and_then(|d| d.get("ms"))
        .and_then(|v| v.as_f64())
        .unwrap_or(300.0);

    // Easing — solve springs at compile time!
    let easing_css = if let Some(easing) = value.get("easing") {
        let easing_type = easing
            .get("easing_type")
            .and_then(|v| v.as_str())
            .unwrap_or("Linear");

        match easing_type {
            "Spring" => {
                let stiffness = easing
                    .get("stiffness")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(300.0);
                let damping = easing
                    .get("damping")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(25.0);
                let mass = easing.get("mass").and_then(|v| v.as_f64()).unwrap_or(1.0);
                // Solve the spring ODE at compile time!
                crate::animation::spring::spring_to_css_linear(stiffness, damping, mass)
            }
            "CubicBezier" => {
                let x1 = easing.get("x1").and_then(|v| v.as_f64()).unwrap_or(0.25);
                let y1 = easing.get("y1").and_then(|v| v.as_f64()).unwrap_or(0.1);
                let x2 = easing.get("x2").and_then(|v| v.as_f64()).unwrap_or(0.25);
                let y2 = easing.get("y2").and_then(|v| v.as_f64()).unwrap_or(1.0);
                format!("cubic-bezier({x1},{y1},{x2},{y2})")
            }
            _ => "ease".to_string(),
        }
    } else {
        "ease".to_string()
    };

    // Reduced motion
    let rm = value.get("reduced_motion");
    let has_reduced_motion = rm.is_some();
    let reduced_motion_strategy = rm
        .and_then(|r| r.get("strategy"))
        .and_then(|v| v.as_str())
        .unwrap_or("Remove")
        .to_string();

    Some(CompiledAnimation {
        id,
        target_node_id,
        properties,
        duration_ms,
        easing_css,
        has_reduced_motion,
        reduced_motion_strategy,
    })
}

/// Extract a StateMachine from its IR value.
fn extract_state_machine(value: &Value) -> Option<CompiledStateMachine> {
    let id = value.get("node_id")?.as_str()?.to_string();
    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&id)
        .to_string();

    let states_arr = value.get("states")?.as_array()?;
    let mut states = Vec::new();
    let mut initial_state = String::new();

    for s in states_arr {
        let state_name = s.get("name")?.as_str()?.to_string();
        if s.get("initial").and_then(|v| v.as_bool()).unwrap_or(false) {
            initial_state = state_name.clone();
        }
        states.push(state_name);
    }

    if initial_state.is_empty() && !states.is_empty() {
        initial_state = states[0].clone();
    }

    let transitions = value
        .get("transitions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    Some(CompiledTransition {
                        event: t.get("event")?.as_str()?.to_string(),
                        from: t.get("from")?.as_str()?.to_string(),
                        to: t.get("to")?.as_str()?.to_string(),
                        guard: t.get("guard").and_then(|v| v.as_str()).map(String::from),
                        effect: t.get("effect").and_then(|v| v.as_str()).map(String::from),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Some(CompiledStateMachine {
        id,
        name,
        initial_state,
        states,
        transitions,
    })
}

/// Extract inline styles from the IR value's visual properties.
fn extract_styles(value: &Value, type_name: &str) -> HashMap<String, String> {
    let mut styles = HashMap::new();

    if let Some(opacity) = value.get("opacity").and_then(|v| v.as_f64()) {
        if opacity < 1.0 {
            styles.insert("opacity".to_string(), format!("{opacity}"));
        }
    }

    let color_field = if type_name == "Surface" {
        "fill"
    } else {
        "background"
    };
    if let Some(color) = value.get(color_field) {
        if let (Some(r), Some(g), Some(b), Some(a)) = (
            color.get("r").and_then(|v| v.as_u64()),
            color.get("g").and_then(|v| v.as_u64()),
            color.get("b").and_then(|v| v.as_u64()),
            color.get("a").and_then(|v| v.as_u64()),
        ) {
            if a == 255 {
                styles.insert("background-color".to_string(), format!("rgb({r},{g},{b})"));
            } else {
                let af = a as f64 / 255.0;
                styles.insert(
                    "background-color".to_string(),
                    format!("rgba({r},{g},{b},{af:.2})"),
                );
            }
        }
    }

    if let Some(color) = value.get("color") {
        if let (Some(r), Some(g), Some(b)) = (
            color.get("r").and_then(|v| v.as_u64()),
            color.get("g").and_then(|v| v.as_u64()),
            color.get("b").and_then(|v| v.as_u64()),
        ) {
            styles.insert("color".to_string(), format!("rgb({r},{g},{b})"));
        }
    }

    if let Some(fs) = value.get("font_size") {
        if let Some(val) = fs.get("value").and_then(|v| v.as_f64()) {
            let unit = fs.get("unit").and_then(|v| v.as_str()).unwrap_or("Px");
            let css_unit = match unit {
                "Rem" => "rem",
                "Em" => "em",
                "Percent" => "%",
                _ => "px",
            };
            styles.insert("font-size".to_string(), format!("{val}{css_unit}"));
        }
    }

    if let Some(fw) = value.get("font_weight").and_then(|v| v.as_str()) {
        let weight = match fw {
            "Thin" => "100",
            "ExtraLight" => "200",
            "Light" => "300",
            "Medium" => "500",
            "SemiBold" => "600",
            "Bold" => "700",
            "ExtraBold" => "800",
            "Black" => "900",
            _ => "400",
        };
        if weight != "400" {
            styles.insert("font-weight".to_string(), weight.to_string());
        }
    }

    if let Some(ta) = value.get("text_align").and_then(|v| v.as_str()) {
        let align = match ta {
            "Center" => "center",
            "End" => "end",
            "Justify" => "justify",
            _ => "",
        };
        if !align.is_empty() {
            styles.insert("text-align".to_string(), align.to_string());
        }
    }

    if let Some(pad) = value.get("padding") {
        let top = length_to_css(pad.get("top"));
        let right = length_to_css(pad.get("right"));
        let bottom = length_to_css(pad.get("bottom"));
        let left = length_to_css(pad.get("left"));
        if !top.is_empty() || !right.is_empty() || !bottom.is_empty() || !left.is_empty() {
            styles.insert(
                "padding".to_string(),
                format!(
                    "{} {} {} {}",
                    if top.is_empty() { "0" } else { &top },
                    if right.is_empty() { "0" } else { &right },
                    if bottom.is_empty() { "0" } else { &bottom },
                    if left.is_empty() { "0" } else { &left },
                ),
            );
        }
    }

    if let Some(cr) = value.get("corner_radius") {
        let tl = length_to_css(cr.get("top_left"));
        let tr = length_to_css(cr.get("top_right"));
        let br = length_to_css(cr.get("bottom_right"));
        let bl = length_to_css(cr.get("bottom_left"));
        if !tl.is_empty() || !tr.is_empty() || !br.is_empty() || !bl.is_empty() {
            styles.insert("border-radius".to_string(), format!("{tl} {tr} {br} {bl}"));
        }
    }

    // Width / height / min / max
    let width = length_to_css(value.get("width"));
    if !width.is_empty() {
        styles.insert("width".to_string(), width);
    }
    let height = length_to_css(value.get("height"));
    if !height.is_empty() {
        styles.insert("height".to_string(), height);
    }
    let min_width = length_to_css(value.get("min_width"));
    if !min_width.is_empty() {
        styles.insert("min-width".to_string(), min_width);
    }
    let max_width = length_to_css(value.get("max_width"));
    if !max_width.is_empty() {
        styles.insert("max-width".to_string(), max_width);
    }
    let min_height = length_to_css(value.get("min_height"));
    if !min_height.is_empty() {
        styles.insert("min-height".to_string(), min_height);
    }
    let max_height = length_to_css(value.get("max_height"));
    if !max_height.is_empty() {
        styles.insert("max-height".to_string(), max_height);
    }

    // Line height
    if let Some(lh) = value.get("line_height").and_then(|v| v.as_f64()) {
        if (lh - 1.5).abs() > 0.01 {
            styles.insert("line-height".to_string(), format!("{lh}"));
        }
    }

    // Letter spacing
    let ls = length_to_css(value.get("letter_spacing"));
    if !ls.is_empty() {
        styles.insert("letter-spacing".to_string(), ls);
    }

    // Text decoration
    if let Some(td) = value.get("text_decoration").and_then(|v| v.as_str()) {
        match td {
            "Underline" => {
                styles.insert("text-decoration".to_string(), "underline".to_string());
            }
            "LineThrough" => {
                styles.insert("text-decoration".to_string(), "line-through".to_string());
            }
            _ => {}
        }
    }

    // Text overflow + max lines
    if let Some(to) = value.get("text_overflow").and_then(|v| v.as_str()) {
        if to == "Ellipsis" {
            styles.insert("text-overflow".to_string(), "ellipsis".to_string());
            styles.insert("overflow".to_string(), "hidden".to_string());
        }
    }
    if let Some(ml) = value.get("max_lines").and_then(|v| v.as_i64()) {
        if ml > 0 {
            styles.insert("display".to_string(), "-webkit-box".to_string());
            styles.insert("-webkit-line-clamp".to_string(), ml.to_string());
            styles.insert("-webkit-box-orient".to_string(), "vertical".to_string());
            styles.insert("overflow".to_string(), "hidden".to_string());
        }
    }

    // Font family
    if let Some(ff) = value.get("font_family").and_then(|v| v.as_str()) {
        if !ff.is_empty() {
            styles.insert("font-family".to_string(), format!("'{ff}',sans-serif"));
        }
    }

    // Border
    if let Some(border) = value.get("border") {
        for (side, css_side) in [
            ("top", "border-top"),
            ("right", "border-right"),
            ("bottom", "border-bottom"),
            ("left", "border-left"),
        ] {
            if let Some(b) = border.get(side) {
                let width = length_to_css(b.get("width"));
                let bstyle = b
                    .get("style")
                    .and_then(|v| v.as_str())
                    .unwrap_or("solid")
                    .to_lowercase();
                let color = b
                    .get("color")
                    .and_then(|c| {
                        let r = c.get("r")?.as_u64()?;
                        let g = c.get("g")?.as_u64()?;
                        let b_val = c.get("b")?.as_u64()?;
                        Some(format!("rgb({r},{g},{b_val})"))
                    })
                    .unwrap_or_default();
                if !width.is_empty() {
                    styles.insert(css_side.to_string(), format!("{width} {bstyle} {color}"));
                }
            }
        }
    }

    // Grid rows
    if let Some(rows) = value.get("grid_rows").and_then(|v| v.as_array()) {
        let row_vals: Vec<String> = rows
            .iter()
            .filter_map(|r| {
                let val = r.get("value")?.as_f64()?;
                let unit = r.get("unit").and_then(|u| u.as_str()).unwrap_or("Fr");
                let css_unit = match unit {
                    "Fr" => "fr",
                    "Px" => "px",
                    "Rem" => "rem",
                    "Percent" => "%",
                    _ => "fr",
                };
                Some(format!("{val}{css_unit}"))
            })
            .collect();
        if !row_vals.is_empty() {
            styles.insert("grid-template-rows".to_string(), row_vals.join(" "));
        }
    }

    // Box shadow
    if let Some(shadows) = value.get("shadow").and_then(|v| v.as_array()) {
        let shadow_parts: Vec<String> = shadows
            .iter()
            .filter_map(|s| {
                let ox = length_to_css(s.get("offset_x"));
                let oy = length_to_css(s.get("offset_y"));
                let blur = length_to_css(s.get("blur"));
                let spread = length_to_css(s.get("spread"));
                let color = s.get("color").and_then(|c| {
                    let r = c.get("r")?.as_u64()?;
                    let g = c.get("g")?.as_u64()?;
                    let b = c.get("b")?.as_u64()?;
                    let a = c.get("a")?.as_u64()?;
                    if a == 255 {
                        Some(format!("rgb({r},{g},{b})"))
                    } else {
                        Some(format!("rgba({r},{g},{b},{:.2})", a as f64 / 255.0))
                    }
                })?;
                let inset = if s.get("inset").and_then(|v| v.as_bool()).unwrap_or(false) {
                    "inset "
                } else {
                    ""
                };
                Some(format!("{inset}{ox} {oy} {blur} {spread} {color}"))
            })
            .collect();
        if !shadow_parts.is_empty() {
            styles.insert("box-shadow".to_string(), shadow_parts.join(", "));
        }
    }

    // Overflow
    if let Some(ox) = value.get("overflow_x").and_then(|v| v.as_str()) {
        if ox != "Visible" {
            styles.insert("overflow-x".to_string(), ox.to_lowercase());
        }
    }
    if let Some(oy) = value.get("overflow_y").and_then(|v| v.as_str()) {
        if oy != "Visible" {
            styles.insert("overflow-y".to_string(), oy.to_lowercase());
        }
    }

    // Grid template columns
    if let Some(cols) = value.get("grid_columns").and_then(|v| v.as_array()) {
        let col_vals: Vec<String> = cols
            .iter()
            .filter_map(|c| {
                let num = c.get("value")?.as_f64()?;
                let unit = c.get("unit").and_then(|u| u.as_str()).unwrap_or("Px");
                match unit {
                    "Fr" => Some(format!("{num}fr")),
                    "Percent" => Some(format!("{num}%")),
                    "Px" => Some(format!("{num}px")),
                    _ => Some(format!("{num}px")),
                }
            })
            .collect();
        if !col_vals.is_empty() {
            styles.insert("grid-template-columns".to_string(), col_vals.join(" "));
        }
    }

    // Absolute positioning
    if let Some(pos) = value.get("position").and_then(|v| v.as_str()) {
        if pos != "Relative" {
            styles.insert("position".to_string(), pos.to_lowercase());
        }
    }
    let top_pos = length_to_css(value.get("top"));
    if !top_pos.is_empty() {
        styles.insert("top".to_string(), top_pos);
    }
    let right_pos = length_to_css(value.get("right"));
    if !right_pos.is_empty() {
        styles.insert("right".to_string(), right_pos);
    }
    let bottom_pos = length_to_css(value.get("bottom"));
    if !bottom_pos.is_empty() {
        styles.insert("bottom".to_string(), bottom_pos);
    }
    let left_pos = length_to_css(value.get("left"));
    if !left_pos.is_empty() {
        styles.insert("left".to_string(), left_pos);
    }

    styles
}

fn length_to_css(val: Option<&Value>) -> String {
    val.and_then(|v| {
        let num = v.get("value")?.as_f64()?;
        let unit = v.get("unit").and_then(|u| u.as_str()).unwrap_or("Px");
        let css_unit = match unit {
            "Rem" => "rem",
            "Em" => "em",
            "Percent" => "%",
            "Vw" => "vw",
            "Vh" => "vh",
            "Dvh" => "dvh",
            "Svh" => "svh",
            "Fr" => "fr",
            _ => "px",
        };
        Some(format!("{num}{css_unit}"))
    })
    .unwrap_or_default()
}

fn collect_responsive_rules(
    nodes: &[crate::compiler_ir::CNode],
) -> Vec<crate::compiler_ir::CompiledResponsiveRule> {
    let mut rules = Vec::new();

    for node in nodes {
        if let NodeKind::NonVisual { type_name, data } = &node.kind {
            if type_name == "ResponsiveRule" {
                // Parse breakpoints
                let breakpoints: std::collections::HashMap<String, f64> = data
                    .get("breakpoints")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|bp| {
                                let name = bp.get("name")?.as_str()?;
                                let min_width = bp
                                    .get("min_width")
                                    .and_then(|w| w.get("value"))
                                    .and_then(|v| v.as_f64())?;
                                Some((name.to_string(), min_width))
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                // Parse overrides per breakpoint
                if let Some(overrides) = data.get("responsive_overrides").and_then(|v| v.as_array())
                {
                    for override_set in overrides {
                        let bp_name = override_set
                            .get("breakpoint_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let min_width = breakpoints.get(bp_name).copied().unwrap_or(0.0);

                        let props: Vec<(String, String, String)> = override_set
                            .get("overrides")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|o| {
                                        let target = o.get("target_node_id")?.as_str()?;
                                        let property = o.get("property")?.as_str()?;
                                        let value = o.get("value")?.as_str()?;
                                        Some((
                                            target.to_string(),
                                            property.to_string(),
                                            value.to_string(),
                                        ))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        if !props.is_empty() {
                            rules.push(crate::compiler_ir::CompiledResponsiveRule {
                                min_width_px: min_width,
                                overrides: props,
                            });
                        }
                    }
                }
            }
        }
    }

    rules
}

fn parse_rich_text_block(block: &serde_json::Value) -> crate::compiler_ir::RichTextBlock {
    crate::compiler_ir::RichTextBlock {
        block_type: block
            .get("block_type")
            .and_then(|v| v.as_str())
            .unwrap_or("Paragraph")
            .to_string(),
        level: block.get("level").and_then(|v| v.as_i64()).unwrap_or(0) as i8,
        children: block
            .get("children")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|s| crate::compiler_ir::RichTextSpan {
                        text: s
                            .get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        marks: s
                            .get("marks")
                            .and_then(|v| v.as_array())
                            .map(|marks| {
                                marks
                                    .iter()
                                    .filter_map(|m| m.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        link_url: s.get("link_url").and_then(|v| v.as_str()).map(String::from),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        media_src: block
            .get("media_src")
            .and_then(|v| v.as_str())
            .map(String::from),
        media_alt: block
            .get("media_alt")
            .and_then(|v| v.as_str())
            .map(String::from),
        code_language: block
            .get("code_language")
            .and_then(|v| v.as_str())
            .map(String::from),
        rows: block
            .get("rows")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(parse_rich_text_block).collect())
            .unwrap_or_default(),
    }
}
