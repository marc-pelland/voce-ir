//! Voce IR Email Compiler — compiles IR to email-safe HTML.
//!
//! Email HTML is notoriously constrained:
//! - Table-based layout (no flexbox/grid)
//! - Inline CSS only (no <style> blocks in many clients)
//! - No JavaScript
//! - Limited CSS support varies by email client
//!
//! This compiler maps IR to table layouts with inline styles,
//! tested against common email clients (Gmail, Outlook, Apple Mail).

use anyhow::{Context, Result};
use serde_json::Value;

pub struct EmailResult {
    pub html: String,
    pub size_bytes: usize,
}

/// Compile IR JSON to email-safe HTML.
pub fn compile_email(json: &str) -> Result<EmailResult> {
    let doc: Value = serde_json::from_str(json).context("Failed to parse IR")?;
    let mut html = String::with_capacity(4096);

    // Email DOCTYPE and wrapper
    html.push_str("<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Transitional//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-transitional.dtd\">\n");
    html.push_str("<html xmlns=\"http://www.w3.org/1999/xhtml\">\n<head>\n");
    html.push_str("<meta http-equiv=\"Content-Type\" content=\"text/html; charset=UTF-8\" />\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n");

    // Title from metadata
    if let Some(root) = doc.get("root") {
        if let Some(meta) = root.get("metadata") {
            if let Some(title) = meta.get("title").and_then(|v| v.as_str()) {
                html.push_str(&format!("<title>{}</title>\n", esc_html(title)));
            }
        }
    }

    html.push_str("</head>\n<body style=\"margin:0;padding:0;background-color:#f4f4f4\">\n");

    // Outer table (email wrapper)
    html.push_str("<table role=\"presentation\" cellpadding=\"0\" cellspacing=\"0\" width=\"100%\" style=\"background-color:#f4f4f4\">\n<tr>\n<td align=\"center\">\n");

    // Content table (600px max width — email standard)
    html.push_str("<table role=\"presentation\" cellpadding=\"0\" cellspacing=\"0\" width=\"600\" style=\"max-width:600px;background-color:#ffffff\">\n");

    if let Some(root) = doc.get("root") {
        if let Some(children) = root.get("children").and_then(|v| v.as_array()) {
            for child in children {
                emit_email_node(&mut html, child);
            }
        }
    }

    html.push_str("</table>\n</td>\n</tr>\n</table>\n</body>\n</html>\n");

    let size = html.len();
    Ok(EmailResult {
        html,
        size_bytes: size,
    })
}

