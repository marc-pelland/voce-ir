//! S79 B4 — contract-as-only-interface guarantee.
//!
//! The differentiator-grade invariant: every fact an agent needs is
//! reachable through the contract envelopes (`skills`, `graph`,
//! `doctor`, `fix-plan`, and the diagnostic envelope from `voce
//! validate`). Because Voce has no human-readable source text, an
//! agent that hits a wall can never "just read the code" — this test
//! guarantees the contract is complete enough that it doesn't have to.
//!
//! Each test below is a *representative agent task*. It performs only
//! contract-envelope reads (calling lib functions that produce them)
//! and asserts the facts required to complete that task are present.
//! If a future change weakens the contract — a new diagnostic code
//! emitted at runtime but not declared in `CodeMeta`; a new node type
//! used in IR but missing from the FB union; a compile target dropped
//! from the registry — these tests fail loudly and force the contract
//! addition before any drift ships.

use std::collections::HashSet;

use voce_validator::index::NodeIndex;
use voce_validator::ir::VoceIr;
use voce_validator::{doctor, fix_loop, graph, skills, validate};

fn manifest() -> skills::Manifest {
    skills::build(env!("CARGO_PKG_VERSION"), vec![])
}

/// Every contract envelope carries `contract_version` — the load-bearing
/// guarantee for any consumer pinning against a contract revision.
#[test]
fn every_envelope_declares_contract_version() {
    let m = manifest();
    assert!(!m.contract_version.is_empty(), "skills missing contract_version");

    let (ir, idx) = parse_ir(r#"{ "root": { "node_id": "r" } }"#);
    let g = graph::build(&ir, &idx);
    assert!(!g.contract_version.is_empty(), "graph missing contract_version");

    let doc = doctor::run(&std::env::temp_dir(), false);
    assert!(!doc.contract_version.is_empty(), "doctor missing contract_version");

    let plan = fix_loop::run(r#"{ "root": { "node_id": "r" } }"#, &fix_loop::LoopOptions::default())
        .expect("fix loop")
        .plan;
    assert!(!plan.contract_version.is_empty(), "fix-plan missing contract_version");
}

/// Agent task: "Tell me what this diagnostic code means and what to do
/// about it." Pick a code the validator definitely emits (A11Y007 — text
/// contrast) and assert the manifest answers every question without
/// requiring source inspection.
#[test]
fn task_understand_a_diagnostic_code() {
    let m = manifest();
    let code = m
        .diagnostic_codes
        .iter()
        .find(|c| c.code == "A11Y007")
        .expect("A11Y007 must be in the manifest — it's a real validator code");

    // What does it mean?
    assert!(!code.summary.is_empty(), "A11Y007 has no summary");
    // What should I do?
    assert!(!code.hint.is_empty(), "A11Y007 has no hint");
    // Where can I learn more?
    assert!(
        code.docs_url.starts_with("http"),
        "A11Y007 docs_url must be a URL, got {}",
        code.docs_url
    );
    // Which pass owns this? Must reference a real declared pass.
    let pass_names: HashSet<&str> = m.validation_passes.iter().map(|p| p.name).collect();
    assert!(
        pass_names.contains(code.pass),
        "A11Y007 pass '{}' is not in the manifest's validation_passes",
        code.pass
    );
    // Is it auto-fixable? The bool is part of the contract regardless.
    let _ = code.fixable;
}

/// Agent task: "I want to add a Form to this page — is that a thing,
/// and what union tag do I encode it with?"
#[test]
fn task_discover_node_types_and_their_validation_codes() {
    let m = manifest();
    let names: HashSet<&str> = m.node_types.iter().map(|n| n.name).collect();
    for required in ["FormNode", "Surface", "TextNode", "MediaNode", "StateMachine"] {
        assert!(
            names.contains(required),
            "core node type {required} missing from manifest — agents authoring IR need this"
        );
    }
    // Union tags are stable identifiers — required for binary encoding.
    for nt in &m.node_types {
        assert!(nt.union_tag > 0, "{} has union_tag 0 (NONE)", nt.name);
    }
    // Form-relevant validation codes must be present so an agent knows
    // what rules its FormNode will be measured against.
    let codes: HashSet<&str> = m.diagnostic_codes.iter().map(|c| c.code).collect();
    let frm_count = codes.iter().filter(|c| c.starts_with("FRM")).count();
    assert!(frm_count > 0, "no FRM* codes documented — forms are unspec'd to agents");
    let a11y_count = codes.iter().filter(|c| c.starts_with("A11Y")).count();
    assert!(a11y_count > 0, "no A11Y* codes documented");
}

/// Agent task: "What can I compile this to, and which target should I
/// pick for an email newsletter?" — discoverable purely from skills.
#[test]
fn task_discover_compile_targets() {
    let m = manifest();
    assert!(!m.compile_targets.is_empty(), "no compile targets registered");
    let ids: HashSet<&str> = m.compile_targets.iter().map(|t| t.id).collect();
    for required in ["dom", "email", "ios-swiftui"] {
        assert!(ids.contains(required), "target {required} missing from manifest");
    }
    for t in m.compile_targets {
        assert!(!t.outputs.is_empty(), "target {} declares no outputs", t.id);
        assert!(!t.notes.is_empty(), "target {} has no notes — agents need context", t.id);
    }
}

/// Agent task: "Read the current IR's structure so I can refactor without
/// breaking references" — must be fully answerable from `graph`.
#[test]
fn task_reason_about_ir_structure() {
    let json = r#"{
        "root": {
            "node_id": "r",
            "semantic_nodes": [{ "node_id": "s", "role": "button", "label": "Go" }],
            "children": [
                { "value_type": "Container", "value": { "node_id": "wrap", "children": [
                    { "value_type": "Surface", "value": { "node_id": "btn", "semantic_node_id": "s" } }
                ] } },
                { "value_type": "StateMachine", "value": {
                    "node_id": "sm", "name": "toggle",
                    "states": [
                        { "name": "off", "initial": true },
                        { "name": "on" },
                        { "name": "orphan" }
                    ],
                    "transitions": [
                        { "event": "tap", "from": "off", "to": "on" }
                    ]
                } }
            ]
        }
    }"#;
    let (ir, idx) = parse_ir(json);
    let g = graph::build(&ir, &idx);

    // Composition reachable — agent can walk parent→child without reading IR.
    assert!(g.nodes.iter().any(|n| n.id == "btn"));
    assert!(g
        .composition_edges
        .iter()
        .any(|e| e.parent == "wrap" && e.child == "btn"));

    // Reference edges typed + carry resolved/dangling — agent can find
    // dangling refs without re-running the validator.
    let semref = g
        .reference_edges
        .iter()
        .find(|e| e.from == "btn" && e.to == "s")
        .expect("semantic edge btn→s missing");
    assert!(semref.to_resolved);

    // State-machine reachability surfaced — agent can detect unreachable
    // states without writing its own BFS.
    let sm = g.state_machines.first().expect("state machine present");
    assert!(sm.unreachable_states.iter().any(|n| n == "orphan"));
    assert!(sm.states.iter().any(|s| s.name == "on" && s.reachable));
}

/// Agent task: "Is the project itself healthy enough for me to act?" —
/// answered by `doctor`. Stable check IDs are the agent's vocabulary.
#[test]
fn task_inspect_project_health() {
    let report = doctor::run(&std::env::temp_dir(), false);
    assert!(!report.checks.is_empty());
    for check in &report.checks {
        assert!(!check.id.is_empty(), "every check must have a stable ID");
        // IDs are the contract — agents key remediation logic off them.
        // The pattern is `DOC-<DOMAIN>-NNN`.
        assert!(
            check.id.starts_with("DOC-"),
            "check ID {} doesn't match the DOC-<DOMAIN>-NNN contract",
            check.id
        );
        assert!(check.docs_url.starts_with("http"));
    }
}

/// Agent task: "Drive auto-repair to convergence on this IR without
/// human input." Fix-plan envelope must expose enough to drive the
/// apply/retry loop headlessly.
#[test]
fn task_drive_headless_repair() {
    // IR with at least one Safe-fixable diagnostic (Surface missing
    // node_id → STR002 → Safe fix).
    let json = r#"{
        "root": { "node_id": "r", "children": [
            { "value_type": "Surface", "value": { "decorative": true } }
        ] }
    }"#;
    let plan = fix_loop::run(json, &fix_loop::LoopOptions::default())
        .expect("fix loop")
        .plan;
    // Convergence signal is part of the contract — agent uses it to
    // decide whether to apply or escalate.
    assert!(plan.converges);
    assert!(!plan.plan.is_empty());
    for step in &plan.plan {
        assert!(step.step > 0);
        assert!(!step.code.is_empty());
        assert!(!step.node_path.is_empty());
        assert!(!step.rationale.is_empty());
        assert!(!step.patch.is_empty());
        for op in &step.patch {
            // JSON-Patch ops the agent can apply directly.
            assert!(["add", "remove", "replace"].contains(&op.op.as_str()));
            assert!(op.path.starts_with('/'));
        }
    }
}

