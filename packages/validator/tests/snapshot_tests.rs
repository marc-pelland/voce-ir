//! Insta snapshot tests — canonical IR fragments compiled to DOM output.
//!
//! Each test compiles a minimal IR fragment and snapshots the output.
//! Run `cargo insta review` to accept new snapshots.

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn fixture(name: &str) -> String {
    let path = workspace_root().join(format!("tests/fixtures/{name}"));
    fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read fixture: {}", path.display()))
}

fn compile_to_html(json: &str) -> String {
    voce_compiler_dom::compile(json, &voce_compiler_dom::CompileOptions::default())
        .expect("Compilation failed")
        .html
}

#[test]
fn snapshot_text_heading() {
    let html = compile_to_html(&fixture("text-heading.voce.json"));
    insta::assert_snapshot!("text_heading_dom", html);
}

#[test]
fn snapshot_container_row() {
    let html = compile_to_html(&fixture("container-row.voce.json"));
    insta::assert_snapshot!("container_row_dom", html);
}

#[test]
fn snapshot_surface_card() {
    let html = compile_to_html(&fixture("surface-card.voce.json"));
    insta::assert_snapshot!("surface_card_dom", html);
}

#[test]
fn snapshot_media_image() {
    let html = compile_to_html(&fixture("media-image.voce.json"));
    insta::assert_snapshot!("media_image_dom", html);
}

#[test]
fn snapshot_form_contact() {
    let html = compile_to_html(&fixture("form-contact.voce.json"));
    insta::assert_snapshot!("form_contact_dom", html);
}

#[test]
fn snapshot_container_grid() {
    let html = compile_to_html(&fixture("container-grid.voce.json"));
    insta::assert_snapshot!("grid_dom", html);
}

#[test]
fn snapshot_decorative_surface() {
    let html = compile_to_html(&fixture("decorative-surface.voce.json"));
    insta::assert_snapshot!("decorative_surface_dom", html);
}

#[test]
fn snapshot_nested_layout() {
    let html = compile_to_html(&fixture("nested-layout.voce.json"));
    insta::assert_snapshot!("nested_layout_dom", html);
}

#[test]
fn snapshot_gesture_tap() {
    let html = compile_to_html(&fixture("gesture-tap.voce.json"));
    insta::assert_snapshot!("gesture_tap_dom", html);
}

#[test]
fn snapshot_state_machine() {
    let html = compile_to_html(&fixture("state-machine.voce.json"));
    insta::assert_snapshot!("state_machine_dom", html);
}

// Additional tests for adapter output formats

#[test]
fn snapshot_validation_errors() {
    let json = r#"{ "schema_version_major": 1, "schema_version_minor": 0 }"#;
    let result = voce_validator::validate(json).unwrap();
    let errors: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| format!("[{}] {}: {}", d.severity, d.code, d.message))
        .collect();
    insta::assert_snapshot!("validation_missing_root", errors.join("\n"));
}

#[test]
fn snapshot_email_output() {
    let html = voce_compiler_email::compile_email(&fixture("text-heading.voce.json"))
        .expect("Email compilation failed")
        .html;
    insta::assert_snapshot!("text_heading_email", html);
}

#[test]
fn snapshot_swiftui_output() {
    let swift = voce_compiler_ios::compile_swiftui(&fixture("container-row.voce.json"))
        .expect("SwiftUI compilation failed")
        .swift;
    insta::assert_snapshot!("container_row_swiftui", swift);
}

#[test]
fn snapshot_compose_output() {
    let kotlin = voce_compiler_android::compile_compose(&fixture("text-heading.voce.json"))
        .expect("Compose compilation failed")
        .kotlin;
    insta::assert_snapshot!("text_heading_compose", kotlin);
}
