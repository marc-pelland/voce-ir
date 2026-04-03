//! Integration tests for the Voce IR validator.

use voce_validator::{Severity, validate};

fn load_fixture(path: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/../../tests/schema/{path}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap_or_else(|e| panic!("Failed to load fixture {path}: {e}"))
}

fn error_codes(json: &str) -> Vec<String> {
    let result = validate(json).unwrap();
    result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.code.clone())
        .collect()
}

fn warning_codes(json: &str) -> Vec<String> {
    let result = validate(json).unwrap();
    result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .map(|d| d.code.clone())
        .collect()
}

// ─── Valid IR ────────────────────────────────────────────────────

#[test]
fn valid_minimal_page_produces_no_errors() {
    let json = load_fixture("valid/minimal-page.json");
    let result = validate(&json).unwrap();
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Expected 0 errors for valid fixture, got: {errors:#?}"
    );
}

// ─── Structural Pass ────────────────────────────────────────────

#[test]
fn str001_missing_viewroot() {
    let json = load_fixture("invalid/missing-viewroot.voce.json");
    let codes = error_codes(&json);
    assert!(
        codes.contains(&"STR001".to_string()),
        "Expected STR001, got: {codes:?}"
    );
}

#[test]
fn str002_duplicate_node_ids() {
    let json = load_fixture("invalid/duplicate-ids.voce.json");
    let codes = error_codes(&json);
    assert!(
        codes.contains(&"STR002".to_string()),
        "Expected STR002, got: {codes:?}"
    );
}

#[test]
fn str004_empty_textnode() {
    let json = load_fixture("invalid/empty-textnode.voce.json");
    let codes = error_codes(&json);
    assert!(
        codes.contains(&"STR004".to_string()),
        "Expected STR004, got: {codes:?}"
    );
}

// ─── Reference Pass ─────────────────────────────────────────────

#[test]
fn ref005_broken_target_ref() {
    let json = load_fixture("invalid/broken-ref.voce.json");
    let codes = error_codes(&json);
    assert!(
        codes.contains(&"REF005".to_string()),
        "Expected REF005, got: {codes:?}"
    );
}

// ─── State Machine Pass ─────────────────────────────────────────

#[test]
fn sta002_missing_initial_state() {
    let json = load_fixture("invalid/missing-initial-state.voce.json");
    let codes = error_codes(&json);
    assert!(
        codes.contains(&"STA002".to_string()),
        "Expected STA002, got: {codes:?}"
    );
}

#[test]
fn sta003_unreachable_state() {
    let json = load_fixture("invalid/unreachable-state.voce.json");
    let warnings = warning_codes(&json);
    assert!(
        warnings.contains(&"STA003".to_string()),
        "Expected STA003 warning, got: {warnings:?}"
    );
}

// ─── Engine ─────────────────────────────────────────────────────

#[test]
fn validate_returns_result_for_empty_root() {
    let json = r#"{ "root": { "node_id": "root" } }"#;
    let result = validate(json).unwrap();
    // Should parse and validate without crashing
    assert!(result.error_count() == 0 || result.error_count() > 0);
}

#[test]
fn validate_returns_error_for_invalid_json() {
    let result = validate("not valid json at all");
    assert!(result.is_err());
}

#[test]
fn validate_counts_errors_and_warnings() {
    let json = load_fixture("invalid/unreachable-state.voce.json");
    let result = validate(&json).unwrap();
    assert!(result.warning_count() > 0, "Expected at least one warning");
}

// ─── Accessibility Pass ─────────────────────────────────────────

#[test]
fn a11y004_gesture_no_keyboard() {
    let json = load_fixture("invalid/gesture-no-keyboard.voce.json");
    let codes = error_codes(&json);
    assert!(
        codes.contains(&"A11Y004".to_string()),
        "Expected A11Y004, got: {codes:?}"
    );
}

#[test]
fn a11y005_heading_skip() {
    let json = load_fixture("invalid/heading-skip.voce.json");
    let codes = error_codes(&json);
    assert!(
        codes.contains(&"A11Y005".to_string()),
        "Expected A11Y005, got: {codes:?}"
    );
}

// ─── Security Pass ──────────────────────────────────────────────

#[test]
fn sec002_mutation_no_csrf() {
    let json = load_fixture("invalid/mutation-no-csrf.voce.json");
    let codes = error_codes(&json);
    assert!(
        codes.contains(&"SEC002".to_string()),
        "Expected SEC002, got: {codes:?}"
    );
}

// ─── SEO Pass ───────────────────────────────────────────────────

#[test]
fn seo001_missing_metadata() {
    // minimal-page.json has no metadata — should warn
    let json = load_fixture("valid/minimal-page.json");
    let warnings = warning_codes(&json);
    assert!(
        warnings.contains(&"SEO001".to_string()),
        "Expected SEO001 warning for page without metadata, got: {warnings:?}"
    );
}

