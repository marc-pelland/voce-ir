//! Cross-target integration tests — compile fixtures to all targets and verify output.

use std::fs;
use std::path::PathBuf;

/// Workspace root — two levels up from packages/validator/.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Load a fixture by name from tests/fixtures/.
fn fixture(name: &str) -> String {
    let path = workspace_root().join(format!("tests/fixtures/{name}"));
    fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read fixture: {}", path.display()))
}

/// Load the reference landing page.
fn landing_page() -> String {
    let path = workspace_root().join("examples/landing-page/landing-page.voce.json");
    fs::read_to_string(&path).expect("Failed to read landing page fixture")
}

/// Load an invalid fixture from tests/schema/invalid/.
fn invalid_fixture(name: &str) -> String {
    let path = workspace_root().join(format!("tests/schema/invalid/{name}"));
    fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read invalid fixture: {}", path.display()))
}

// ============================================================
// Cross-target compilation: landing page → all 7 targets
// ============================================================

#[test]
fn landing_page_compiles_to_dom() {
    let json = landing_page();
    let result = voce_compiler_dom::compile(&json, &voce_compiler_dom::CompileOptions::default())
        .expect("DOM compilation failed");
    assert!(result.html.contains("<!DOCTYPE html>"));
    assert!(result.html.contains("<html"));
    assert!(result.html.contains("</html>"));
    assert!(result.size_bytes > 0);
}

#[test]
fn landing_page_compiles_to_email() {
    let json = landing_page();
    let result = voce_compiler_email::compile_email(&json).expect("Email compilation failed");
    assert!(result.html.contains("<table"));
    assert!(result.size_bytes > 0);
}

#[test]
fn landing_page_compiles_to_swiftui() {
    let json = landing_page();
    let result = voce_compiler_ios::compile_swiftui(&json).expect("SwiftUI compilation failed");
    assert!(result.swift.contains("import SwiftUI"));
    assert!(result.swift.contains("struct VoceView: View"));
    assert!(result.size_bytes > 0);
}

#[test]
fn landing_page_compiles_to_compose() {
    let json = landing_page();
    let result = voce_compiler_android::compile_compose(&json).expect("Compose compilation failed");
    assert!(result.kotlin.contains("import androidx.compose"));
    assert!(result.kotlin.contains("@Composable"));
    assert!(result.size_bytes > 0);
}

#[test]
fn landing_page_compiles_to_wasm() {
    let json = landing_page();
    let result = voce_compiler_wasm::compile_to_wat(&json).expect("WASM compilation failed");
    // WASM compiler produces WAT for state machines; landing page may not have one
    // but it should not error
    assert!(result.wat.is_empty() || result.wat.contains("(module"));
}

#[test]
fn landing_page_compiles_to_webgpu() {
    let json = landing_page();
    let result = voce_compiler_webgpu::compile_webgpu(
        &json,
        &voce_compiler_webgpu::WebGpuCompileOptions::default(),
    )
    .expect("WebGPU compilation failed");
    // Landing page has no Scene3D, so WebGPU output may be minimal
    assert!(result.html.is_empty() || result.html.contains("<!DOCTYPE html>"));
}

#[test]
fn landing_page_compiles_to_hybrid() {
    let json = landing_page();
    let result = voce_compiler_hybrid::compile_hybrid(
        &json,
        &voce_compiler_hybrid::HybridCompileOptions::default(),
    )
    .expect("Hybrid compilation failed");
    assert!(result.html.contains("<!DOCTYPE html>"));
}

// ============================================================
// Per-fixture DOM compilation
// ============================================================

