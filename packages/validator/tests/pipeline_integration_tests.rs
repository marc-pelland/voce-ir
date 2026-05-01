//! Full-pipeline integration tests (S69 part 2).
//!
//! These exercise validate → compile → assert across the whole stack, not
//! one stage at a time. Catches the class of bugs that pass unit tests but
//! break when the pieces meet — e.g. a validator pass that silently accepts
//! an IR the compiler can't render, or compiler emit that doesn't match the
//! IR's semantic intent.

use voce_compiler_dom::{CompileOptions, compile};
use voce_validator::validate;

// ── Helpers ─────────────────────────────────────────────────────────────────

fn validate_then_compile(json: &str) -> (voce_validator::ValidationResult, String) {
    let result = validate(json).expect("validation parse failed");
    let compiled = compile(json, &CompileOptions::default()).expect("compile failed");
    (result, compiled.html)
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

// ── Test 1: simple landing page ─────────────────────────────────────────────

#[test]
fn pipeline_simple_landing_e2e() {
    let json = r#"{
      "schema_version_major": 1,
      "schema_version_minor": 0,
      "root": {
        "node_id": "root",
        "document_language": "en",
        "metadata": {
          "title": "Test Landing",
          "description": "A short test description that satisfies SEO length minimums for this integration check."
        },
        "semantic_nodes": [
          { "node_id": "sem-main", "role": "main", "label": "Main content" }
        ],
        "children": [
          {
            "value_type": "Container",
            "value": {
              "node_id": "main",
              "direction": "Column",
              "semantic_node_id": "sem-main",
              "padding": {
                "top": { "value": 32, "unit": "Px" },
                "bottom": { "value": 32, "unit": "Px" },
                "left": { "value": 24, "unit": "Px" },
                "right": { "value": 24, "unit": "Px" }
              },
              "children": [
                {
                  "value_type": "TextNode",
                  "value": {
                    "node_id": "title",
                    "content": "Hello, world",
                    "heading_level": 1,
                    "font_size": { "value": 48, "unit": "Px" },
                    "font_weight": "Bold"
                  }
                },
                {
                  "value_type": "TextNode",
                  "value": {
                    "node_id": "subtitle",
                    "content": "An integration test page rendering through the full pipeline.",
                    "font_size": { "value": 18, "unit": "Px" }
                  }
                }
              ]
            }
          }
        ]
      }
    }"#;

    let (result, html) = validate_then_compile(json);

    // Validation: clean, no errors
    assert!(
        !result.has_errors(),
        "validation errors: {:?}",
        result.diagnostics
    );

    // The compiled HTML respects the IR's intent
    assert!(html.contains("<html lang=\"en\""), "lang attr missing");
    assert!(
        html.contains("<title>Test Landing</title>"),
        "title missing"
    );
    assert!(
        html.contains("<meta name=\"description\""),
        "meta description missing"
    );
    assert!(html.contains("role=\"main\""), "role=main missing");

    // Heading hierarchy: exactly one h1
    assert_eq!(count_occurrences(&html, "<h1"), 1, "h1 count");

    // S64 baseline CSS is present
    assert!(html.contains("--voce-fg"), "fallback theme palette missing");
    assert!(
        html.contains("h1,h2,h3,h4,h5,h6{line-height:1.2"),
        "typography rhythm CSS missing"
    );
}

// ── Test 2: form submission flow ────────────────────────────────────────────

