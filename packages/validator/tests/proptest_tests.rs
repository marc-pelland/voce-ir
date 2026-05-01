//! Property-based tests for the validator (S69).
//!
//! These exercise invariants the validator must hold *for any input*, not
//! just curated cases. The strategies use generic strings and small JSON
//! shapes rather than fully-formed IR — that keeps the tests fast and
//! catches the most embarrassing class of bugs (panics, nondeterminism,
//! drift between catalog and runtime).

use proptest::prelude::*;
use serde_json::{Value, json};
use std::collections::HashSet;

use voce_validator::errors::CodeMeta;
use voce_validator::passes::all_passes;
use voce_validator::{Severity, validate};

// ── Strategies ──────────────────────────────────────────────────────────────

/// Random JSON values up to depth 3. Used to exercise the validator's input
/// surface without building a full IR strategy.
fn arb_json() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|n| json!(n)),
        ".*".prop_map(Value::String),
    ];
    leaf.prop_recursive(3, 16, 8, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..6).prop_map(Value::Array),
            prop::collection::hash_map("[a-z][a-z_0-9]*", inner, 0..6).prop_map(|m| {
                let mut obj = serde_json::Map::new();
                for (k, v) in m {
                    obj.insert(k, v);
                }
                Value::Object(obj)
            }),
        ]
    })
}

// ── Properties ──────────────────────────────────────────────────────────────

proptest! {
    /// validate() must return a Result for any string — never panic. The most
    /// basic invariant; catches integer-overflow and unwrap/expect regressions.
    #[test]
    fn validate_never_panics_on_arbitrary_strings(input in ".{0,2000}") {
        let _ = validate(&input);
    }

    /// validate() must return a Result for any JSON value too.
    #[test]
    fn validate_never_panics_on_arbitrary_json(v in arb_json()) {
        let json_str = serde_json::to_string(&v).unwrap();
        let _ = validate(&json_str);
    }

    /// validate() is a pure function of its input. Two calls with the same
    /// input must yield the same Ok/Err shape, same error/warning counts,
    /// and the same code sequence in the same order.
    #[test]
    fn validate_is_deterministic(v in arb_json()) {
        let json_str = serde_json::to_string(&v).unwrap();
        let r1 = validate(&json_str);
        let r2 = validate(&json_str);
        match (r1, r2) {
            (Ok(a), Ok(b)) => {
                prop_assert_eq!(a.error_count(), b.error_count());
                prop_assert_eq!(a.warning_count(), b.warning_count());
                let codes_a: Vec<String> =
                    a.diagnostics.iter().map(|d| d.code.clone()).collect();
                let codes_b: Vec<String> =
                    b.diagnostics.iter().map(|d| d.code.clone()).collect();
                prop_assert_eq!(codes_a, codes_b);
            }
            (Err(e1), Err(e2)) => prop_assert_eq!(e1, e2),
            _ => prop_assert!(false, "validate yielded different Result variants"),
        }
    }

    /// has_errors / error_count / warning_count must agree internally:
    /// has_errors() is true iff error_count() > 0; counts sum to total.
    #[test]
    fn count_helpers_agree_with_diagnostics(v in arb_json()) {
        let json_str = serde_json::to_string(&v).unwrap();
        if let Ok(r) = validate(&json_str) {
            prop_assert_eq!(r.has_errors(), r.error_count() > 0);
            let total = r.diagnostics.len();
            prop_assert_eq!(r.error_count() + r.warning_count(), total);
            // Every diagnostic must be one of the two severities — no silent
            // pass-through of unknown variants.
            for d in &r.diagnostics {
                prop_assert!(
                    matches!(d.severity, Severity::Error | Severity::Warning)
                );
            }
        }
    }

    /// Every emitted diagnostic must reference a code that the catalog knows
    /// about, OR a "ParseError"/"INTERNAL" sentinel from input parsing.
    /// Catches drift between pass emission and CodeMeta declarations.
    #[test]
    fn every_emitted_code_is_in_catalog(v in arb_json()) {
        let json_str = serde_json::to_string(&v).unwrap();
        if let Ok(r) = validate(&json_str) {
            let known = catalog_codes();
            for d in &r.diagnostics {
                prop_assert!(
                    known.contains(d.code.as_str()) || d.code == "PARSE",
                    "code {:?} emitted but not in registry",
                    d.code
                );
            }
        }
    }
}

fn catalog_codes() -> HashSet<&'static str> {
    let mut set = HashSet::new();
    for pass in all_passes() {
        for meta in pass.codes() {
            let CodeMeta { code, .. } = *meta;
            set.insert(code);
        }
    }
    set
}
