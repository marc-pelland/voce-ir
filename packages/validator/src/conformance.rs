//! S91 Slice 1 — publishable conformance runner.
//!
//! Graduates the cross-target-parity test machinery (S68) into a
//! library API + CLI command so the conformance kit is *runnable* by
//! third parties — anyone shipping a Voce compiler in another
//! language, or a custom adapter, can certify against the same
//! semantic-equivalence checks the reference implementation uses.
//!
//! The semantic-summary algorithm lives in `crate::semantic_summary`
//! (the S91 D3 specified extractor). This module adds:
//!
//! - **Levels** (`Core` / `Standard` / `Full`) — capability tiers a
//!   target may claim.
//! - **Profiles** — per-target preservation contracts. Derived from
//!   `targets::ConformanceClass` so a target's classification in the
//!   single canonical registry determines what conformance asks of it.
//! - **`ConformanceReport`** — contract-versioned envelope third
//!   parties consume.
//! - **`run`** — dependency-injected runner: the caller supplies a
//!   compile function so the lib stays free of cross-crate compiler
//!   deps (the CLI binary wires in the actual compilers).

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::semantic_summary::SemanticSummary;
use crate::targets::{ConformanceClass, TargetInfo, ALL as TARGETS};

/// Capability tier a target may certify at. Higher tiers include
/// everything in the lower ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    /// Layout, text, semantics, named media. Every conformant target
    /// MUST pass Core.
    Core,
    /// + forms, landmarks, links.
    Standard,
    /// + gestures (JS-driven interactivity).
    Full,
}

impl std::str::FromStr for Level {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "core" => Ok(Level::Core),
            "standard" => Ok(Level::Standard),
            "full" => Ok(Level::Full),
            other => Err(format!("unknown conformance level: {other}")),
        }
    }
}

/// Which semantic dimensions a target is *required* to preserve under
/// the chosen level. A `false` flag means the medium legitimately may
/// degrade that dimension (informational, not a failure).
#[derive(Debug, Clone, Default, Serialize, schemars::JsonSchema)]
pub struct Profile {
    pub headings: bool,
    pub media_names: bool,
    pub landmarks: bool,
    pub forms: bool,
    pub links: bool,
    pub gestures: bool,
}

/// Result of one fixture × target check. Status semantics:
///
/// - `Pass` — every required dimension preserved exactly.
/// - `PassDegraded` — required dimensions preserved; some optional
///   dimension legitimately differed (recorded in `divergences`).
/// - `Fail` — a required dimension diverged. Bug.
/// - `NotApplicable` — HTML-scraping is the wrong lens (e.g. WebGPU
///   paints on GPU); needs an out-of-band extractor.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct FixtureResult {
    pub fixture: String,
    pub status: FixtureStatus,
    /// Human-readable divergence notes. Empty on `Pass`; populated for
    /// `PassDegraded` / `Fail` so an agent can route remediation.
    pub divergences: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum FixtureStatus {
    Pass,
    PassDegraded,
    Fail,
    NotApplicable,
}

