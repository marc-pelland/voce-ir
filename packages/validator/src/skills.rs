//! S79 A1 — `voce skills`: the unified, reflected, machine-consumable
//! capability manifest.
//!
//! Every field below is *reflected* from a single source of truth:
//! - validation passes / diagnostic codes ← `passes::all_passes()` (engine)
//! - node types ← `voce_schema::voce::ENUM_VALUES_CHILD_UNION` (generated FB bindings)
//! - compile targets ← `crate::targets::ALL` (canonical registry)
//! - cli commands ← clap `Command::get_subcommands()` (supplied by the binary)
//! - diagnostic docs URLs ← `formatter::docs_url`
//!
//! There is no hand-maintained list anywhere in this module. A hand list
//! rots; if it rots, the manifest lies; if the manifest lies, the whole
//! "agent contract" thesis fails. So the test for the manifest is also
//! the test for the contract's honesty.

use serde::Serialize;

use crate::formatter;
use crate::passes;
use crate::targets;

/// Semver of the agent contract envelope itself. Bumped major when any
/// existing field semantics change; minor on additive fields. Distinct
/// from `voce` version and `voce-schema` version.
pub const CONTRACT_VERSION: &str = "1.0.0";

/// One CLI subcommand description, supplied by the binary because clap's
/// `Command` lives in `main.rs` (not the lib).
#[derive(Debug, Clone, Serialize)]
pub struct CliCommand {
    pub name: String,
    pub about: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PassEntry {
    pub name: &'static str,
    /// Codes this pass owns, in the order the engine registers them.
    pub codes: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodeEntry {
    pub code: &'static str,
    pub pass: &'static str,
    pub summary: &'static str,
    pub hint: &'static str,
    /// `true` when the code carries a JSON-Patch fix (S67); the actual
    /// confidence level rides on each runtime diagnostic.
    pub fixable: bool,
    pub docs_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeTypeEntry {
    pub name: &'static str,
    /// FlatBuffers union discriminant — stable across schema minors.
    pub union_tag: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct Manifest {
    pub contract_version: &'static str,
    pub voce_version: &'static str,
    pub validation_passes: Vec<PassEntry>,
    pub diagnostic_codes: Vec<CodeEntry>,
    pub node_types: Vec<NodeTypeEntry>,
    pub compile_targets: &'static [targets::TargetInfo],
    pub cli_commands: Vec<CliCommand>,
}

/// Build the manifest. Caller supplies CLI command names because clap
/// `Command` introspection lives in the binary.
pub fn build(voce_version: &'static str, cli_commands: Vec<CliCommand>) -> Manifest {
    let mut validation_passes = Vec::new();
    let mut diagnostic_codes = Vec::new();
    for pass in passes::all_passes() {
        let pass_name = pass.name();
        let codes: Vec<&'static str> = pass.codes().iter().map(|m| m.code).collect();
        validation_passes.push(PassEntry {
            name: pass_name,
            codes: codes.clone(),
        });
        for meta in pass.codes() {
            diagnostic_codes.push(CodeEntry {
                code: meta.code,
                pass: pass_name,
                summary: meta.summary,
                hint: meta.hint,
                fixable: meta.fix_confidence.is_some(),
                docs_url: formatter::docs_url(meta.code),
            });
        }
    }

    // Node types reflected from the generated FlatBuffers union — the
    // single source of truth for "what node types Voce IR has." Skip
    // the synthetic NONE variant (tag 0). FlatBuffers marks the
    // `ENUM_VALUES_*` arrays deprecated in favor of associated
    // constants, but the only "non-deprecated" alternative is
    // hand-listing every variant — which would defeat the whole point
    // of a reflected manifest (a hand list rots and makes the contract
    // lie). The array is still generated and remains the right source.
    #[allow(deprecated)]
    let node_types: Vec<NodeTypeEntry> = voce_schema::voce::ENUM_VALUES_CHILD_UNION
        .iter()
        .filter_map(|v| v.variant_name().map(|n| (n, v.0)))
        .filter(|(name, _)| *name != "NONE")
        .map(|(name, tag)| NodeTypeEntry {
            name,
            union_tag: tag,
        })
        .collect();

    Manifest {
        contract_version: CONTRACT_VERSION,
        voce_version,
        validation_passes,
        diagnostic_codes,
        node_types,
        compile_targets: targets::ALL,
        cli_commands,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cli_stub() -> Vec<CliCommand> {
        vec![CliCommand {
            name: "validate".into(),
            about: "Validate an IR file".into(),
        }]
    }

    #[test]
    fn manifest_carries_contract_version_and_voce_version() {
        let m = build("0.1.0", cli_stub());
        assert_eq!(m.contract_version, CONTRACT_VERSION);
        assert_eq!(m.voce_version, "0.1.0");
    }

    #[test]
    fn validation_passes_and_codes_are_reflected_consistently() {
        let m = build("0.1.0", cli_stub());
        assert!(
            !m.validation_passes.is_empty(),
            "manifest must report at least one pass"
        );
        // Every code's `pass` must reference a real pass in the same manifest.
        let pass_names: std::collections::HashSet<&str> =
            m.validation_passes.iter().map(|p| p.name).collect();
        for c in &m.diagnostic_codes {
            assert!(
                pass_names.contains(c.pass),
                "code {} references unknown pass '{}'",
                c.code,
                c.pass
            );
            assert!(!c.summary.is_empty(), "code {} has empty summary", c.code);
            assert!(!c.hint.is_empty(), "code {} has empty hint", c.code);
        }
    }

    #[test]
    fn node_types_match_schema_count_and_include_core_types() {
        let m = build("0.1.0", cli_stub());
        // FB union has 28 entries (NONE + 27 real types).
        assert_eq!(m.node_types.len(), 27, "node type count drift vs schema");
        let names: std::collections::HashSet<&str> =
            m.node_types.iter().map(|n| n.name).collect();
        for required in ["Container", "Surface", "TextNode", "MediaNode", "FormNode"] {
            assert!(names.contains(required), "missing core type {required}");
        }
    }

    #[test]
    fn compile_targets_include_dom_oracle() {
        let m = build("0.1.0", cli_stub());
        assert!(m.compile_targets.iter().any(|t| t.id == "dom"));
        assert_eq!(m.compile_targets.len(), 7);
    }

    #[test]
    fn manifest_serializes_to_json() {
        let m = build("0.1.0", cli_stub());
        let json = serde_json::to_value(&m).expect("manifest serializes");
        assert_eq!(json["contract_version"], CONTRACT_VERSION);
        assert!(!json["diagnostic_codes"].as_array().unwrap().is_empty());
        assert!(json["compile_targets"].is_array());
    }
}
