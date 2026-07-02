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
    // Equal-fraction grids become auto-fit so they collapse on narrow screens.
    assert!(
        result
            .html
            .contains("grid-template-columns:repeat(auto-fit, minmax(min(100%, 367px), 1fr))"),
        "got: {}",
        result.html
    );
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
    // px font sizes become rem (respects the user's font-size preference).
    assert!(
        result.html.contains("font-size:1.5rem"),
        "got: {}",
        result.html
    );
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

#[test]
fn surface_href_icon_only_gets_synthesized_aria_label() {
    // An icon-only link: Surface with href, no semantic label, no link
    // text — only a MediaNode whose alt describes it. D3 (S82) promotes
    // that alt to an explicit aria-label on the <a>.
    let json = r#"{
        "root": {
            "node_id": "root",
            "children": [{
                "value_type": "Surface",
                "value": {
                    "node_id": "settings-link",
                    "href": "/settings",
                    "children": [
                        {
                            "value_type": "MediaNode",
                            "value": {
                                "node_id": "gear",
                                "src": "/icons/gear.svg",
                                "alt": "Settings"
                            }
                        }
                    ]
                }
            }]
        }
    }"#;

    let result = compile(json, &CompileOptions::default()).unwrap();
    assert!(
        result.html.contains("aria-label=\"Settings\""),
        "icon-only Surface link should get a synthesized aria-label, got:\n{}",
        result.html
    );
}

#[test]
fn surface_href_with_link_text_is_not_relabeled() {
    // The link has visible text — that text is the accessible name, so
    // no aria-label should be synthesized.
    let json = r#"{
        "root": {
            "node_id": "root",
            "children": [{
                "value_type": "Surface",
                "value": {
                    "node_id": "cta",
                    "href": "/signup",
                    "children": [
                        { "value_type": "TextNode", "value": { "node_id": "t", "content": "Sign up" } }
                    ]
                }
            }]
        }
    }"#;

    let result = compile(json, &CompileOptions::default()).unwrap();
    assert!(result.html.contains("Sign up"));
    assert!(
        !result.html.contains("aria-label="),
        "link with visible text should not get a synthesized aria-label, got:\n{}",
        result.html
    );
}

#[test]
fn surface_href_semantic_label_is_not_overridden() {
    // An explicit SemanticNode label always wins over the derived name.
    let json = r#"{
        "root": {
            "node_id": "root",
            "semantic_nodes": [
                { "node_id": "sem", "role": "Button", "label": "Account settings" }
            ],
            "children": [{
                "value_type": "Surface",
                "value": {
                    "node_id": "acct",
                    "href": "/account",
                    "semantic_node_id": "sem",
                    "children": [
                        {
                            "value_type": "MediaNode",
                            "value": { "node_id": "ic", "src": "/i.svg", "alt": "gear" }
                        }
                    ]
                }
            }]
        }
    }"#;

    let result = compile(json, &CompileOptions::default()).unwrap();
    assert!(result.html.contains("aria-label=\"Account settings\""));
    assert!(
        !result.html.contains("aria-label=\"gear\""),
        "semantic label must not be overridden by the MediaNode alt, got:\n{}",
        result.html
    );
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

// ─── Output escaping / injection (security) ─────────────────────

#[test]
fn javascript_href_is_neutralized() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "TextNode", "value": { "node_id": "t", "content": "Click", "href": "javascript:alert(1)" } }
        ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    assert!(!html.contains("javascript:alert(1)"));
    assert!(html.contains("href=\"#\""));
}

#[test]
fn attribute_breakout_in_href_is_escaped() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "TextNode", "value": { "node_id": "t", "content": "Click", "href": "https://x\" onclick=\"alert(1)" } }
        ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    assert!(!html.contains("onclick=\"alert(1)\""));
    assert!(html.contains("&quot;"));
}

#[test]
fn bogus_target_is_dropped() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "TextNode", "value": { "node_id": "t", "content": "Click", "href": "https://e.com", "target": "\" onload=\"x" } }
        ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    assert!(!html.contains("onload"));
}