#[derive(Debug, Clone, Default, Serialize, schemars::JsonSchema)]
pub struct ConformanceSummary {
    pub pass: usize,
    pub pass_degraded: usize,
    pub fail: usize,
    pub not_applicable: usize,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ConformanceReport {
    pub contract_version: &'static str,
    pub target: String,
    pub level: Level,
    pub profile: Profile,
    /// Overall: `Pass` if no `Fail`s and every required fixture passed
    /// (degraded counts as pass); `Fail` if any `Fail`.
    pub overall: FixtureStatus,
    pub fixtures: Vec<FixtureResult>,
    pub summary: ConformanceSummary,
}

/// The default 13-fixture corpus that ships with Voce. Same set the
/// S68 cross-target-parity test exercises; reused here so the runner
/// has a stable baseline corpus without a separate `conformance/`
/// duplicate.
pub const DEFAULT_CORPUS: &[&str] = &[
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

/// Derive the per-target preservation contract for the chosen level.
/// Maps `ConformanceClass` to which dimensions are required.
pub fn profile_for(target: &TargetInfo, level: Level) -> Profile {
    let core = matches!(level, Level::Core | Level::Standard | Level::Full);
    let standard = matches!(level, Level::Standard | Level::Full);
    let full = matches!(level, Level::Full);

    match target.conformance_class {
        ConformanceClass::OracleFull | ConformanceClass::OracleFullSuperset => Profile {
            headings: core,
            media_names: core,
            landmarks: core,
            forms: standard,
            links: standard,
            gestures: full,
        },
        ConformanceClass::RequiredContract => Profile {
            // Email-class: headings + named media at Core; links at
            // Standard. Forms / gestures / landmarks degrade by medium
            // (false at every level — informational, not required).
            headings: core,
            media_names: core,
            landmarks: false,
            forms: false,
            links: standard,
            gestures: false,
        },
        // Out-of-HTML-lens targets — every fixture is N/A under this
        // runner. A future slice adds the right extractor per target.
        ConformanceClass::NonHtmlVisual
        | ConformanceClass::LogicOnly
        | ConformanceClass::Native => Profile::default(),
    }
}

/// Look up a target by id from the canonical registry.
pub fn find_target(id: &str) -> Option<&'static TargetInfo> {
    TARGETS.iter().find(|t| t.id == id)
}

/// Compare an observed summary to the expected, returning the divergence
/// notes the profile cares about. A floor convention applies to count
/// dimensions (links / media): observed must be `>=` expected — a
/// target may legitimately *add* a skip link, never drop one.
pub fn diff(expected: &SemanticSummary, observed: &SemanticSummary, p: &Profile) -> Vec<String> {
    let mut out = Vec::new();
    if p.headings && observed.heading_levels != expected.heading_levels {
        out.push(format!(
            "heading_levels IR={:?} got={:?}",
            expected.heading_levels, observed.heading_levels
        ));
    }
    if p.forms && observed.form_field_count != expected.form_field_count {
        out.push(format!(
            "form_field_count IR={} got={}",
            expected.form_field_count, observed.form_field_count
        ));
    }
    if p.media_names && observed.media_with_name_count != expected.media_with_name_count {
        out.push(format!(
            "media_with_name_count IR={} got={}",
            expected.media_with_name_count, observed.media_with_name_count
        ));
    }
    if p.links && observed.link_count < expected.link_count {
        out.push(format!(
            "link_count IR={} got={} (dropped)",
            expected.link_count, observed.link_count
        ));
    }
    if p.gestures && observed.gesture_count < expected.gesture_count {
        out.push(format!(
            "gesture_count IR={} got={} (dropped)",
            expected.gesture_count, observed.gesture_count
        ));
    }
    if p.landmarks {
        for role in &expected.landmark_roles {
            if !observed.landmark_roles.contains(role) {
                out.push(format!(
                    "landmark '{role}' in IR, missing (got {:?})",
                    observed.landmark_roles
                ));
            }
        }
    }
    out
}

/// Run conformance against `target` using `compile_fn` (the dependency
/// injection that keeps this lib free of cross-compiler deps).
///
/// `compile_fn` receives the IR JSON and returns either compiled HTML
/// for HTML-family targets, or `None` if the target's compile failed —
/// in which case the fixture is marked `Fail` with an error note.
pub fn run<F>(
    target: &TargetInfo,
    level: Level,
    corpus_root: &Path,
    corpus: &[&str],
    mut compile_fn: F,
) -> ConformanceReport
where
    F: FnMut(&str) -> Result<String, String>,
{
    let profile = profile_for(target, level);
    let mut fixtures: Vec<FixtureResult> = Vec::new();
    let mut summary = ConformanceSummary::default();

    // Non-HTML-lens targets short-circuit to NotApplicable per fixture —
    // honest classification (cf. S68 WebGPU finding) rather than
    // pretending to verify something the lens can't see.
    let html_lens_applicable = matches!(
        target.conformance_class,
        ConformanceClass::OracleFull
            | ConformanceClass::OracleFullSuperset
            | ConformanceClass::RequiredContract
    );

    for &name in corpus {
        if !html_lens_applicable {
            fixtures.push(FixtureResult {
                fixture: name.to_string(),
                status: FixtureStatus::NotApplicable,
                divergences: vec![format!(
                    "target {} ({:?}) requires an out-of-HTML-lens extractor (S91 Slice 2+)",
                    target.id, target.conformance_class
                )],
            });
            summary.not_applicable += 1;
            continue;
        }

        let path: PathBuf = corpus_root.join(name);
        let ir_json = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                fixtures.push(FixtureResult {
                    fixture: name.to_string(),
                    status: FixtureStatus::Fail,
                    divergences: vec![format!("read {}: {e}", path.display())],
                });
                summary.fail += 1;
                continue;
            }
        };