fn emit_email_node(html: &mut String, child: &Value) {
    let type_name = child
        .get("value_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let value = child.get("value").cloned().unwrap_or(Value::Null);

    match type_name {
        "Container" => {
            let bg = color_style(&value, "background");
            let padding = padding_style(&value);

            html.push_str(&format!("<tr>\n<td style=\"{bg}{padding}\">\n"));

            // Nested table for columns
            let direction = value
                .get("direction")
                .and_then(|v| v.as_str())
                .unwrap_or("Column");
            if direction == "Row" {
                html.push_str("<table role=\"presentation\" cellpadding=\"0\" cellspacing=\"0\" width=\"100%\"><tr>\n");
                if let Some(children) = value.get("children").and_then(|v| v.as_array()) {
                    let col_width = 100 / children.len().max(1);
                    for c in children {
                        html.push_str(&format!("<td width=\"{col_width}%\" valign=\"top\">\n"));
                        emit_email_node(html, c);
                        html.push_str("</td>\n");
                    }
                }
                html.push_str("</tr></table>\n");
            } else if let Some(children) = value.get("children").and_then(|v| v.as_array()) {
                html.push_str("<table role=\"presentation\" cellpadding=\"0\" cellspacing=\"0\" width=\"100%\">\n");
                for c in children {
                    emit_email_node(html, c);
                }
                html.push_str("</table>\n");
            }

            html.push_str("</td>\n</tr>\n");
        }
        "TextNode" => {
            let content = esc_html(value.get("content").and_then(|v| v.as_str()).unwrap_or(""));
            let heading = value
                .get("heading_level")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let size = value
                .get("font_size")
                .and_then(|f| f.get("value"))
                .and_then(|v| v.as_f64())
                .unwrap_or(16.0);
            let weight = value
                .get("font_weight")
                .and_then(|v| v.as_str())
                .unwrap_or("Regular");
            let color = color_style(&value, "color");
            let align = value
                .get("text_align")
                .and_then(|v| v.as_str())
                .unwrap_or("Start");
            let text_align = match align {
                "Center" => "center",
                "End" => "right",
                _ => "left",
            };

            let fw = match weight {
                "Bold" | "SemiBold" | "ExtraBold" | "Black" => "bold",
                _ => "normal",
            };

            let tag = if (1..=6).contains(&heading) {
                format!("h{heading}")
            } else {
                "p".to_string()
            };

            // A TextNode with an href is a link — wrap the content in an
            // email-safe anchor (a heading link stays <hN><a>…</a></hN>).
            let href = value.get("href").and_then(|v| v.as_str()).unwrap_or("");
            let inner = if href.is_empty() {
                content.clone()
            } else {
                let target = value
                    .get("target")
                    .and_then(|v| v.as_str())
                    .filter(|t| VALID_TARGETS.contains(t));
                let target_attr = target
                    .map(|t| format!(" target=\"{t}\""))
                    .unwrap_or_default();
                let rel = if target == Some("_blank") {
                    " rel=\"noopener noreferrer\""
                } else {
                    ""
                };
                format!(
                    "<a href=\"{}\"{target_attr}{rel} style=\"color:inherit\">{content}</a>",
                    safe_href(href)
                )
            };

            html.push_str(&format!(
                "<tr><td style=\"font-family:Arial,Helvetica,sans-serif;font-size:{size:.0}px;font-weight:{fw};{color}text-align:{text_align};padding:4px 0\"><{tag} style=\"margin:0\">{inner}</{tag}></td></tr>\n"
            ));
        }
        "Surface" => {
            let bg = color_style_field(&value, "fill");
            let padding = padding_style(&value);
            let radius = value
                .get("corner_radius")
                .and_then(|cr| cr.get("top_left"))
                .and_then(|v| v.get("value"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let border_radius = if radius > 0.0 {
                format!("border-radius:{radius:.0}px;")
            } else {
                String::new()
            };

            html.push_str(&format!(
                "<tr><td style=\"{bg}{padding}{border_radius}\">\n"
            ));
            // A Surface with an href is a clickable block (button/card
            // link) — wrap its content in a block-level anchor.
            let href = value.get("href").and_then(|v| v.as_str()).unwrap_or("");
            let has_href = !href.is_empty();
            if has_href {
                let target = value
                    .get("target")
                    .and_then(|v| v.as_str())
                    .filter(|t| VALID_TARGETS.contains(t));
                let target_attr = target
                    .map(|t| format!(" target=\"{t}\""))
                    .unwrap_or_default();
                let rel = if target == Some("_blank") {
                    " rel=\"noopener noreferrer\""
                } else {
                    ""
                };
                html.push_str(&format!(
                    "<a href=\"{}\"{target_attr}{rel} style=\"display:block;text-decoration:none;color:inherit\">\n",
                    safe_href(href)
                ));
            }
            if let Some(children) = value.get("children").and_then(|v| v.as_array()) {
                html.push_str("<table role=\"presentation\" cellpadding=\"0\" cellspacing=\"0\" width=\"100%\">\n");
                for c in children {
                    emit_email_node(html, c);
                }
                html.push_str("</table>\n");
            }
            if has_href {
                html.push_str("</a>\n");
            }
            html.push_str("</td></tr>\n");
        }
        "MediaNode" => {
            let src = value.get("src").and_then(|v| v.as_str()).unwrap_or("");
            let alt = value.get("alt").and_then(|v| v.as_str()).unwrap_or("");
            html.push_str(&format!(
                "<tr><td><img src=\"{}\" alt=\"{}\" style=\"display:block;max-width:100%;height:auto\" /></td></tr>\n",
                safe_src(src),
                esc_attr(alt)
            ));
        }
        _ => {}
    }
}

/// Valid `target` attribute values. A `target` outside this set is dropped so
/// an IR string can neither break out of the attribute nor defeat the
/// `rel="noopener"` auto-emission for `_blank`.
const VALID_TARGETS: &[&str] = &["_self", "_blank", "_parent", "_top"];

/// Escape a string for HTML text content. IR content is AI- or user-authored
/// and must never be trusted as markup.
fn esc_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Escape a string for a double-quoted HTML attribute value.
fn esc_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// True for URL schemes that can execute code (mirrors the validator's SEC
/// pass). Any non-image `data:` URL is treated as suspect.
fn is_dangerous_scheme(url: &str) -> bool {
    let t = url.trim_start().to_ascii_lowercase();
    t.starts_with("javascript:")
        || t.starts_with("vbscript:")
        || (t.starts_with("data:") && !t.starts_with("data:image/"))
}

/// Return an attribute-safe `href`, collapsing dangerous-scheme URLs to `#`.
fn safe_href(url: &str) -> String {
    if is_dangerous_scheme(url) {
        "#".to_string()
    } else {
        esc_attr(url)
    }
}

/// Return an attribute-safe image `src`, dropping dangerous-scheme URLs.
fn safe_src(url: &str) -> String {
    if is_dangerous_scheme(url) {
        String::new()
    } else {
        esc_attr(url)
    }
}

fn color_style(value: &Value, field: &str) -> String {
    color_style_field(value, field)
}

fn color_style_field(value: &Value, field: &str) -> String {
    value
        .get(field)
        .and_then(|c| {
            let r = c.get("r")?.as_u64()?;
            let g = c.get("g")?.as_u64()?;
            let b = c.get("b")?.as_u64()?;
            let prop = if field == "color" {
                "color"
            } else {
                "background-color"
            };
            Some(format!("{prop}:rgb({r},{g},{b});"))
        })
        .unwrap_or_default()
}

fn padding_style(value: &Value) -> String {
    value
        .get("padding")
        .map(|p| {
            let top = p
                .get("top")
                .and_then(|v| v.get("value"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let right = p
                .get("right")
                .and_then(|v| v.get("value"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let bottom = p
                .get("bottom")
                .and_then(|v| v.get("value"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let left = p
                .get("left")
                .and_then(|v| v.get("value"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            format!("padding:{top:.0}px {right:.0}px {bottom:.0}px {left:.0}px;")
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_content_is_html_escaped() {
        let ir = r#"{"root":{"children":[
            {"value_type":"TextNode","value":{"content":"<script>alert(1)</script>"}}
        ]}}"#;
        let out = compile_email(ir).unwrap().html;
        assert!(!out.contains("<script>alert(1)"));
        assert!(out.contains("&lt;script&gt;alert(1)"));
    }

    #[test]
    fn javascript_href_is_neutralized() {
        let ir = r#"{"root":{"children":[
            {"value_type":"TextNode","value":{"content":"x","href":"javascript:alert(1)"}}
        ]}}"#;
        let out = compile_email(ir).unwrap().html;
        assert!(!out.contains("javascript:alert(1)"));
        assert!(out.contains("href=\"#\""));
    }

    #[test]
    fn attribute_breakout_in_href_is_escaped() {
        let ir = r#"{"root":{"children":[
            {"value_type":"TextNode","value":{"content":"x","href":"https://x\" onmouseover=\"alert(1)"}}
        ]}}"#;
        let out = compile_email(ir).unwrap().html;
        assert!(!out.contains("onmouseover=\"alert(1)\""));
        assert!(out.contains("&quot;"));
    }

    #[test]
    fn bogus_target_is_dropped() {
        let ir = r#"{"root":{"children":[
            {"value_type":"TextNode","value":{"content":"x","href":"https://e.com","target":"\" onload=\"x"}}
        ]}}"#;
        let out = compile_email(ir).unwrap().html;
        assert!(!out.contains("onload"));
    }

    #[test]
    fn image_src_javascript_is_dropped() {
        let ir = r#"{"root":{"children":[
            {"value_type":"MediaNode","value":{"src":"javascript:alert(1)","alt":"a"}}
        ]}}"#;
        let out = compile_email(ir).unwrap().html;
        assert!(!out.contains("javascript:alert(1)"));
        assert!(out.contains("src=\"\""));
    }
}