#[test]
fn heading_link_closes_its_anchor() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "TextNode", "value": { "node_id": "h", "content": "Title", "heading_level": 2, "href": "https://e.com" } }
        ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    assert!(
        html.contains("</a></h2>"),
        "heading link must close the anchor: {html}"
    );
}

#[test]
fn jsonld_script_breakout_is_neutralized() {
    let json = r#"{
        "root": {
            "node_id": "root",
            "metadata": { "structured_data": [
                { "schema_type": "Article", "properties_json": "\"headline\":\"x</script><script>alert(1)</script>\"" }
            ] },
            "children": []
        }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    assert!(!html.contains("</script><script>alert(1)"));
    assert!(html.contains("\\u003c"));
}

#[test]
fn hostile_csp_override_falls_back_to_default() {
    let json = r#"{
        "root": {
            "node_id": "root",
            "metadata": { "content_security_policy": "default-src *; script-src 'self' 'unsafe-inline'" },
            "children": []
        }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    assert!(
        html.contains("frame-ancestors 'none'"),
        "weakening override must not be honored"
    );
}

// ─── Responsiveness ─────────────────────────────────────────────

#[test]
fn responsive_rule_emits_bounded_media_query_and_hook() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "Container", "value": { "node_id": "grid", "layout": "Grid",
                "grid_columns": [ { "value": 1, "unit": "Fr" }, { "value": 1, "unit": "Fr" }, { "value": 1, "unit": "Fr" } ],
                "children": [] } },
            { "value_type": "ResponsiveRule", "value": {
                "node_id": "rr",
                "breakpoints": [
                    { "name": "mobile", "min_width": { "value": 0.0, "unit": "Px" } },
                    { "name": "desktop", "min_width": { "value": 1024.0, "unit": "Px" } }
                ],
                "responsive_overrides": [
                    { "breakpoint_name": "mobile", "overrides": [
                        { "target_node_id": "grid", "property": "grid_columns", "value": "1fr" }
                    ] }
                ]
            } }
        ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    // Mobile range is bounded by the next breakpoint, not the broken max-width:0.
    assert!(
        !html.contains("max-width:0px"),
        "regression: max-width:0 bug"
    );
    assert!(html.contains("@media(max-width:1023.98px)"), "got: {html}");
    // IR property name mapped to CSS, and !important so it beats inline styles.
    assert!(html.contains("grid-template-columns:1fr !important"));
    // The target node got a stable hook so the selector matches something.
    assert!(html.contains("data-voce-id=\"grid\""));
}

#[test]
fn top_responsive_breakpoint_is_unbounded() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "Container", "value": { "node_id": "c", "children": [] } },
            { "value_type": "ResponsiveRule", "value": {
                "node_id": "rr",
                "breakpoints": [
                    { "name": "mobile", "min_width": { "value": 0.0, "unit": "Px" } },
                    { "name": "desktop", "min_width": { "value": 1024.0, "unit": "Px" } }
                ],
                "responsive_overrides": [
                    { "breakpoint_name": "desktop", "overrides": [
                        { "target_node_id": "c", "property": "padding", "value": "64px" }
                    ] }
                ]
            } }
        ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    assert!(html.contains("@media(min-width:1024px)"), "got: {html}");
    assert!(
        !html.contains("and (max-width"),
        "top breakpoint must be unbounded"
    );
}

#[test]
fn intrinsic_length_units_and_gap_unit_are_respected() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "Container", "value": { "node_id": "c", "layout": "Stack",
                "width": { "value": 0, "unit": "Auto" },
                "gap": { "value": 1.5, "unit": "Rem" },
                "children": [] } }
        ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    assert!(
        html.contains("width:auto"),
        "Auto must emit `auto`, not 0px: {html}"
    );
    assert!(!html.contains("width:0px"));
    assert!(
        html.contains("gap:1.5rem"),
        "gap must keep its unit: {html}"
    );
    assert!(!html.contains("gap:1.5px"));
}