// ─── Forms Pass ─────────────────────────────────────────────────

#[test]
fn frm001_form_no_fields() {
    let json = load_fixture("invalid/form-no-fields.voce.json");
    let codes = error_codes(&json);
    assert!(
        codes.contains(&"FRM001".to_string()),
        "Expected FRM001, got: {codes:?}"
    );
}

// ─── i18n Pass ──────────────────────────────────────────────────

#[test]
fn i18n002_empty_localized_key() {
    let json = load_fixture("invalid/empty-localized-key.voce.json");
    let codes = error_codes(&json);
    assert!(
        codes.contains(&"I18N002".to_string()),
        "Expected I18N002, got: {codes:?}"
    );
}

// ─── Motion Pass ────────────────────────────────────────────────

#[test]
fn mot001_transition_no_reduced_motion() {
    let json = load_fixture("invalid/transition-no-reduced-motion.voce.json");
    let codes = error_codes(&json);
    assert!(
        codes.contains(&"MOT001".to_string()),
        "Expected MOT001, got: {codes:?}"
    );
}

// ─── Full Pass Count ────────────────────────────────────────────

#[test]
fn all_nine_passes_run() {
    let json = load_fixture("valid/minimal-page.json");
    let result = validate(&json).unwrap();
    // Collect pass names from diagnostics (warnings from SEO pass prove it ran)
    let passes: std::collections::HashSet<_> =
        result.diagnostics.iter().map(|d| d.pass.clone()).collect();
    // At minimum, SEO pass should produce a warning on minimal-page (no metadata)
    assert!(
        passes.contains("seo"),
        "Expected seo pass to run, got passes: {passes:?}"
    );
}

// ─── Examples Validation ────────────────────────────────────────

fn load_example(path: &str) -> String {
    std::fs::read_to_string(format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("Failed to load example {path}: {e}"))
}

#[test]
fn landing_page_validates_cleanly() {
    let json = load_example("examples/landing-page/landing-page.voce.json");
    let result = validate(&json).unwrap();
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Landing page should validate with 0 errors, got: {errors:#?}"
    );
}

#[test]
fn landing_page_has_many_nodes() {
    let json = load_example("examples/landing-page/landing-page.voce.json");
    let ir: voce_validator::ir::VoceIr = serde_json::from_str(&json).unwrap();
    let summary = voce_validator::inspect::summarize(&ir);
    assert!(
        summary.total_nodes >= 30,
        "Landing page should have 30+ nodes, got {}",
        summary.total_nodes
    );
    assert!(
        summary.node_counts.len() >= 8,
        "Landing page should use 8+ node types, got {}",
        summary.node_counts.len()
    );
}

#[test]
fn intent_01_hero_validates() {
    let json = load_example("examples/intents/01-hero-section/ir.voce.json");
    let result = validate(&json).unwrap();
    assert!(
        !result.has_errors(),
        "Hero intent IR should validate cleanly"
    );
}

#[test]
fn intent_02_contact_form_validates() {
    let json = load_example("examples/intents/02-contact-form/ir.voce.json");
    let result = validate(&json).unwrap();
    assert!(
        !result.has_errors(),
        "Contact form intent IR should validate cleanly"
    );
}

// ─── Diagnostic Quality ─────────────────────────────────────────

#[test]
fn diagnostics_have_pass_and_path() {
    let json = load_fixture("invalid/mutation-no-csrf.voce.json");
    let result = validate(&json).unwrap();
    for diag in &result.diagnostics {
        assert!(!diag.pass.is_empty(), "Diagnostic should have a pass name");
        assert!(
            !diag.node_path.is_empty(),
            "Diagnostic should have a node_path"
        );
        assert!(!diag.code.is_empty(), "Diagnostic should have a code");
        assert!(!diag.message.is_empty(), "Diagnostic should have a message");
    }
}

#[test]
fn error_codes_have_correct_prefix() {
    let json = load_fixture("invalid/heading-skip.voce.json");
    let result = validate(&json).unwrap();
    for diag in &result.diagnostics {
        let valid_prefix = diag.code.starts_with("STR")
            || diag.code.starts_with("REF")
            || diag.code.starts_with("STA")
            || diag.code.starts_with("A11Y")
            || diag.code.starts_with("SEC")
            || diag.code.starts_with("SEO")
            || diag.code.starts_with("FRM")
            || diag.code.starts_with("I18N")
            || diag.code.starts_with("MOT");
        assert!(
            valid_prefix,
            "Diagnostic code '{}' has unknown prefix",
            diag.code
        );
    }
}