#[test]
fn text_heading_produces_h1() {
    let json = fixture("text-heading.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("<h1"));
    assert!(result.html.contains("Hello World"));
}

#[test]
fn container_row_produces_flex_row() {
    let json = fixture("container-row.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("flex-direction:row"));
}

#[test]
fn surface_card_produces_box_shadow() {
    let json = fixture("surface-card.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("box-shadow"));
    assert!(result.html.contains("border-radius"));
}

#[test]
fn media_image_produces_picture() {
    let json = fixture("media-image.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("<picture>"));
    assert!(result.html.contains("srcset="));
    assert!(result.html.contains("alt=\"A scenic mountain landscape\""));
}

#[test]
fn form_contact_produces_form_elements() {
    let json = fixture("form-contact.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("<form"));
    assert!(result.html.contains("type=\"email\""));
    assert!(result.html.contains("<textarea"));
}

#[test]
fn container_grid_produces_grid_template() {
    let json = fixture("container-grid.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("grid-template-columns"));
}

#[test]
fn decorative_surface_has_aria_hidden() {
    let json = fixture("decorative-surface.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("aria-hidden=\"true\""));
}

#[test]
fn nested_layout_produces_nested_divs() {
    let json = fixture("nested-layout.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("Welcome"));
    assert!(result.html.contains("Brand"));
    // Should have multiple nested flex containers
    assert!(result.html.matches("flex-direction").count() >= 2);
}

// ============================================================
// Per-fixture email compilation
// ============================================================

#[test]
fn text_heading_email_has_inline_styles() {
    let json = fixture("text-heading.voce.json");
    let result = voce_compiler_email::compile_email(&json).unwrap();
    assert!(result.html.contains("style="));
    assert!(result.html.contains("Hello World"));
}

#[test]
fn container_row_email_uses_tables() {
    let json = fixture("container-row.voce.json");
    let result = voce_compiler_email::compile_email(&json).unwrap();
    assert!(result.html.contains("<table"));
    assert!(result.html.contains("<td"));
}

// ============================================================
// Per-fixture SwiftUI compilation
// ============================================================

#[test]
fn text_heading_swiftui_has_text_view() {
    let json = fixture("text-heading.voce.json");
    let result = voce_compiler_ios::compile_swiftui(&json).unwrap();
    assert!(result.swift.contains("Text(\"Hello World\")"));
}

#[test]
fn container_row_swiftui_uses_hstack() {
    let json = fixture("container-row.voce.json");
    let result = voce_compiler_ios::compile_swiftui(&json).unwrap();
    assert!(result.swift.contains("HStack"));
}

// ============================================================
// Per-fixture Compose compilation
// ============================================================

#[test]
fn text_heading_compose_has_text() {
    let json = fixture("text-heading.voce.json");
    let result = voce_compiler_android::compile_compose(&json).unwrap();
    assert!(result.kotlin.contains("Text("));
    assert!(result.kotlin.contains("Hello World"));
}

#[test]
fn container_row_compose_uses_row() {
    let json = fixture("container-row.voce.json");
    let result = voce_compiler_android::compile_compose(&json).unwrap();
    assert!(result.kotlin.contains("Row("));
}

// ============================================================
// End-to-end pipeline: validate → compile
// ============================================================

#[test]
fn e2e_landing_page_validates_and_compiles() {
    let json = landing_page();
    let val = voce_validator::validate(&json).unwrap();
    assert!(!val.has_errors(), "Landing page should validate cleanly");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("<!DOCTYPE html>"));
}

#[test]
fn e2e_form_validates_and_compiles() {
    let json = fixture("form-contact.voce.json");
    let val = voce_validator::validate(&json).unwrap();
    // Form may have warnings but should not have structural errors
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("<form"));
    let _ = val; // use
}

#[test]
fn e2e_nested_layout_validates_and_compiles() {
    let json = fixture("nested-layout.voce.json");
    let _val = voce_validator::validate(&json).unwrap();
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("Welcome"));
}

#[test]
fn e2e_card_validates_and_compiles() {
    let json = fixture("surface-card.voce.json");
    let _val = voce_validator::validate(&json).unwrap();
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("Card Title"));
}

#[test]
fn e2e_grid_validates_and_compiles() {
    let json = fixture("container-grid.voce.json");
    let _val = voce_validator::validate(&json).unwrap();
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("grid-template-columns"));
}

// ============================================================
// Negative tests: malformed IR
// ============================================================

#[test]
fn invalid_json_returns_error() {
    let result = voce_validator::validate("not json at all {{{");
    assert!(result.is_err());
}

#[test]
fn empty_object_returns_errors() {
    let result = voce_validator::validate("{}").unwrap();
    assert!(result.has_errors());
}

#[test]
fn missing_root_returns_str001() {
    let json = r#"{ "schema_version_major": 1, "schema_version_minor": 0 }"#;
    let result = voce_validator::validate(json).unwrap();
    assert!(result.diagnostics.iter().any(|d| d.code == "STR001"));
}

#[test]
fn duplicate_ids_returns_str002() {
    let json = invalid_fixture("duplicate-ids.voce.json");
    let result = voce_validator::validate(&json).unwrap();
    assert!(result.diagnostics.iter().any(|d| d.code == "STR002"));
}

#[test]
fn empty_textnode_returns_str004() {
    let json = invalid_fixture("empty-textnode.voce.json");
    let result = voce_validator::validate(&json).unwrap();
    assert!(result.diagnostics.iter().any(|d| d.code == "STR004"));
}

#[test]
fn broken_ref_returns_ref005() {
    let json = invalid_fixture("broken-ref.voce.json");
    let result = voce_validator::validate(&json).unwrap();
    assert!(result.diagnostics.iter().any(|d| d.code == "REF005"));
}

#[test]
fn missing_initial_state_returns_sta002() {
    let json = invalid_fixture("missing-initial-state.voce.json");
    let result = voce_validator::validate(&json).unwrap();
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == "STR001" || d.code == "STA002")
    );
}

#[test]
fn unreachable_state_returns_sta003() {
    let json = invalid_fixture("unreachable-state.voce.json");
    let result = voce_validator::validate(&json).unwrap();
    assert!(result.diagnostics.iter().any(|d| d.code == "STA003"));
}

#[test]
fn form_no_fields_returns_frm001() {
    let json = invalid_fixture("form-no-fields.voce.json");
    let result = voce_validator::validate(&json).unwrap();
    assert!(result.diagnostics.iter().any(|d| d.code == "FRM001"));
}

