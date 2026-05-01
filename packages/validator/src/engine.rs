//! Validation engine — orchestrates passes and collects diagnostics.

use std::collections::HashMap;

use crate::config::ValidatorConfig;
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

/// Run all validation passes on a JSON IR string with default config.
///
/// Returns a `ValidationResult` containing all diagnostics from all passes,
/// plus per-pass execution metadata (`result.passes`) for downstream consumers
/// that want timing and outcome per pass — see `--verbose-passes` in the CLI.
/// The JSON string must be the Voce IR canonical JSON format.
///
/// For project-specific severity overrides, use `validate_with_config`.
pub fn validate(json: &str) -> Result<ValidationResult, String> {
    validate_with_config(json, &ValidatorConfig::default())
}

/// Run all validation passes with project config applied. Severity overrides
/// from `config.severity_overrides` are applied before per-pass error/warning
/// counts are computed, so all downstream metrics reflect the final severity.
pub fn validate_with_config(
    json: &str,
    config: &ValidatorConfig,
) -> Result<ValidationResult, String> {
    let ir: VoceIr =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse IR JSON: {e}"))?;

    let index = NodeIndex::build(&ir);
    let mut result = ValidationResult::default();

    // Pre-build a code → hint lookup so engine can inject hints onto every
    // diagnostic without each pass having to remember to do it. Hints live
    // on CodeMeta in the per-pass CODES consts; this is the single point
    // where they get attached to runtime diagnostics.
    let hints = build_hint_lookup();

    for pass in passes::all_passes() {
        let before = result.diagnostics.len();
        let duration_us = time_us(|| pass.run(&ir, &index, &mut result));

        let mut error_count = 0;
        let mut warning_count = 0;
        let mut codes: Vec<String> = Vec::new();
        for diag in result.diagnostics[before..].iter_mut() {
            // Project-level severity override comes first so the rest of this
            // loop (counts, codes list) reflects the final severity.
            if let Some(&overridden) = config.severity_overrides.get(diag.code.as_str()) {
                diag.severity = overridden;
            }
            match diag.severity {
                Severity::Error => error_count += 1,
                Severity::Warning => warning_count += 1,
            }
            if !codes.contains(&diag.code) {
                codes.push(diag.code.clone());
            }
            if diag.hint.is_none()
                && let Some(hint) = hints.get(diag.code.as_str())
            {
                diag.hint = Some((*hint).to_string());
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

/// Aggregate every pass's CodeMeta into a code → hint lookup. Built once per
/// validate() call; cheap because there are <50 codes total.
fn build_hint_lookup() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    for pass in passes::all_passes() {
        for meta in pass.codes() {
            map.insert(meta.code, meta.hint);
        }
    }
    map
}
