//! S79 A4 — JSON Schemas for the agent contract envelopes.
//!
//! Two guarantees per envelope, both CI-enforced:
//!
//! 1. **Drift gate** — the committed schema in
//!    `docs/schema/contract/v1/<name>.schema.json` is regenerated from
//!    the live struct at test time and asserted equal. Any field
//!    change is therefore a *deliberate* edit (commit the regenerated
//!    schema after running with `UPDATE_CONTRACT_SCHEMAS=1`), and a
//!    breaking change forces a conscious `contract_version` major bump
//!    by surfacing the diff in code review.
//! 2. **Live conformance** — real envelope output (built in-process
//!    from the same lib code the binary calls) is validated against
//!    the committed schema, so the published contract isn't aspirational.
//!
//! Slice 1 covers the two S79-owned envelopes: `skills` and `graph`.
//! Validator output (`voce validate --format json`) and `voce compile
//! --perf-report` are deferred to a follow-up; documented in the
//! contract README.

use std::fs;
use std::path::PathBuf;

use voce_validator::index::NodeIndex;
use voce_validator::ir::VoceIr;
use voce_validator::{doctor, graph, skills};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn contract_path(name: &str) -> PathBuf {
    workspace_root()
        .join("docs/schema/contract/v1")
        .join(format!("{name}.schema.json"))
}

fn assert_schema_in_sync(envelope: &str, generated: serde_json::Value) {
    let path = contract_path(envelope);
    let pretty = serde_json::to_string_pretty(&generated).unwrap();

    // `UPDATE_CONTRACT_SCHEMAS=1` rewrites the committed schema. Used
    // intentionally after a contract field change; review the diff and
    // bump contract_version if breaking.
    if std::env::var("UPDATE_CONTRACT_SCHEMAS").ok().as_deref() == Some("1") {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, format!("{pretty}\n")).unwrap();
        eprintln!("(UPDATE_CONTRACT_SCHEMAS) wrote {}", path.display());
        return;
    }

    let committed_text = fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "Missing contract schema {}. Run with UPDATE_CONTRACT_SCHEMAS=1 to seed it.",
            path.display()
        )
    });
    let committed: serde_json::Value = serde_json::from_str(&committed_text)
        .unwrap_or_else(|e| panic!("committed schema {} is not valid JSON: {e}", path.display()));

    assert_eq!(
        generated, committed,
        "{envelope} schema drifted from the live struct.\n\
         Run `UPDATE_CONTRACT_SCHEMAS=1 cargo test -p voce-validator --test contract_schemas` \
         to regenerate, review the diff, and bump contract_version (major if breaking).",
    );
}

fn validate_against(schema: &serde_json::Value, instance: &serde_json::Value, envelope: &str) {
    let validator = jsonschema::validator_for(schema)
        .unwrap_or_else(|e| panic!("could not compile {envelope} schema: {e}"));
    let errors: Vec<String> = validator
        .iter_errors(instance)
        .map(|e| format!("  - {} (at {})", e, e.instance_path))
        .collect();
    assert!(
        errors.is_empty(),
        "live {envelope} envelope failed schema validation:\n{}",
        errors.join("\n")
    );
}

fn fixture(name: &str) -> (VoceIr, NodeIndex) {
    let path = workspace_root().join("tests/fixtures").join(name);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {name}: {e}"));
    let ir: VoceIr = serde_json::from_str(&text).expect("parse fixture");
    let idx = NodeIndex::build(&ir);
    (ir, idx)
}

// ─── Drift tests ────────────────────────────────────────────────

#[test]
fn skills_schema_in_sync() {
    let schema = serde_json::to_value(schemars::schema_for!(skills::Manifest)).unwrap();
    assert_schema_in_sync("skills", schema);
}

#[test]
fn graph_schema_in_sync() {
    let schema = serde_json::to_value(schemars::schema_for!(graph::GraphExport)).unwrap();
    assert_schema_in_sync("graph", schema);
}

#[test]
fn doctor_schema_in_sync() {
    let schema = serde_json::to_value(schemars::schema_for!(doctor::DoctorReport)).unwrap();
    assert_schema_in_sync("doctor", schema);
}

#[test]
fn perf_report_schema_in_sync() {
    // PerfReport lives in voce-compiler-dom (where it's produced); the
    // contract schema lives here because docs/schema/contract/ is the
    // single home for the agent contract. validator already depends on
    // compiler-dom.
    let schema =
        serde_json::to_value(schemars::schema_for!(voce_compiler_dom::perf::PerfReport)).unwrap();
    assert_schema_in_sync("perf-report", schema);
}

// ─── Live conformance ────────────────────────────────────────────

#[test]
fn live_skills_output_matches_schema() {
    let path = contract_path("skills");
    let schema: serde_json::Value = match fs::read_to_string(&path) {
        Ok(t) => serde_json::from_str(&t).expect("schema parses"),
        Err(_) => return, // skills_schema_in_sync owns the missing-file panic.
    };
    let manifest = skills::build(
        env!("CARGO_PKG_VERSION"),
        vec![skills::CliCommand {
            name: "validate".into(),
            about: "Validate an IR file".into(),
        }],
    );
    let instance = serde_json::to_value(&manifest).unwrap();
    validate_against(&schema, &instance, "skills");
}

#[test]
fn live_graph_output_matches_schema() {
    let path = contract_path("graph");
    let schema: serde_json::Value = match fs::read_to_string(&path) {
        Ok(t) => serde_json::from_str(&t).expect("schema parses"),
        Err(_) => return, // graph_schema_in_sync owns the missing-file panic.
    };
    // Use a fixture exercising both reference edges and (attempted)
    // state machines — i.e. a realistic graph, not an empty one.
    let (ir, idx) = fixture("links-and-nav.voce.json");
    let g = graph::build(&ir, &idx);
    let instance = serde_json::to_value(&g).unwrap();
    validate_against(&schema, &instance, "graph");
}

#[test]
fn live_doctor_output_matches_schema() {
    let path = contract_path("doctor");
    let schema: serde_json::Value = match fs::read_to_string(&path) {
        Ok(t) => serde_json::from_str(&t).expect("schema parses"),
        Err(_) => return, // doctor_schema_in_sync owns the missing-file panic.
    };
    // Run doctor against the workspace root — a known-good .voce/ is
    // present and toolchain checks exercise real machine state.
    let report = doctor::run(&workspace_root(), false);
    let instance = serde_json::to_value(&report).unwrap();
    validate_against(&schema, &instance, "doctor");
}

#[test]
fn live_perf_report_output_matches_schema() {
    let path = contract_path("perf-report");
    let schema: serde_json::Value = match fs::read_to_string(&path) {
        Ok(t) => serde_json::from_str(&t).expect("schema parses"),
        Err(_) => return, // perf_report_schema_in_sync owns the missing-file panic.
    };
    // Compile a fixture with perf collection enabled and validate the
    // real PerfReport, not a hand-built struct.
    let json = std::fs::read_to_string(workspace_root().join("tests/fixtures/text-heading.voce.json"))
        .expect("read fixture");
    let opts = voce_compiler_dom::CompileOptions {
        collect_perf_report: true,
        ..Default::default()
    };
    let result = voce_compiler_dom::compile(&json, &opts).expect("compile");
    let report = result.perf_report.expect("perf_report present when collect=true");
    let instance = serde_json::to_value(&report).unwrap();
    validate_against(&schema, &instance, "perf-report");
}
