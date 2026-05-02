//! Integration tests for the DOM compiler.

use voce_compiler_dom::{CompileOptions, compile};

fn load_example(path: &str) -> String {
    std::fs::read_to_string(format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("Failed to load {path}: {e}"))
}

// ─── Basic Compilation ──────────────────────────────────────────

#[test]
fn minimal_ir_compiles_to_valid_html() {
    let json = r#"{
        "root": {
            "node_id": "root",
            "children": [
                { "value_type": "TextNode", "value": { "node_id": "t", "content": "Hello World" } }
            ]
        }
    }"#;

    let result = compile(json, &CompileOptions::default()).unwrap();
    assert!(result.html.contains("<!DOCTYPE html>"));
    assert!(result.html.contains("<html"));
    assert!(result.html.contains("Hello World"));
    assert!(result.html.contains("</html>"));
}

#[test]
fn empty_root_produces_valid_html() {
    let json = r#"{ "root": { "node_id": "root" } }"#;
    let result = compile(json, &CompileOptions::default()).unwrap();
    assert!(result.html.contains("<!DOCTYPE html>"));
    assert!(result.html.contains("<body>"));
    assert!(result.html.contains("</body>"));
}

// ─── Layout ─────────────────────────────────────────────────────

#[test]
fn container_flex_row_emits_correct_css() {
    let json = r#"{
        "root": {
            "node_id": "root",
            "children": [{
                "value_type": "Container",
                "value": {
                    "node_id": "row",
                    "layout": "Flex",
                    "direction": "Row",
                    "main_align": "SpaceBetween",
                    "cross_align": "Center",
                    "gap": { "value": 16.0, "unit": "Px" }
                }
            }]
        }
    }"#;

    let result = compile(json, &CompileOptions::default()).unwrap();
    assert!(result.html.contains("flex-direction:row"));
    assert!(result.html.contains("justify-content:space-between"));
    assert!(result.html.contains("align-items:center"));
    assert!(result.html.contains("gap:16px"));
}

#[test]
fn container_grid_emits_template_columns() {
    let json = r#"{
        "root": {
            "node_id": "root",
            "children": [{
                "value_type": "Container",
                "value": {
                    "node_id": "grid",
                    "layout": "Grid",
                    "grid_columns": [
                        { "value": 1.0, "unit": "Fr" },
                        { "value": 1.0, "unit": "Fr" },
                        { "value": 1.0, "unit": "Fr" }
                    ]
                }
            }]
        }
    }"#;

    let result = compile(json, &CompileOptions::default()).unwrap();
    assert!(result.html.contains("display:grid"));
    assert!(result.html.contains("grid-template-columns:1fr 1fr 1fr"));
}

// ─── Text ───────────────────────────────────────────────────────

#[test]
fn heading_level_emits_correct_tag() {
    let json = r#"{
        "root": {
            "node_id": "root",
            "children": [
                { "value_type": "TextNode", "value": { "node_id": "h1", "content": "Title", "heading_level": 1 } },
                { "value_type": "TextNode", "value": { "node_id": "h2", "content": "Subtitle", "heading_level": 2 } },
                { "value_type": "TextNode", "value": { "node_id": "p", "content": "Body text" } }
            ]
        }
    }"#;

    let result = compile(json, &CompileOptions::default()).unwrap();
    assert!(result.html.contains("<h1"));
    assert!(result.html.contains("Title</h1>"));
    assert!(result.html.contains("<h2"));
    assert!(result.html.contains("Subtitle</h2>"));
    assert!(result.html.contains("<p"));
    assert!(result.html.contains("Body text</p>"));
}

#[test]
fn text_styles_emit_correctly() {
    let json = r#"{
        "root": {
            "node_id": "root",
            "children": [{
                "value_type": "TextNode",
                "value": {
                    "node_id": "styled",
                    "content": "Bold text",
                    "font_size": { "value": 24.0, "unit": "Px" },
                    "font_weight": "Bold",
                    "text_align": "Center",
                    "color": { "r": 255, "g": 0, "b": 0, "a": 255 }
                }
            }]
        }
    }"#;

    let result = compile(json, &CompileOptions::default()).unwrap();
    assert!(result.html.contains("font-size:24px"));
    assert!(result.html.contains("font-weight:700"));
    assert!(result.html.contains("text-align:center"));
    assert!(result.html.contains("color:rgb(255,0,0)"));
}

