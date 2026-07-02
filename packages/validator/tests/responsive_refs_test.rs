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