#[test]
fn reduced_motion_strategies_all_reduce() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "Surface", "value": { "node_id": "a", "children": [] } },
            { "value_type": "Surface", "value": { "node_id": "b", "children": [] } },
            { "value_type": "AnimationTransition", "value": {
                "node_id": "anim-a", "target_node_id": "a",
                "properties": [ { "property": "opacity", "from": "0", "to": "1" } ],
                "duration": { "ms": 400 },
                "reduced_motion": { "strategy": "ReduceDuration", "reduced_duration": { "ms": 20 } }
            } },
            { "value_type": "AnimationTransition", "value": {
                "node_id": "anim-b", "target_node_id": "b",
                "properties": [ { "property": "opacity", "from": "0", "to": "1" } ],
                "duration": { "ms": 400 },
                "reduced_motion": { "strategy": "Simplify" }
            } }
        ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    // ReduceDuration shortens rather than removing.
    assert!(
        html.contains("[data-voce-id=\"a\"]{transition-duration:20ms!important;}"),
        "ReduceDuration must emit a shortened duration: {html}"
    );
    // Simplify (no simplified data available) falls back to removing motion,
    // never to doing nothing.
    assert!(
        html.contains("[data-voce-id=\"b\"]{transition:none!important;}"),
        "Simplify must fall back to a safe floor: {html}"
    );
}

#[test]
fn large_heading_gets_fluid_clamp() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "TextNode", "value": { "node_id": "h", "content": "Big", "heading_level": 1,
                "font_size": { "value": 56.0, "unit": "Px" } } }
        ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    // Scales from a reduced floor up to the authored size, so it never
    // overflows a narrow screen.
    assert!(html.contains("font-size:clamp("), "got: {html}");
    assert!(
        html.contains("3.5rem)"),
        "max should be the authored 56px: {html}"
    );
    assert!(!html.contains("font-size:56px"));
}

#[test]
fn equal_fraction_grid_becomes_auto_fit_but_unequal_is_preserved() {
    let equal = r#"{ "root": { "node_id": "root", "children": [
        { "value_type": "Container", "value": { "node_id": "g", "layout": "Grid",
            "grid_columns": [ {"value":1,"unit":"Fr"}, {"value":1,"unit":"Fr"} ], "children": [] } } ] } }"#;
    let html = compile(equal, &CompileOptions::default()).unwrap().html;
    assert!(
        html.contains("repeat(auto-fit, minmax(min(100%, 550px), 1fr))"),
        "got: {html}"
    );

    let unequal = r#"{ "root": { "node_id": "root", "children": [
        { "value_type": "Container", "value": { "node_id": "g", "layout": "Grid",
            "grid_columns": [ {"value":1,"unit":"Fr"}, {"value":2,"unit":"Fr"} ], "children": [] } } ] } }"#;
    let html2 = compile(unequal, &CompileOptions::default()).unwrap().html;
    assert!(
        html2.contains("grid-template-columns:1fr 2fr"),
        "unequal preserved: {html2}"
    );
    assert!(!html2.contains("auto-fit"));
}

#[test]
fn rich_text_table_is_wrapped_for_horizontal_scroll() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "RichTextNode", "value": { "node_id": "rt", "blocks": [
                { "block_type": "Table", "rows": [
                    { "block_type": "TableRow", "rows": [
                        { "block_type": "TableCell", "children": [ { "text": "A" } ] },
                        { "block_type": "TableCell", "children": [ { "text": "B" } ] }
                    ] }
                ] }
            ] } }
        ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    assert!(
        html.contains("<div style=\"overflow-x:auto\">"),
        "table must be wrapped: {html}"
    );
    assert!(html.contains("<table>"));
}

// ─── Accessibility: interactive widgets ─────────────────────────

