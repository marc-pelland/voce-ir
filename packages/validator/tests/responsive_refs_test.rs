//! ResponsiveRule override targets must reference existing nodes (audit F4).

use voce_validator::validate;

#[test]
fn responsive_override_targeting_missing_node_is_flagged() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "Container", "value": { "node_id": "grid", "children": [] } },
            { "value_type": "ResponsiveRule", "value": {
                "node_id": "rr",
                "breakpoints": [ { "name": "m", "min_width": { "value": 0, "unit": "Px" } } ],
                "responsive_overrides": [ { "breakpoint_name": "m", "overrides": [
                    { "target_node_id": "does-not-exist", "property": "font_size", "value": "1rem" }
                ] } ]
            } }
        ] }
    }"#;
    let result = validate(json).expect("validate returns a result");
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.code == "REF005" && d.message.contains("does-not-exist")),
        "expected REF005 for the missing responsive override target; got: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| &d.code)
            .collect::<Vec<_>>()
    );
}

#[test]
fn responsive_override_targeting_existing_node_is_ok() {
    let json = r#"{
        "root": { "node_id": "root", "children": [
            { "value_type": "Container", "value": { "node_id": "grid", "children": [] } },
            { "value_type": "ResponsiveRule", "value": {
                "node_id": "rr",
                "breakpoints": [ { "name": "m", "min_width": { "value": 0, "unit": "Px" } } ],
                "responsive_overrides": [ { "breakpoint_name": "m", "overrides": [
                    { "target_node_id": "grid", "property": "font_size", "value": "1rem" }
                ] } ]
            } }
        ] }
    }"#;
    let result = validate(json).expect("validate returns a result");
    assert!(
        !result.diagnostics.iter().any(|d| d.code == "REF005"),
        "a valid responsive target must not raise REF005; got: {:?}",
        result.diagnostics
    );
}

fn has_rsp001(json: &str) -> bool {
    validate(json)
        .expect("validate")
        .diagnostics
        .iter()
        .any(|d| d.code == "RSP001")
}

#[test]
fn large_fixed_width_without_max_or_override_warns() {
    let json = r#"{ "root": { "node_id": "root", "children": [
        { "value_type": "Container", "value": { "node_id": "wide",
            "width": { "value": 800, "unit": "Px" }, "children": [] } }
    ] } }"#;
    assert!(has_rsp001(json), "expected RSP001 for an 800px fixed width");
}

#[test]
fn fixed_width_is_exempt_with_max_width_override_or_small_size() {
    // Has max_width -> no warning.
    let with_max = r#"{ "root": { "node_id": "root", "children": [
        { "value_type": "Container", "value": { "node_id": "w",
            "width": { "value": 800, "unit": "Px" }, "max_width": { "value": 100, "unit": "Percent" }, "children": [] } }
    ] } }"#;
    assert!(!has_rsp001(with_max), "max_width should exempt");

    // A ResponsiveRule targets it -> author handled it.
    let with_rule = r#"{ "root": { "node_id": "root", "children": [
        { "value_type": "Container", "value": { "node_id": "w", "width": { "value": 800, "unit": "Px" }, "children": [] } },
        { "value_type": "ResponsiveRule", "value": { "node_id": "rr",
            "breakpoints": [ { "name": "m", "min_width": { "value": 0, "unit": "Px" } } ],
            "responsive_overrides": [ { "breakpoint_name": "m", "overrides": [
                { "target_node_id": "w", "property": "width", "value": "100%" } ] } ] } }
    ] } }"#;
    assert!(!has_rsp001(with_rule), "responsive override should exempt");

    // Small fixed width -> no warning.
    let small = r#"{ "root": { "node_id": "root", "children": [
        { "value_type": "Container", "value": { "node_id": "w", "width": { "value": 400, "unit": "Px" }, "children": [] } }
    ] } }"#;
    assert!(!has_rsp001(small), "400px is under the threshold");
}
