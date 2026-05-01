//! Validation engine — orchestrates passes and collects diagnostics.

use crate::errors::{PassResult, Severity, ValidationResult};
use crate::index::NodeIndex;
use crate::ir::VoceIr;
use crate::passes;

/// Time a pass's execution in microseconds. On native targets this is real
/// wall-clock time via `std::time::Instant`. On `wasm32-unknown-unknown` the
/// std `Instant` panics ("time not implemented") so we report 0 — callers in
/// the browser should derive timing on the JS side via `performance.now()`
/// if they need it. F-025.
#[cfg(not(target_arch = "wasm32"))]
fn time_us(f: impl FnOnce()) -> u64 {
    let started = std::time::Instant::now();
    f();
    u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX)
}

#[cfg(target_arch = "wasm32")]
fn time_us(f: impl FnOnce()) -> u64 {
    f();
    0
}

/// Run all validation passes on a JSON IR string.
///
/// Returns a `ValidationResult` containing all diagnostics from all passes,
/// plus per-pass execution metadata (`result.passes`) for downstream consumers
/// that want timing and outcome per pass — see `--verbose-passes` in the CLI.
/// The JSON string must be the Voce IR canonical JSON format.
pub fn validate(json: &str) -> Result<ValidationResult, String> {
    let ir: VoceIr =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse IR JSON: {e}"))?;

    let index = NodeIndex::build(&ir);
    let mut result = ValidationResult::default();

    for pass in passes::all_passes() {
        let before = result.diagnostics.len();
        let duration_us = time_us(|| pass.run(&ir, &index, &mut result));

        let mut error_count = 0;
        let mut warning_count = 0;
        let mut codes: Vec<String> = Vec::new();
        for diag in &result.diagnostics[before..] {
            match diag.severity {
                Severity::Error => error_count += 1,
                Severity::Warning => warning_count += 1,
            }
            if !codes.contains(&diag.code) {
                codes.push(diag.code.clone());
            }
        }

        result.passes.push(PassResult {
            name: pass.name().to_string(),
            duration_us,
            error_count,
            warning_count,
            codes,
        });
    }

    Ok(result)
}