// ─── Surface ────────────────────────────────────────────────────

#[test]
fn surface_decorative_has_aria_hidden() {
    let json = r#"{
        "root": {
            "node_id": "root",
            "children": [{
                "value_type": "Surface",
                "value": {
                    "node_id": "bg",
                    "decorative": true,
                    "fill": { "r": 0, "g": 0, "b": 0, "a": 255 }
                }
            }]
        }
    }"#;

    let result = compile(json, &CompileOptions::default()).unwrap();
    assert!(result.html.contains("role=\"presentation\""));
    assert!(result.html.contains("aria-hidden=\"true\""));
    assert!(result.html.contains("background-color:rgb(0,0,0)"));
}

// ─── Media ──────────────────────────────────────────────────────

#[test]
fn media_node_emits_img_with_lazy_loading() {
    let json = r#"{
        "root": {
            "node_id": "root",
            "children": [{
                "value_type": "MediaNode",
                "value": {
                    "node_id": "hero-img",
                    "src": "/images/hero.jpg",
                    "alt": "Hero image",
                    "above_fold": true
                }
            }]
        }
    }"#;

    let result = compile(json, &CompileOptions::default()).unwrap();
    assert!(result.html.contains("src=\"/images/hero.jpg\""));
    assert!(result.html.contains("alt=\"Hero image\""));
    assert!(result.html.contains("loading=\"eager\""));
    assert!(result.html.contains("fetchpriority=\"high\""));
}

// ─── SEO & Security ─────────────────────────────────────────────

#[test]
fn metadata_emits_head_tags() {
    let json = r#"{
        "root": {
            "node_id": "root",
            "metadata": {
                "title": "My Page",
                "description": "A great page",
                "canonical_url": "https://example.com",
                "open_graph": {
                    "title": "My Page OG",
                    "image": "https://example.com/og.png"
                }
            }
        }
    }"#;

    let result = compile(json, &CompileOptions::default()).unwrap();
    assert!(result.html.contains("<title>My Page</title>"));
    assert!(result.html.contains("content=\"A great page\""));
    assert!(result.html.contains("href=\"https://example.com\""));
    assert!(result.html.contains("og:title"));
    assert!(result.html.contains("og:image"));
}

#[test]
fn security_headers_always_present() {
    let json = r#"{ "root": { "node_id": "root" } }"#;
    let result = compile(json, &CompileOptions::default()).unwrap();
    assert!(result.html.contains("X-Content-Type-Options"));
    assert!(result.html.contains("X-Frame-Options"));
    assert!(result.html.contains("nosniff"));
    assert!(result.html.contains("DENY"));
}

// ─── Landing Page End-to-End ────────────────────────────────────

#[test]
fn landing_page_compiles_under_12kb() {
    // Budget bumped from 10 KB → 12 KB in S64. The compiler now emits
    // baseline typography/list/code/blockquote/hr/table CSS plus a fallback
    // theme palette with prefers-color-scheme; ~1.7 KB of additional stylesheet
    // applies to every compiled page. Trade-off explicitly accepted: the
    // budget grew, but every output now looks presentable by default.
    let json = load_example("examples/landing-page/landing-page.voce.json");
    let result = compile(&json, &CompileOptions::default()).unwrap();
    assert!(
        result.size_bytes < 12_000,
        "Landing page should be under 12KB, got {} bytes",
        result.size_bytes
    );
}

#[test]
fn landing_page_has_valid_structure() {
    let json = load_example("examples/landing-page/landing-page.voce.json");
    let result = compile(&json, &CompileOptions::default()).unwrap();
    assert!(result.html.contains("<!DOCTYPE html>"));
    assert!(result.html.contains("<h1"));
    assert!(result.html.contains("<h2"));
    assert!(result.html.contains("X-Frame-Options"));
    assert!(result.html.contains("og:title"));
    assert!(result.html.contains("The code is gone."));
    assert!(result.html.contains("grid-template-columns"));
    assert!(result.html.contains("sotto voce"));
}

