//! S68 cross-target parity — semantic-equivalence verifier.
//!
//! Distinct from `cross_target_tests.rs` (per-target smoke tests): this
//! derives a representation-independent [`SemanticSummary`] from each
//! fixture's IR (the contract) and from the DOM-compiled artifact
//! (the *oracle* target, per S91), and asserts the oracle preserves the
//! IR's meaning. This is the foundation every other target is measured
//! against; SwiftUI/Compose/WASM observed extraction + the full
//! compatibility matrix land in follow-up slices.

use std::fs;
use std::path::PathBuf;

use voce_validator::semantic_summary::SemanticSummary;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// The curated parity corpus. These already exist in `tests/fixtures/`
/// and between them exercise headings, layout, links/nav, media, forms,
/// gestures, state machines, theming, and composition.
const CORPUS: &[&str] = &[
    "container-grid.voce.json",
    "container-row.voce.json",
    "decorative-surface.voce.json",
    "form-contact.voce.json",
    "gesture-tap.voce.json",
    "links-and-nav.voce.json",
    "media-image.voce.json",
    "nested-layout.voce.json",
    "semantic-a11y.voce.json",
    "state-machine.voce.json",
    "surface-card.voce.json",
    "text-heading.voce.json",
    "theme-dark.voce.json",
];

fn load(name: &str) -> String {
    let path = workspace_root().join("tests/fixtures").join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// The DOM target (oracle) must preserve the IR's semantics exactly on
/// the non-degradable dimensions. Any divergence here is a compiler bug,
/// not a medium limitation — DOM can represent everything.
#[test]
fn dom_oracle_preserves_ir_semantics_across_corpus() {
    let mut failures = Vec::new();

    for &name in CORPUS {
        let json = load(name);
        let expected = SemanticSummary::from_ir(&json)
            .unwrap_or_else(|e| panic!("{name}: IR parse failed: {e}"));

        let compiled = voce_compiler_dom::compile(&json, &voce_compiler_dom::CompileOptions::default())
            .unwrap_or_else(|e| panic!("{name}: DOM compile failed: {e:?}"));
        let observed = SemanticSummary::from_html(&compiled.html);

        // Heading hierarchy: order and levels must survive verbatim.
        if observed.heading_levels != expected.heading_levels {
            failures.push(format!(
                "{name}: heading_levels IR={:?} DOM={:?}",
                expected.heading_levels, observed.heading_levels
            ));
        }
        // Form fields must all be emitted.
        if observed.form_field_count != expected.form_field_count {
            failures.push(format!(
                "{name}: form_field_count IR={} DOM={}",
                expected.form_field_count, observed.form_field_count
            ));
        }
        // Named images must keep their accessible name.
        if observed.media_with_name_count != expected.media_with_name_count {
            failures.push(format!(
                "{name}: media_with_name_count IR={} DOM={}",
                expected.media_with_name_count, observed.media_with_name_count
            ));
        }
        // Interactive affordances must not be dropped (DOM may add some,
        // e.g. wrapping anchors — so this is a floor, not equality).
        if observed.interactive_count < expected.interactive_count {
            failures.push(format!(
                "{name}: interactive_count IR={} DOM={} (dropped)",
                expected.interactive_count, observed.interactive_count
            ));
        }
        // Every IR landmark must appear in the DOM output.
        for role in &expected.landmark_roles {
            if !observed.landmark_roles.contains(role) {
                failures.push(format!(
                    "{name}: landmark '{role}' present in IR, missing in DOM \
                     (DOM landmarks: {:?})",
                    observed.landmark_roles
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "DOM oracle diverged from IR semantics:\n  {}",
        failures.join("\n  ")
    );
}