        let expected = match SemanticSummary::from_ir(&ir_json) {
            Ok(s) => s,
            Err(e) => {
                fixtures.push(FixtureResult {
                    fixture: name.to_string(),
                    status: FixtureStatus::Fail,
                    divergences: vec![format!("IR parse: {e}")],
                });
                summary.fail += 1;
                continue;
            }
        };

        let html = match compile_fn(&ir_json) {
            Ok(h) => h,
            Err(e) => {
                fixtures.push(FixtureResult {
                    fixture: name.to_string(),
                    status: FixtureStatus::Fail,
                    divergences: vec![format!("compile: {e}")],
                });
                summary.fail += 1;
                continue;
            }
        };

        let observed = SemanticSummary::from_html(&html);
        let divergences = diff(&expected, &observed, &profile);

        let status = if divergences.is_empty() {
            FixtureStatus::Pass
        } else {
            FixtureStatus::Fail
        };
        match status {
            FixtureStatus::Pass => summary.pass += 1,
            FixtureStatus::PassDegraded => summary.pass_degraded += 1,
            FixtureStatus::Fail => summary.fail += 1,
            FixtureStatus::NotApplicable => summary.not_applicable += 1,
        }
        fixtures.push(FixtureResult {
            fixture: name.to_string(),
            status,
            divergences,
        });
    }

    let overall = if !html_lens_applicable {
        FixtureStatus::NotApplicable
    } else if summary.fail > 0 {
        FixtureStatus::Fail
    } else if summary.pass_degraded > 0 {
        FixtureStatus::PassDegraded
    } else {
        FixtureStatus::Pass
    };

    ConformanceReport {
        contract_version: crate::skills::CONTRACT_VERSION,
        target: target.id.to_string(),
        level,
        profile,
        overall,
        fixtures,
        summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_for_dom_oracle_at_full_requires_everything() {
        let dom = find_target("dom").unwrap();
        let p = profile_for(dom, Level::Full);
        assert!(p.headings && p.media_names && p.landmarks && p.forms && p.links && p.gestures);
    }

    #[test]
    fn profile_for_email_at_standard_requires_links_but_not_forms() {
        let email = find_target("email").unwrap();
        let p = profile_for(email, Level::Standard);
        assert!(p.headings && p.media_names && p.links);
        assert!(!p.forms, "email legitimately can't carry forms");
        assert!(!p.gestures, "email has no JS");
        assert!(!p.landmarks, "email layout is table-based");
    }

    #[test]
    fn profile_for_email_at_core_does_not_require_links() {
        let email = find_target("email").unwrap();
        let p = profile_for(email, Level::Core);
        assert!(p.headings && p.media_names);
        assert!(!p.links, "links are a Standard-level requirement, not Core");
    }

    #[test]
    fn non_html_targets_get_empty_profile() {
        for id in ["webgpu", "ios-swiftui", "android-compose", "wasm"] {
            let t = find_target(id).expect(id);
            let p = profile_for(t, Level::Full);
            assert!(!p.headings && !p.media_names && !p.landmarks);
        }
    }

    #[test]
    fn diff_detects_dropped_link_under_links_required() {
        let expected = SemanticSummary { link_count: 4, ..SemanticSummary::default() };
        let observed = SemanticSummary::default(); // link_count = 0
        let p = Profile { links: true, ..Profile::default() };
        let d = diff(&expected, &observed, &p);
        assert!(d.iter().any(|s| s.contains("link_count IR=4 got=0")));
    }

    #[test]
    fn diff_ignores_dimensions_the_profile_does_not_require() {
        let expected = SemanticSummary { gesture_count: 3, ..SemanticSummary::default() };
        let observed = SemanticSummary::default();
        let p = Profile::default(); // gestures: false
        let d = diff(&expected, &observed, &p);
        assert!(d.is_empty(), "non-required gesture drop is not a divergence");
    }

    #[test]
    fn run_against_non_html_target_yields_n_a_across_corpus() {
        let wgpu = find_target("webgpu").unwrap();
        let r = run(
            wgpu,
            Level::Full,
            std::path::Path::new("/nonexistent"), // never read
            &["a.voce.json", "b.voce.json"],
            |_| Ok(String::new()),
        );
        assert_eq!(r.overall, FixtureStatus::NotApplicable);
        assert_eq!(r.summary.not_applicable, 2);
        assert_eq!(r.summary.fail, 0);
    }
}