// ─── Output Size ────────────────────────────────────────────────

#[test]
fn size_bytes_matches_html_length() {
    let json = r#"{ "root": { "node_id": "root" } }"#;
    let result = compile(json, &CompileOptions::default()).unwrap();
    assert_eq!(result.size_bytes, result.html.len());
}

// ─── S70 Day 1: CSP hardening ───────────────────────────────────

#[test]
fn default_csp_includes_hardened_directives() {
    // Plain IR — no inline scripts. CSP should have all four new directives.
    let json = r#"{ "root": { "node_id": "root" } }"#;
    let result = compile(json, &CompileOptions::default()).unwrap();
    assert!(
        result.html.contains("frame-ancestors 'none'"),
        "frame-ancestors missing"
    );
    assert!(result.html.contains("base-uri 'self'"), "base-uri missing");
    assert!(
        result.html.contains("form-action 'self'"),
        "form-action missing"
    );
}

#[test]
fn default_csp_drops_unsafe_inline_for_scripts() {
    let json = r#"{ "root": { "node_id": "root" } }"#;
    let result = compile(json, &CompileOptions::default()).unwrap();
    let csp_line = result
        .html
        .lines()
        .find(|l| l.contains("Content-Security-Policy"))
        .expect("CSP meta tag present");
    // Slice out the script-src segment to avoid matching style-src's 'unsafe-inline'.
    let after_script = csp_line.split("script-src").nth(1).unwrap();
    let script_segment = after_script.split(';').next().unwrap();
    assert!(
        !script_segment.contains("'unsafe-inline'"),
        "script-src should not allow inline scripts; got: {script_segment}"
    );
}

#[test]
fn csp_includes_sha256_for_inline_script_when_emitted() {
    // GestureHandler causes the compiler to emit interactive JS — the script
    // body must round-trip through the CSP as a sha256 source.
    let json = r#"{
        "root": {
            "node_id": "root",
            "children": [
                {
                    "value_type": "GestureHandler",
                    "value": {
                        "node_id": "tap-handler",
                        "gesture_type": "Tap",
                        "target_node_id": "btn",
                        "keyboard_equivalent": "Enter"
                    }
                },
                {
                    "value_type": "TextNode",
                    "value": { "node_id": "btn", "content": "click me" }
                }
            ]
        }
    }"#;
    let result = compile(json, &CompileOptions::default()).unwrap();
    let csp_line = result
        .html
        .lines()
        .find(|l| l.contains("Content-Security-Policy"))
        .expect("CSP meta tag");
    assert!(
        csp_line.contains("'sha256-"),
        "expected sha256 hash in script-src; got: {csp_line}"
    );
}

#[test]
fn page_metadata_csp_override_replaces_default() {
    let json = r#"{
        "root": {
            "node_id": "root",
            "metadata": {
                "title": "x",
                "content_security_policy": "default-src 'none'; img-src https:"
            },
            "children": []
        }
    }"#;
    let result = compile(json, &CompileOptions::default()).unwrap();
    let csp_line = result
        .html
        .lines()
        .find(|l| l.contains("Content-Security-Policy"))
        .expect("CSP meta tag");
    assert!(
        csp_line.contains("default-src 'none'") && csp_line.contains("img-src https:"),
        "override didn't take effect; got: {csp_line}"
    );
    // Override is verbatim — no hardened directives merged in.
    assert!(
        !csp_line.contains("frame-ancestors"),
        "override should NOT auto-merge defaults"
    );
}

#[test]
fn empty_csp_override_falls_back_to_default() {
    let json = r#"{
        "root": {
            "node_id": "root",
            "metadata": { "title": "x", "content_security_policy": "   " },
            "children": []
        }
    }"#;
    let result = compile(json, &CompileOptions::default()).unwrap();
    assert!(
        result.html.contains("frame-ancestors 'none'"),
        "blank override should not suppress the hardened default"
    );
}