#[test]
fn gesture_target_is_a_focusable_operable_button() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "Surface", "value": { "node_id": "toggle", "children": [
                { "value_type": "TextNode", "value": { "node_id": "lbl", "content": "Toggle" } }
            ] } },
            { "value_type": "StateMachine", "value": { "node_id": "sm", "initial_state": "off",
                "states": ["off", "on"],
                "transitions": [ { "from": "off", "event": "tap", "to": "on" } ] } },
            { "value_type": "GestureHandler", "value": { "node_id": "g", "target_node_id": "toggle",
                "gesture_type": "Tap", "trigger_event": "tap", "trigger_state_machine": "sm" } }
        ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    // The div target is now a focusable button.
    assert!(html.contains("role=\"button\""), "got: {html}");
    assert!(html.contains("tabindex=\"0\""));
    // And keyboard-operable: Enter and Space activate it, Space preventDefault'd.
    assert!(
        html.contains("e.key==='Enter'||e.key===' '"),
        "keyboard equiv missing: {html}"
    );
    assert!(html.contains("e.preventDefault()"));
}

#[test]
fn live_region_emits_aria_live_on_target() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "Container", "value": { "node_id": "status", "children": [] } },
            { "value_type": "LiveRegion", "value": { "node_id": "lr", "target_node_id": "status",
                "politeness": "Assertive", "atomic": true, "relevant": "All",
                "role_description": "Cart updates" } }
        ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    assert!(html.contains("aria-live=\"assertive\""), "got: {html}");
    assert!(html.contains("aria-atomic=\"true\""));
    assert!(html.contains("aria-relevant=\"all\""));
    assert!(html.contains("aria-roledescription=\"Cart updates\""));
}

#[test]
fn form_field_errors_are_wired_and_autocomplete_is_complete() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "FormNode", "value": { "node_id": "f", "action_endpoint": "/x", "action_method": "post",
                "fields": [
                    { "name": "addr", "field_type": "text", "label": "Address",
                      "autocomplete": "StreetAddress",
                      "validations": [ { "rule_type": "Required", "message": "Required" } ] }
                ] } }
        ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    // F10: the full autocomplete vocabulary maps (no longer collapses to off).
    assert!(
        html.contains("autocomplete=\"street-address\""),
        "got: {html}"
    );
    // F9: error container is programmatically associated with the field...
    assert!(html.contains("aria-describedby=\"f-addr-error\""));
    // ...and the JS marks invalidity + focuses the first invalid field.
    assert!(html.contains("setAttribute('aria-invalid','true')"));
    assert!(html.contains("firstInvalid.focus()"));
}

