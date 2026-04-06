//! StateMachine → WASM compilation.
//!
//! Each state machine compiles to a WAT function that:
//! - Stores current state in linear memory
//! - Takes an event ID (i32) as input
//! - Looks up the transition in a table
//! - Returns the new state ID (i32)

use serde_json::Value;

/// Compile a StateMachine to WAT function + export.
///
/// Returns (function_wat, export_wat).
pub fn compile_state_machine(value: &Value, func_name: &str) -> (String, String) {
    let states: Vec<String> = value
        .get("states")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s.get("name").and_then(|n| n.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let transitions: Vec<(String, String, String)> = value
        .get("transitions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    let event = t.get("event")?.as_str()?.to_string();
                    let from = t.get("from")?.as_str()?.to_string();
                    let to = t.get("to")?.as_str()?.to_string();
                    Some((event, from, to))
                })
                .collect()
        })
        .unwrap_or_default();

    // State name → integer mapping
    let state_ids: std::collections::HashMap<&str, i32> = states
        .iter()
        .enumerate()
        .map(|(i, s)| (s.as_str(), i as i32))
        .collect();

    // Event name → integer mapping
    let event_names: Vec<String> = transitions
        .iter()
        .map(|(e, _, _)| e.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let event_ids: std::collections::HashMap<&str, i32> = event_names
        .iter()
        .enumerate()
        .map(|(i, e)| (e.as_str(), i as i32))
        .collect();

    // Memory offset for this state machine's current state (4 bytes per SM)
    let mem_offset = 0; // TODO: allocate based on SM count

    // Build the transition table as a series of br_table branches
    let mut func = format!(
        "  ;; StateMachine: {func_name} ({} states, {} transitions)\n",
        states.len(),
        transitions.len()
    );
    func.push_str(&format!(
        "  (func ${func_name}_send (param $event i32) (result i32)\n"
    ));
    func.push_str(&format!(
        "    (local $state i32)\n    (local.set $state (i32.load (i32.const {mem_offset})))\n"
    ));

    // Generate if/else chain for state + event matching
    for (event, from, to) in &transitions {
        let from_id = state_ids.get(from.as_str()).copied().unwrap_or(-1);
        let to_id = state_ids.get(to.as_str()).copied().unwrap_or(-1);
        let event_id = event_ids.get(event.as_str()).copied().unwrap_or(-1);

        func.push_str(&format!("    ;; {from} + {event} → {to}\n"));
        func.push_str(&format!(
            "    (if (i32.and (i32.eq (local.get $state) (i32.const {from_id})) (i32.eq (local.get $event) (i32.const {event_id})))\n"
        ));
        func.push_str(&format!(
            "      (then (i32.store (i32.const {mem_offset}) (i32.const {to_id})) (return (i32.const {to_id})))\n"
        ));
        func.push_str("    )\n");
    }

    // No matching transition — return current state
    func.push_str("    (local.get $state)\n");
    func.push_str("  )\n");

    // Init function — sets initial state
    let initial_id = states
        .iter()
        .position(|s| {
            value
                .get("states")
                .and_then(|v| v.as_array())
                .and_then(|arr| {
                    arr.iter().find(|st| {
                        st.get("name").and_then(|n| n.as_str()) == Some(s)
                            && st.get("initial").and_then(|i| i.as_bool()).unwrap_or(false)
                    })
                })
                .is_some()
        })
        .unwrap_or(0) as i32;

    func.push_str(&format!(
        "  (func ${func_name}_init\n    (i32.store (i32.const {mem_offset}) (i32.const {initial_id}))\n  )\n"
    ));

    // Get current state
    func.push_str(&format!(
        "  (func ${func_name}_get (result i32)\n    (i32.load (i32.const {mem_offset}))\n  )\n"
    ));

    let export = format!(
        "  (export \"{func_name}_send\" (func ${func_name}_send))\n  (export \"{func_name}_init\" (func ${func_name}_init))\n  (export \"{func_name}_get\" (func ${func_name}_get))"
    );

    (func, export)
}
