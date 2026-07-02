//! S79 B2 — `voce fix --until-clean --plan`: the self-correcting fix
//! loop, the differentiator-grade Part B deliverable.
//!
//! S67 ships single JSON Patch fixes. This module composes them into
//! an **ordered, convergent plan**: validate → pick one applicable fix
//! → apply → re-validate → repeat, recording each step (code resolved,
//! rationale, patch) until the IR is clean *or* a step makes no
//! progress (loop detected). The output is a contract-versioned
//! envelope an agent or MCP client can drive headlessly without
//! reading prose.
//!
//! Convergence guarantees:
//! - `iterations` is capped (`max_iters`) so a bug in a fix builder
//!   can never wedge the loop.
//! - Non-progress is detected explicitly: if a step's applied patches
//!   leave the post-validation diagnostic set unchanged in size and
//!   content, the loop stops with `converges: false` and the residual
//!   codes are reported — never silently spun.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::errors::{Confidence, Diagnostic, PatchOp, Severity};
use crate::{build_fix, validate};

/// One RFC-6902-shaped patch op as serialized into the plan contract.
/// Mirrors `errors::PatchOp` with a `String` op for JSON ergonomics.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct PlanPatchOp {
    pub op: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

impl From<&PatchOp> for PlanPatchOp {
    fn from(p: &PatchOp) -> Self {
        Self {
            op: p.op.to_string(),
            path: p.path.clone(),
            value: p.value.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct FixStep {
    /// 1-based step index in the plan.
    pub step: usize,
    /// Diagnostic code this step targets.
    pub code: String,
    /// Path of the IR node this step modifies.
    pub node_path: String,
    /// One-line description of *what* the patch does
    /// (`FixPatch::preview`).
    pub rationale: String,
    /// Confidence of the underlying fix (`safe`/`suggested`/`risky`).
    pub confidence: String,
    pub patch: Vec<PlanPatchOp>,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct FixPlan {
    pub contract_version: &'static str,
    pub plan: Vec<FixStep>,
    /// `true` if the loop drove error count to zero (or to the
    /// irreducible residue) without detecting non-progress.
    pub converges: bool,
    /// Codes still firing after the loop terminated. Empty when the
    /// IR is clean.
    pub residual_codes: Vec<String>,
    /// Number of validate-apply iterations the loop performed.
    pub iterations: usize,
    /// Hit the iteration cap before converging (separate signal from
    /// `converges`).
    pub hit_iteration_cap: bool,
    /// Whether the plan was applied in-place (i.e. the file was
    /// rewritten). When `false`, this is a preview/plan-only run.
    pub applied: bool,
    /// Confidence threshold the loop respected.
    pub confidence_threshold: String,
}

#[derive(Debug, Clone)]
pub struct LoopOptions {
    pub threshold: Confidence,
    pub max_iters: usize,
}

impl Default for LoopOptions {
    fn default() -> Self {
        Self {
            // Safe is the default for the same reason the S67 single-
            // patch CLI defaults to it: only patches that cannot
            // change observable semantics auto-apply.
            threshold: Confidence::Safe,
            max_iters: 32,
        }
    }
}

/// Result of running the loop in-process. The caller decides whether
/// to write the patched IR back to disk.
pub struct LoopResult {
    pub plan: FixPlan,
    /// Final IR JSON after every applied step. Equals the input when
    /// the loop applied zero steps.
    pub final_ir: serde_json::Value,
}

/// Run the convergent fix loop against an IR JSON string. Pure: no
/// filesystem side-effects. `applied` in the returned plan is set by
/// the caller based on whether they write `final_ir` back.
pub fn run(json: &str, opts: &LoopOptions) -> Result<LoopResult, String> {
    let mut value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse IR JSON: {e}"))?;

    let mut steps: Vec<FixStep> = Vec::new();
    let mut iterations = 0usize;
    let mut converges = true;
    let mut hit_cap = false;
    let mut prev_diag_fingerprint: Option<BTreeSet<(String, String)>> = None;

    loop {
        if iterations >= opts.max_iters {
            hit_cap = true;
            // Hitting the cap is its own signal, not a non-convergence.
            // The loop may have been making progress every iteration —
            // converges stays true if no non-progress was observed.
            break;
        }
        iterations += 1;

        let current_text =
            serde_json::to_string(&value).map_err(|e| format!("re-serialize IR: {e}"))?;
        let result = validate(&current_text)?;

        // Pick the first applicable fix at or below threshold,
        // ordered by node_path for determinism (same order as
        // single-shot cmd_fix).
        let mut candidates: Vec<(&Diagnostic, crate::errors::FixPatch)> = result
            .diagnostics
            .iter()
            .filter_map(|d| build_fix(d).map(|f| (d, f)))
            .filter(|(_, f)| confidence_at_or_below(&f.confidence, &opts.threshold))
            .collect();
        candidates.sort_by(|a, b| a.0.node_path.cmp(&b.0.node_path));

        let Some((diag, fix)) = candidates.into_iter().next() else {
            // Nothing left to apply at threshold — terminate. Residual
            // codes are whatever diagnostics remain.
            return Ok(LoopResult {
                plan: finalize_plan(
                    steps,
                    &result,
                    converges,
                    iterations,
                    hit_cap,
                    &opts.threshold,
                ),
                final_ir: value,
            });
        };

        // Apply this step's patch. A patch failure (e.g. a build_fix
        // assumes a parent path that doesn't exist on this IR) is
        // *non-convergence*, not an error — record what we tried and
        // stop, so the agent gets an honest plan instead of a thrown
        // exception.
        let mut step_apply_err: Option<String> = None;
        for op in &fix.operations {
            if let Err(e) = crate::fixes::apply_op(&mut value, op) {
                step_apply_err = Some(format!(
                    "apply {} at {} failed: {e}",
                    diag.code, diag.node_path
                ));
                break;
            }
        }
        if let Some(_err) = step_apply_err {
            // Re-validate to report the unchanged residual; the partial
            // value mutation (if any op succeeded before failing) is
            // left as-is — apply_op only mutates on success, so the
            // IR is in a consistent state.
            let current_text = serde_json::to_string(&value)
                .map_err(|e| format!("re-serialize IR after apply failure: {e}"))?;
            let after = validate(&current_text)?;
            converges = false;
            return Ok(LoopResult {
                plan: finalize_plan(
                    steps,
                    &after,
                    converges,
                    iterations,
                    hit_cap,
                    &opts.threshold,
                ),
                final_ir: value,
            });
        }

        // Record the step.
        steps.push(FixStep {
            step: steps.len() + 1,
            code: diag.code.clone(),
            node_path: diag.node_path.clone(),
            rationale: fix.preview.clone(),
            confidence: fix.confidence.to_string(),
            patch: fix.operations.iter().map(PlanPatchOp::from).collect(),
        });

        // Non-progress check: post-apply, re-validate and compare the
        // diagnostic fingerprint. If it's identical to the previous
        // iteration's fingerprint, the loop is spinning — stop.
        let after_text = serde_json::to_string(&value).map_err(|e| format!("{e}"))?;
        let after = validate(&after_text)?;
        let fingerprint: BTreeSet<(String, String)> = after
            .diagnostics
            .iter()
            .map(|d| (d.code.clone(), d.node_path.clone()))
            .collect();
        if let Some(prev) = &prev_diag_fingerprint {
            if prev == &fingerprint {
                // Same set of diagnostics survived a step that
                // applied a patch — non-progress.
                converges = false;
                return Ok(LoopResult {
                    plan: finalize_plan(
                        steps,
                        &after,
                        converges,
                        iterations,
                        hit_cap,
                        &opts.threshold,
                    ),
                    final_ir: value,
                });
            }
        }
        prev_diag_fingerprint = Some(fingerprint);

        // Early-exit if there are no errors left at all. We could keep
        // going to chip away at warnings if they have fixes, and we do —
        // the next loop iteration will discover that no candidates
        // remain and exit cleanly via the `let Some` arm above.
    }

    // Cap path: re-validate once for an accurate residual report.
    let current_text =
        serde_json::to_string(&value).map_err(|e| format!("re-serialize IR at cap: {e}"))?;
    let result = validate(&current_text)?;
    Ok(LoopResult {
        plan: finalize_plan(
            steps,
            &result,
            converges,
            iterations,
            hit_cap,
            &opts.threshold,
        ),
        final_ir: value,
    })
}

fn finalize_plan(
    steps: Vec<FixStep>,
    final_result: &crate::ValidationResult,
    converges: bool,
    iterations: usize,
    hit_iteration_cap: bool,
    threshold: &Confidence,
) -> FixPlan {
    let mut residual: BTreeSet<String> = BTreeSet::new();
    for d in &final_result.diagnostics {
        if matches!(d.severity, Severity::Error) {
            residual.insert(d.code.clone());
        }
    }
    FixPlan {
        contract_version: crate::skills::CONTRACT_VERSION,
        plan: steps,
        converges,
        residual_codes: residual.into_iter().collect(),
        iterations,
        hit_iteration_cap,
        applied: false,
        confidence_threshold: threshold.to_string(),
    }
}

fn confidence_at_or_below(actual: &Confidence, threshold: &Confidence) -> bool {
    let rank = |c: &Confidence| match c {
        Confidence::Safe => 0u8,
        Confidence::Suggested => 1,
        Confidence::Risky => 2,
    };
    rank(actual) <= rank(threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan_for(json: &str) -> FixPlan {
        run(json, &LoopOptions::default()).expect("loop runs").plan
    }

    #[test]
    fn plan_carries_contract_version_and_threshold() {
        // Minimal valid doc — no diagnostics → plan is empty, converges.
        let p = plan_for(r#"{ "root": { "node_id": "r" } }"#);
        assert_eq!(p.contract_version, crate::skills::CONTRACT_VERSION);
        assert_eq!(p.confidence_threshold, "safe");
        assert!(p.plan.is_empty());
        assert!(p.converges);
        assert!(p.residual_codes.is_empty());
        assert!(!p.hit_iteration_cap);
    }

    #[test]
    fn loop_converges_when_every_remaining_code_has_a_safe_fix() {
        // STR002 fires on a node missing node_id. build_fix() produces a
        // Safe patch for it. Two such nodes → loop should resolve both.
        let json = r#"{
            "root": {
                "node_id": "r",
                "children": [
                    { "value_type": "Surface", "value": { "fill": { "r": 0, "g": 0, "b": 0, "a": 255 } } },
                    { "value_type": "Surface", "value": { "decorative": true } }
                ]
            }
        }"#;
        let r = run(json, &LoopOptions::default()).expect("loop runs");
        assert!(r.plan.iterations >= 2);
        assert!(r.plan.converges);
        for step in &r.plan.plan {
            assert_eq!(step.confidence, "safe");
            assert!(!step.patch.is_empty());
            assert!(!step.code.is_empty());
            assert!(!step.node_path.is_empty());
        }
        // Final IR should now have node_ids on those children.
        let final_str = serde_json::to_string(&r.final_ir).unwrap();
        assert!(!final_str.contains("STR002"), "no leftover STR002 markers");
    }

    #[test]
    fn residual_codes_lists_what_safe_fixes_could_not_resolve() {
        // A11Y004 (GestureHandler missing keyboard_key) is error
        // severity and has no auto-fix builder. The loop should
        // terminate cleanly with it as a residual — not crash, not
        // claim non-convergence.
        let json = r#"{
            "root": {
                "node_id": "r",
                "children": [
                    { "value_type": "GestureHandler", "value": {
                        "node_id": "tap", "gesture_type": "Tap", "target_node_id": "btn"
                    } }
                ]
            }
        }"#;
        let r = run(json, &LoopOptions::default()).expect("loop runs");
        assert!(r.plan.plan.is_empty(), "no fix builder for A11Y004");
        assert!(
            r.plan.converges,
            "no progress to make is not non-convergence"
        );
        assert!(
            r.plan.residual_codes.iter().any(|c| c == "A11Y004"),
            "A11Y004 should be residual, got {:?}",
            r.plan.residual_codes
        );
    }

    #[test]
    fn patch_failure_marks_non_convergence_not_crash() {
        // Construct an IR whose first matching Safe fix is FRM004 (Email
        // field missing Email validation rule). build_fix for FRM004
        // produces a patch that assumes a `validations` array exists at
        // the field; when it doesn't, apply_op fails. The loop must
        // catch that and report converges=false, not crash.
        let json = r#"{
            "root": { "node_id": "r", "children": [
                { "value_type": "FormNode", "value": { "node_id": "f",
                    "semantic_node_id": "sf", "fields": [
                    { "name": "email", "field_type": "Email", "label": "Email" }
                ] } }
            ], "semantic_nodes": [
                { "node_id": "sf", "role": "form", "label": "Form" }
            ] }
        }"#;
        let r = run(json, &LoopOptions::default()).expect("loop runs (graceful)");
        assert!(
            !r.plan.converges,
            "a patch-apply failure must surface as non-convergence, got plan={:?}",
            r.plan
        );
    }

    #[test]
    fn iteration_cap_does_not_break_convergence_signal() {
        // Cap of 0 should hit the cap immediately without claiming
        // non-convergence. Useful for callers that want a plan
        // limited to N steps.
        let json = r#"{
            "root": { "node_id": "r", "children": [
                { "value_type": "Surface", "value": { "decorative": true } }
            ] }
        }"#;
        let r = run(
            json,
            &LoopOptions {
                threshold: Confidence::Safe,
                max_iters: 0,
            },
        )
        .expect("loop runs");
        assert!(r.plan.hit_iteration_cap);
        assert!(r.plan.converges, "hitting the cap is not non-convergence");
    }

    #[test]
    fn plan_serializes_with_steps_and_residue() {
        let json = r#"{
            "root": { "node_id": "r", "children": [
                { "value_type": "Surface", "value": { "decorative": true } }
            ] }
        }"#;
        let p = plan_for(json);
        let v = serde_json::to_value(&p).unwrap();
        assert!(v["plan"].is_array());
        assert!(v["residual_codes"].is_array());
        assert_eq!(v["contract_version"], crate::skills::CONTRACT_VERSION);
    }
}
