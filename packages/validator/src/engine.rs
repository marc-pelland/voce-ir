//! Validation engine — orchestrates passes and collects diagnostics.

use crate::errors::ValidationResult;
use crate::index::NodeIndex;
use crate::ir::VoceIr;
use crate::passes;

/// Run all validation passes on a JSON IR string.
///
/// Returns a `ValidationResult` containing all diagnostics from all passes.
/// The JSON string must be the Voce IR canonical JSON format.
pub fn validate(json: &str) -> Result<ValidationResult, String> {
    // Deserialize
    let ir: VoceIr =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse IR JSON: {e}"))?;

    // Build index
    let index = NodeIndex::build(&ir);

    // Run passes
    let mut result = ValidationResult::default();
    for pass in passes::all_passes() {
        pass.run(&ir, &index, &mut result);
    }

    Ok(result)
}