#[test]
fn form_radio_and_hidden_fields_are_labelled_correctly() {
    let json = r#"{
        "root": { "node_id": "root",
            "semantic_nodes": [ { "node_id": "form-sem", "role": "form", "label": "Signup" } ],
            "children": [
            { "value_type": "FormNode", "value": { "node_id": "f", "semantic_node_id": "form-sem",
                "action_endpoint": "/x", "action_method": "post",
                "fields": [
                    { "name": "plan", "field_type": "Radio", "label": "Plan",
                      "options": ["Free", "Pro"], "validations": [] },
                    { "name": "token", "field_type": "Hidden", "label": "Token", "validations": [] }
                ] } }
        ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    // F6: the FormNode's SemanticNode reaches the <form>.
    assert!(
        html.contains("aria-label=\"Signup\""),
        "form aria-label missing: {html}"
    );
    // F8: radio group uses fieldset+legend, not a dead `for=` label.
    assert!(html.contains("<fieldset"), "radio needs a fieldset: {html}");
    assert!(html.contains("<legend>Plan</legend>"));
    assert!(
        !html.contains("<label for=\"f-plan\">"),
        "no dead group label"
    );
    // Hidden field gets no visible label.
    assert!(
        !html.contains("<label for=\"f-token\">"),
        "hidden field must not be labelled"
    );
}

#[test]
fn main_landmark_gets_skip_link_and_focus_target() {
    let json = r#"{
        "root": { "node_id": "root",
            "semantic_nodes": [ { "node_id": "m", "role": "main" } ],
            "children": [
                { "value_type": "Container", "value": { "node_id": "content", "semantic_node_id": "m", "children": [] } }
            ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    // Skip link present and points at the main landmark.
    assert!(
        html.contains("<a class=\"skip-link\" href=\"#main\">"),
        "got: {html}"
    );
    // Main landmark is a <main> with a focusable id target.
    assert!(html.contains("<main"));
    assert!(html.contains("id=\"main\" tabindex=\"-1\""));
}

#[test]
fn no_skip_link_without_a_main_landmark() {
    let json = r#"{ "root": { "node_id": "root", "children": [
        { "value_type": "TextNode", "value": { "node_id": "t", "content": "Hi" } }
    ] } }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    assert!(
        !html.contains("skip-link\" href"),
        "no skip link without a main landmark"
    );
}

#[test]
fn semantic_node_aria_states_are_emitted() {
    let json = r#"{
        "root": { "node_id": "root",
            "semantic_nodes": [ { "node_id": "s", "role": "button", "controls": "panel",
                "aria_expanded": 0, "aria_disabled": true, "tab_index": -1,
                "custom_aria": [ { "key": "aria-haspopup", "value": "menu" } ] } ],
            "children": [
                { "value_type": "Surface", "value": { "node_id": "btn", "semantic_node_id": "s", "children": [] } }
            ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    assert!(html.contains("aria-controls=\"panel\""), "got: {html}");
    assert!(html.contains("aria-expanded=\"false\""));
    assert!(html.contains("aria-disabled=\"true\""));
    assert!(html.contains("tabindex=\"-1\""));
    assert!(html.contains("aria-haspopup=\"menu\""));
}

#[test]
fn custom_aria_rejects_non_aria_attribute_names() {
    let json = r#"{
        "root": { "node_id": "root",
            "semantic_nodes": [ { "node_id": "s", "role": "button",
                "custom_aria": [ { "key": "onclick", "value": "alert(1)" } ] } ],
            "children": [
                { "value_type": "Surface", "value": { "node_id": "btn", "semantic_node_id": "s", "children": [] } }
            ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    assert!(
        !html.contains("onclick"),
        "custom_aria must not emit arbitrary attributes: {html}"
    );
}

#[test]
fn state_machine_reflects_current_state_on_the_element() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "Surface", "value": { "node_id": "toggle", "children": [] } },
            { "value_type": "StateMachine", "value": { "node_id": "sm", "initial_state": "off",
                "states": ["off", "on"],
                "transitions": [ { "from": "off", "event": "tap", "to": "on" } ] } },
            { "value_type": "GestureHandler", "value": { "node_id": "g", "target_node_id": "toggle",
                "gesture_type": "Tap", "trigger_event": "tap", "trigger_state_machine": "sm" } }
        ] }
    }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    // Initial state is seeded and re-reflected after each transition.
    assert!(
        html.contains("setAttribute('data-state',sm.current)"),
        "got: {html}"
    );
}

// ─── Polish defaults ────────────────────────────────────────────

#[test]
fn baseline_css_has_color_scheme_and_disabled_states() {
    let json = r#"{ "root": { "node_id": "root", "children": [
        { "value_type": "TextNode", "value": { "node_id": "t", "content": "Hi" } }
    ] } }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    assert!(html.contains("color-scheme:light dark"), "got: {html}");
    assert!(html.contains("button:disabled"));
    assert!(html.contains("cursor:not-allowed"));
}

#[test]
fn form_marks_busy_and_disables_submit_on_valid_submit() {
    let json = r#"{ "root": { "node_id": "root", "children": [
        { "value_type": "FormNode", "value": { "node_id": "f", "action_endpoint": "/x", "action_method": "post",
            "fields": [ { "name": "email", "field_type": "Email", "label": "Email",
                "validations": [ { "rule_type": "Required", "message": "Required" } ] } ] } }
    ] } }"#;
    let html = compile(json, &CompileOptions::default()).unwrap().html;
    assert!(
        html.contains("setAttribute('aria-busy','true')"),
        "got: {html}"
    );
    assert!(html.contains("b.disabled=true"));
}