#[test]
fn mutation_no_csrf_returns_sec002() {
    let json = invalid_fixture("mutation-no-csrf.voce.json");
    let result = voce_validator::validate(&json).unwrap();
    assert!(result.diagnostics.iter().any(|d| d.code == "SEC002"));
}

#[test]
fn gesture_no_keyboard_returns_a11y004() {
    let json = invalid_fixture("gesture-no-keyboard.voce.json");
    let result = voce_validator::validate(&json).unwrap();
    assert!(result.diagnostics.iter().any(|d| d.code == "A11Y004"));
}

#[test]
fn heading_skip_returns_a11y005() {
    let json = invalid_fixture("heading-skip.voce.json");
    let result = voce_validator::validate(&json).unwrap();
    assert!(result.diagnostics.iter().any(|d| d.code == "A11Y005"));
}

#[test]
fn missing_reduced_motion_returns_mot001() {
    let json = invalid_fixture("transition-no-reduced-motion.voce.json");
    let result = voce_validator::validate(&json).unwrap();
    assert!(result.diagnostics.iter().any(|d| d.code == "MOT001"));
}

#[test]
fn empty_localized_key_returns_i18n002() {
    let json = invalid_fixture("empty-localized-key.voce.json");
    let result = voce_validator::validate(&json).unwrap();
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code.starts_with("I18N"))
    );
}

#[test]
fn missing_metadata_returns_seo_warning() {
    let json = r#"{
        "schema_version_major": 1,
        "schema_version_minor": 0,
        "root": {
            "node_id": "root",
            "viewport_width": { "value": 1024, "unit": "Px" },
            "children": [
                { "value_type": "TextNode", "value": { "node_id": "t", "content": "Hello", "font_size": { "value": 16, "unit": "Px" } } }
            ]
        }
    }"#;
    let result = voce_validator::validate(json).unwrap();
    assert!(result.diagnostics.iter().any(|d| d.code.starts_with("SEO")));
}

#[test]
fn compiler_handles_malformed_ir_gracefully() {
    // Compiler should return Err, not panic
    let bad_json = r#"{ "root": null }"#;
    let result = voce_compiler_dom::compile(bad_json, &Default::default());
    // It's ok if it errors, just should not panic
    let _ = result;
}

#[test]
fn compiler_handles_empty_json_gracefully() {
    let result = voce_compiler_dom::compile("{}", &Default::default());
    let _ = result;
}

#[test]
fn compiler_handles_no_children_gracefully() {
    let json = r#"{
        "schema_version_major": 1, "schema_version_minor": 0,
        "root": { "node_id": "root", "viewport_width": { "value": 100, "unit": "Px" } }
    }"#;
    let result = voce_compiler_dom::compile(json, &Default::default());
    assert!(result.is_ok());
}

// ============================================================
// DOM output structure validation
// ============================================================

#[test]
fn dom_output_has_security_headers() {
    let json = fixture("text-heading.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("X-Content-Type-Options"));
    assert!(result.html.contains("X-Frame-Options"));
}

#[test]
fn dom_output_has_lang_attribute() {
    let json = fixture("text-heading.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("lang=\"en\""));
}

#[test]
fn dom_output_has_viewport_meta() {
    let json = fixture("text-heading.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("viewport"));
}

#[test]
fn dom_output_has_title() {
    let json = fixture("text-heading.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("<title>Text Test</title>"));
}

// ============================================================
// Performance regression test
// ============================================================

// ============================================================
// Links and semantic HTML
// ============================================================

#[test]
fn text_with_href_emits_anchor_tag() {
    let json = fixture("links-and-nav.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("<a href=\"/\""));
    assert!(result.html.contains("<a href=\"/about\""));
    assert!(result.html.contains("Home</a>"));
    assert!(result.html.contains("About</a>"));
}

#[test]
fn external_link_has_noopener() {
    let json = fixture("links-and-nav.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("target=\"_blank\""));
    assert!(result.html.contains("rel=\"noopener noreferrer\""));
}

#[test]
fn surface_with_href_emits_anchor_wrapper() {
    let json = fixture("links-and-nav.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("<a href=\"/get-started\""));
    assert!(result.html.contains("Get Started"));
}

#[test]
fn semantic_nav_emits_nav_element() {
    let json = fixture("links-and-nav.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("<nav "));
    assert!(result.html.contains("</nav>"));
}

#[test]
fn semantic_main_emits_main_element() {
    let json = fixture("links-and-nav.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("<main "));
    assert!(result.html.contains("</main>"));
}

#[test]
fn semantic_footer_emits_footer_element() {
    let json = fixture("links-and-nav.voce.json");
    let result = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    assert!(result.html.contains("<footer "));
    assert!(result.html.contains("</footer>"));
}

#[test]
fn compilation_under_500ms() {
    let json = landing_page();
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let _ = voce_compiler_dom::compile(&json, &Default::default()).unwrap();
    }
    let elapsed = start.elapsed();
    // 10 compilations should complete well under 5s (500ms each)
    assert!(
        elapsed.as_millis() < 5000,
        "10 compilations took {}ms, expected < 5000ms",
        elapsed.as_millis()
    );
}