/// Cross-envelope invariant: **every diagnostic code emitted at runtime
/// must be declared in `skills.diagnostic_codes`.** This is the contract
/// "no surprise codes" guarantee — an agent that pre-loaded the manifest
/// can never encounter an undocumented code in a validate result.
///
/// Runs the validator across a corpus of fixtures (the same parity set
/// S68 uses, plus the `tests/schema/invalid/` files which exercise
/// codes nothing else does). Asserts every emitted code is in the
/// manifest. A new code added to a pass that forgot its CodeMeta entry
/// will fail here, forcing the registration that makes it discoverable.
#[test]
fn no_runtime_diagnostic_lacks_a_manifest_entry() {
    let m = manifest();
    let declared: HashSet<&str> = m.diagnostic_codes.iter().map(|c| c.code).collect();
    let mut undeclared: HashSet<String> = HashSet::new();

    for entry in walk_voce_json_under(&workspace_root().join("tests")) {
        let text = match std::fs::read_to_string(&entry) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let result = match validate(&text) {
            Ok(r) => r,
            Err(_) => continue, // parse failure isn't a diagnostic code
        };
        for d in &result.diagnostics {
            if !declared.contains(d.code.as_str()) {
                undeclared.insert(d.code.clone());
            }
        }
    }

    assert!(
        undeclared.is_empty(),
        "diagnostic codes emitted at runtime but missing from skills.diagnostic_codes \
         (contract incomplete — add CodeMeta entries): {:?}",
        undeclared
    );
}

// ─── helpers ────────────────────────────────────────────────────

fn parse_ir(json: &str) -> (VoceIr, NodeIndex) {
    let ir: VoceIr = serde_json::from_str(json).expect("parse");
    let idx = NodeIndex::build(&ir);
    (ir, idx)
}

fn workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn walk_voce_json_under(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    fn recurse(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                recurse(&path, out);
            } else if path
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.ends_with(".voce.json"))
            {
                out.push(path);
            }
        }
    }
    recurse(dir, &mut out);
    out
}
