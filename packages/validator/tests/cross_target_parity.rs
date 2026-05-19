//! S68 cross-target parity — semantic-equivalence verifier.
//!
//! Distinct from `cross_target_tests.rs` (per-target smoke tests): this
//! derives a representation-independent [`SemanticSummary`] from each
//! fixture's IR (the contract) and from each compiled artifact, and
//! asserts every target preserves the IR's meaning *to the extent its
//! medium allows*. Divergence is classified, never silent:
//!
//! - **DOM** (oracle) and **Hybrid** (DOM + WASM): must preserve
//!   everything — they can represent any IR.
//! - **Email**: a constrained medium. Must preserve heading order and
//!   named media; links/forms/landmarks degrade (see the compatibility
//!   matrix — the link flattening is tracked for Deliverable 5).
//! - **WebGPU**: paints on the GPU behind a fixed HTML shell, so
//!   HTML-scraped parity is not the right lens; it needs an a11y-tree
//!   extractor (future slice). Smoke-covered in `cross_target_tests.rs`.
//!
//! SwiftUI/Compose/WASM language-specific extraction is a later slice.

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

/// Which semantic dimensions a target is *required* to preserve. A
/// `false` flag means the medium legitimately degrades that dimension —
/// recorded in `docs/compatibility-matrix.md`, not asserted here.
struct Profile {
    headings: bool,
    forms: bool,
    media_names: bool,
    links: bool,
    gestures: bool,
    landmarks: bool,
}

/// Compare an observed summary against the IR contract under a profile.
/// Returns human-readable divergence strings for the *required* dims.
fn divergences(
    name: &str,
    target: &str,
    expected: &SemanticSummary,
    observed: &SemanticSummary,
    p: &Profile,
) -> Vec<String> {
    let mut out = Vec::new();
    if p.headings && observed.heading_levels != expected.heading_levels {
        out.push(format!(
            "{name} [{target}]: heading_levels IR={:?} got={:?}",
            expected.heading_levels, observed.heading_levels
        ));
    }
    if p.forms && observed.form_field_count != expected.form_field_count {
        out.push(format!(
            "{name} [{target}]: form_field_count IR={} got={}",
            expected.form_field_count, observed.form_field_count
        ));
    }
    if p.media_names && observed.media_with_name_count != expected.media_with_name_count {
        out.push(format!(
            "{name} [{target}]: media_with_name_count IR={} got={}",
            expected.media_with_name_count, observed.media_with_name_count
        ));
    }
    // Links/gestures are floors: a target may legitimately *add*
    // affordances (skip links, wrapping anchors) but must never drop
    // them. Tracked separately — a JS-less medium keeps links but not
    // gestures, and conflating the two would hide that.
    if p.links && observed.link_count < expected.link_count {
        out.push(format!(
            "{name} [{target}]: link_count IR={} got={} (dropped)",
            expected.link_count, observed.link_count
        ));
    }
    if p.gestures && observed.gesture_count < expected.gesture_count {
        out.push(format!(
            "{name} [{target}]: gesture_count IR={} got={} (dropped)",
            expected.gesture_count, observed.gesture_count
        ));
    }
    if p.landmarks {
        for role in &expected.landmark_roles {
            if !observed.landmark_roles.contains(role) {
                out.push(format!(
                    "{name} [{target}]: landmark '{role}' in IR, missing (got {:?})",
                    observed.landmark_roles
                ));
            }
        }
    }
    out
}

/// DOM (oracle) — must represent every IR faithfully.
#[test]
fn dom_oracle_preserves_ir_semantics() {
    let p = Profile {
        headings: true,
        forms: true,
        media_names: true,
        links: true,
        gestures: true,
        landmarks: true,
    };
    let mut failures = Vec::new();
    for &name in CORPUS {
        let json = load(name);
        let expected = SemanticSummary::from_ir(&json).expect("IR parse");
        let html = voce_compiler_dom::compile(&json, &voce_compiler_dom::CompileOptions::default())
            .expect("DOM compile")
            .html;
        let observed = SemanticSummary::from_html(&html);
        failures.extend(divergences(name, "DOM", &expected, &observed, &p));
    }
    assert!(
        failures.is_empty(),
        "DOM oracle diverged (a compiler bug — DOM can represent everything):\n  {}",
        failures.join("\n  ")
    );
}