#[test]
fn pipeline_form_with_action_e2e() {
    let json = r#"{
      "schema_version_major": 1,
      "schema_version_minor": 0,
      "root": {
        "node_id": "root",
        "document_language": "en",
        "metadata": { "title": "Form Test", "description": "Form rendering test, exercises the FormNode validation pass and DOM emission." },
        "semantic_nodes": [
          { "node_id": "sem-form", "role": "form", "label": "Contact form" }
        ],
        "children": [
          {
            "value_type": "FormNode",
            "value": {
              "node_id": "form",
              "semantic_node_id": "sem-form",
              "validation_mode": "OnBlurThenChange",
              "fields": [
                {
                  "name": "name",
                  "field_type": "Text",
                  "label": "Your name",
                  "validations": [
                    { "rule_type": "Required", "message": "Name is required" }
                  ],
                  "autocomplete": "Name"
                },
                {
                  "name": "email",
                  "field_type": "Email",
                  "label": "Email address",
                  "validations": [
                    { "rule_type": "Required", "message": "Email is required" },
                    { "rule_type": "Email", "message": "Must be a valid email" }
                  ],
                  "autocomplete": "Email"
                }
              ],
              "submission": {
                "action_node_id": "submit-action",
                "encoding": "Json",
                "progressive": true
              }
            }
          },
          {
            "value_type": "ActionNode",
            "value": {
              "node_id": "submit-action",
              "source": { "endpoint": "/api/submit", "provider": "Rest" },
              "method": "POST",
              "csrf_protected": true
            }
          }
        ]
      }
    }"#;

    let (result, html) = validate_then_compile(json);

    assert!(
        !result.has_errors(),
        "validation errors: {:?}",
        result.diagnostics
    );

    // Form structure
    assert!(html.contains("<form"), "form tag missing");
    assert_eq!(
        count_occurrences(&html, "<input"),
        2,
        "expected 2 inputs (name + email)"
    );
    assert!(html.contains("type=\"email\""), "email field type missing");
    assert!(html.contains("required"), "required attr missing");
    assert!(
        html.contains("autocomplete=\"name\""),
        "autocomplete=name missing"
    );
    assert!(
        html.contains("autocomplete=\"email\""),
        "autocomplete=email missing"
    );

    // Labels are emitted (FRM009 / a11y)
    assert!(html.contains("Your name"), "name label missing");
    assert!(html.contains("Email address"), "email label missing");

    // Submit button rendered
    assert!(html.contains("type=\"submit\""), "submit button missing");

    // S61 form CSS defaults applied
    assert!(
        html.contains("form{display:flex;flex-direction:column"),
        "form layout CSS missing"
    );
}

// ── Test 3: state machine compilation ───────────────────────────────────────

#[test]
fn pipeline_state_machine_e2e() {
    let json = r#"{
      "schema_version_major": 1,
      "schema_version_minor": 0,
      "root": {
        "node_id": "root",
        "document_language": "en",
        "metadata": { "title": "State Machine Test", "description": "Exercises StateMachine validation and DOM emission with a 3-state toggle." },
        "semantic_nodes": [
          { "node_id": "sem-main", "role": "main", "label": "Toggle demo" }
        ],
        "children": [
          {
            "value_type": "StateMachine",
            "value": {
              "node_id": "toggle",
              "states": [
                { "name": "idle", "initial": true },
                { "name": "loading" },
                { "name": "done" }
              ],
              "transitions": [
                { "from": "idle", "event": "click", "to": "loading" },
                { "from": "loading", "event": "complete", "to": "done" },
                { "from": "done", "event": "reset", "to": "idle" }
              ]
            }
          },
          {
            "value_type": "Container",
            "value": {
              "node_id": "main",
              "direction": "Column",
              "semantic_node_id": "sem-main",
              "children": [
                {
                  "value_type": "TextNode",
                  "value": {
                    "node_id": "label",
                    "content": "Toggle demo",
                    "heading_level": 1
                  }
                }
              ]
            }
          }
        ]
      }
    }"#;

    let (result, html) = validate_then_compile(json);

    // The state machine validates cleanly (initial state declared, transitions
    // reference real states, no event collisions).
    let state_machine_diags: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.pass == "state-machine")
        .collect();
    assert!(
        state_machine_diags.is_empty(),
        "state machine diagnostics: {:?}",
        state_machine_diags
    );

    // The compiled output renders the page (state-machine logic is compiled
    // but doesn't have a visible target without GestureHandler; the IR is
    // still valid and renders its sibling Container).
    assert!(html.contains("<html"), "compiled output missing");
    assert!(html.contains("Toggle demo"), "label text missing");

    // Per-pass output should report state-machine ran cleanly
    let sm_pass = result
        .passes
        .iter()
        .find(|p| p.name == "state-machine")
        .expect("state-machine pass result missing");
    assert_eq!(sm_pass.error_count, 0);
    assert_eq!(sm_pass.warning_count, 0);
}
