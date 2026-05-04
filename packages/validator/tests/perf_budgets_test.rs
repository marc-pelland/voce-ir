//! Per-fixture perf budgets (S71 Day 3).
//!
//! Reads tests/perf-budgets.toml at the workspace root, compiles each
//! listed fixture, and asserts the result is within the budget.
//!
//! Methodology:
//!   - Each fixture is compiled 10 times in a tight loop. The MINIMUM
//!     wall time wins. Min-of-N is the standard way to filter out OS
//!     scheduler noise on micro-benchmarks of this size.
//!   - Output bytes are exact-deterministic — measured once, asserted
//!     against `max_output_bytes`.
//!   - On regression, the failure message names the fixture, the
//!     observed value, the budget, and the absolute / percent overage.
//!     Reviewers can update the TOML in a chore commit if the change is
//!     intentional, or fix the regression if it isn't.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, serde::Deserialize)]
struct BudgetsFile {
    fixture: BTreeMap<String, FixtureBudget>,
}

#[derive(Debug, serde::Deserialize)]
struct FixtureBudget {
    path: String,
    /// u64 (not u128) because the toml crate doesn't deserialize u128.
    /// A u64 µs holds ~580k years; this is fine.
    max_compile_us: u64,
    max_output_bytes: usize,
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn load_budgets() -> BudgetsFile {
    let path = workspace_root().join("tests/perf-budgets.toml");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
    toml::from_str(&raw).unwrap_or_else(|e| panic!("Failed to parse perf-budgets.toml: {e}"))
}

fn measure_min(json: &str, runs: usize) -> (u64, usize) {
    assert!(runs > 0, "must measure at least once");
    let opts = voce_compiler_dom::CompileOptions::default();
    let mut min_us: u64 = u64::MAX;
    let mut output_bytes = 0;
    for _ in 0..runs {
        let start = Instant::now();
        let result = voce_compiler_dom::compile(json, &opts)
            .expect("compile must succeed in perf budget test");
        let us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
        if us < min_us {
            min_us = us;
            output_bytes = result.size_bytes;
        }
    }
    (min_us, output_bytes)
}

#[test]
fn perf_budgets_for_every_fixture() {
    let budgets = load_budgets();
    assert!(
        !budgets.fixture.is_empty(),
        "perf-budgets.toml has no fixtures"
    );

    let mut failures: Vec<String> = Vec::new();

    for (name, budget) in &budgets.fixture {
        let path = workspace_root().join(&budget.path);
        let json = match fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                failures.push(format!(
                    "{name}: failed to read fixture {}: {e}",
                    path.display()
                ));
                continue;
            }
        };

        let (min_us, output_bytes) = measure_min(&json, 10);

        if min_us > budget.max_compile_us {
            let over_pct = (min_us as f64 - budget.max_compile_us as f64)
                / budget.max_compile_us as f64
                * 100.0;
            failures.push(format!(
                "{name}: compile time {min_us} µs > budget {} µs (+{over_pct:.0}%)",
                budget.max_compile_us
            ));
        }

        if output_bytes > budget.max_output_bytes {
            let over_pct = (output_bytes as f64 - budget.max_output_bytes as f64)
                / budget.max_output_bytes as f64
                * 100.0;
            failures.push(format!(
                "{name}: output {output_bytes} B > budget {} B (+{over_pct:.0}%)",
                budget.max_output_bytes
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "perf budget regression(s) — review and update tests/perf-budgets.toml \
             via a chore commit if the change is intentional:\n\n  {}\n",
            failures.join("\n  ")
        );
    }
}

#[test]
fn perf_budgets_toml_is_well_formed() {
    // Cheap sanity test: parses, every fixture path resolves, every
    // budget is positive. Catches typos / missing fixtures fast even
    // when the perf test itself is too slow / noisy to surface them.
    let budgets = load_budgets();
    for (name, budget) in &budgets.fixture {
        assert!(
            budget.max_compile_us > 0,
            "{name}: max_compile_us must be > 0"
        );
        assert!(
            budget.max_output_bytes > 0,
            "{name}: max_output_bytes must be > 0"
        );
        let path = workspace_root().join(&budget.path);
        assert!(
            path.exists(),
            "{name}: fixture path does not exist: {}",
            path.display()
        );
    }
}
