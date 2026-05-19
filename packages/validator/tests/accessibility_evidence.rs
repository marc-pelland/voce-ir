//! S82 D8 — accessibility evidence, machine-checked.
//!
//! Rather than hand-written per-fixture evidence files that rot, the
//! "receipt" is a continuously-verified guarantee: every fixture in the
//! reference corpus passes the project's own validator with **zero
//! error-severity diagnostics**. If a fixture regresses (or a new one
//! is added that fails a11y/forms/etc.), this fails. The narrative and
//! the documented non-blocking warnings live in
//! `docs/accessibility/EVIDENCE.md`.

use std::fs;
use std::path::PathBuf;

use voce_validator::{Severity, validate};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Same corpus the cross-target parity verifier uses.
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

/// Every reference fixture must validate clean (zero errors). This is
/// the standing accessibility (and overall validity) receipt.
#[test]
fn reference_corpus_has_zero_validation_errors() {
    let mut failures = Vec::new();

    for &name in CORPUS {
        let path = workspace_root().join("tests/fixtures").join(name);
        let json = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {name}: {e}"));
        let result = validate(&json).unwrap_or_else(|e| panic!("{name}: validate failed: {e}"));

        let errors: Vec<String> = result
            .diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| format!("{} {}", d.code, d.message))
            .collect();

        if !errors.is_empty() {
            failures.push(format!("{name}:\n    - {}", errors.join("\n    - ")));
        }
    }

    assert!(
        failures.is_empty(),
        "Reference fixtures must validate with zero errors \
         (accessibility is a compile error — see docs/accessibility/):\n  {}",
        failures.join("\n  ")
    );
}