/// Hybrid = DOM + WASM. It is a superset of DOM and must hold the same
/// full-preservation contract as the oracle.
#[test]
fn hybrid_matches_oracle_contract() {
    let p = Profile {
        headings: true,
        forms: true,
        media_names: true,
        links: true,
        gestures: true,
        landmarks: true,
    };
    let mut failures = Vec::new();
    for &name in CORPUS {
        let json = load(name);
        let expected = SemanticSummary::from_ir(&json).expect("IR parse");
        let html = voce_compiler_hybrid::compile_hybrid(
            &json,
            &voce_compiler_hybrid::HybridCompileOptions::default(),
        )
        .expect("Hybrid compile")
        .html;
        let observed = SemanticSummary::from_html(&html);
        failures.extend(divergences(name, "Hybrid", &expected, &observed, &p));
    }
    assert!(
        failures.is_empty(),
        "Hybrid diverged from the oracle contract:\n  {}",
        failures.join("\n  ")
    );
}

/// Email is a constrained medium. Its *contract* is heading order,
/// named media, and — since the Deliverable-5 anchor fix — interactive
/// links. Forms and landmarks degrade by the medium's nature (email
/// clients block forms; email layout is table-based with no landmark
/// elements). Asserting the required dimensions keeps this honest: a
/// regression in what Email *must* keep still fails.
#[test]
fn email_preserves_its_required_contract() {
    let p = Profile {
        headings: true,
        forms: false,
        media_names: true,
        links: true,
        gestures: false,
        landmarks: false,
    };
    let mut failures = Vec::new();
    for &name in CORPUS {
        let json = load(name);
        let expected = SemanticSummary::from_ir(&json).expect("IR parse");
        let html = voce_compiler_email::compile_email(&json)
            .expect("Email compile")
            .html;
        let observed = SemanticSummary::from_html(&html);
        failures.extend(divergences(name, "Email", &expected, &observed, &p));
    }
    assert!(
        failures.is_empty(),
        "Email dropped a dimension it is required to preserve \
         (heading order / named media):\n  {}",
        failures.join("\n  ")
    );
}

/// Diagnostic dump (ignored by default): prints IR vs every HTML-family
/// target so an agent or maintainer can inspect divergence at a glance.
/// Run with `--ignored --nocapture`.
#[test]
#[ignore]
fn diagnostic_html_family_dump() {
    for &name in CORPUS {
        let json = load(name);
        let ir = SemanticSummary::from_ir(&json).unwrap();
        println!("\n=== {name} ===");
        let row = |t: &str, s: &SemanticSummary| {
            println!(
                "{t:6} h={:?} link={} gest={} form={} media={}/{} lm={:?}",
                s.heading_levels,
                s.link_count,
                s.gesture_count,
                s.form_field_count,
                s.media_with_name_count,
                s.media_decorative_count,
                s.landmark_roles
            );
        };
        row("IR", &ir);
        if let Ok(r) =
            voce_compiler_dom::compile(&json, &voce_compiler_dom::CompileOptions::default())
        {
            row("DOM", &SemanticSummary::from_html(&r.html));
        }
        if let Ok(r) = voce_compiler_email::compile_email(&json) {
            row("EMAIL", &SemanticSummary::from_html(&r.html));
        }
        if let Ok(r) = voce_compiler_webgpu::compile_webgpu(
            &json,
            &voce_compiler_webgpu::WebGpuCompileOptions::default(),
        ) {
            row("WGPU", &SemanticSummary::from_html(&r.html));
        }
        if let Ok(r) = voce_compiler_hybrid::compile_hybrid(
            &json,
            &voce_compiler_hybrid::HybridCompileOptions::default(),
        ) {
            row("HYBR", &SemanticSummary::from_html(&r.html));
        }
    }
}
