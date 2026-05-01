//! Validation passes. Each pass checks one category of rules.
//!
//! All passes implement the `ValidationPass` trait and are registered
//! in `all_passes()`. Passes run in dependency order: structural
//! checks first, then reference resolution, then domain-specific.

pub mod a11y;
pub mod forms;
pub mod i18n;
pub mod motion;
pub mod references;
pub mod security;
pub mod seo;
pub mod state_machine;
pub mod structural;

use crate::errors::{CodeMeta, ValidationResult};
use crate::index::NodeIndex;
use crate::ir::VoceIr;

/// Trait for a validation pass.
pub trait ValidationPass {
    /// Unique name for this pass (e.g., "structural", "references").
    fn name(&self) -> &'static str;

    /// Run the pass, appending diagnostics to the result.
    fn run(&self, ir: &VoceIr, index: &NodeIndex, result: &mut ValidationResult);

    /// Static catalogue of diagnostic codes this pass can emit. Used by
    /// `voce validate --list-codes`. Default empty for passes that haven't
    /// declared their codes yet (S67 transition state).
    fn codes(&self) -> &'static [CodeMeta] {
        &[]
    }
}

/// Return all registered validation passes in execution order.
pub fn all_passes() -> Vec<Box<dyn ValidationPass>> {
    vec![
        // Core passes (Sprint 06)
        Box::new(structural::StructuralPass),
        Box::new(references::ReferencesPass),
        Box::new(state_machine::StateMachinePass),
        // Pillar passes (Sprint 07)
        Box::new(a11y::AccessibilityPass),
        Box::new(security::SecurityPass),
        Box::new(seo::SeoPass),
        Box::new(forms::FormsPass),
        Box::new(i18n::I18nPass),
        Box::new(motion::MotionPass),
    ]
}
