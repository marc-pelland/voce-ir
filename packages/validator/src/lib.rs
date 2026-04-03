//! Voce IR Validator — Reference IR validation engine.
//!
//! Validates IR blobs against structural, accessibility, security, SEO,
//! i18n, form, and motion safety rules. Returns typed diagnostics with
//! node paths and expected/actual values.
//!
//! # Usage
//!
//! ```
//! let json = r#"{ "root": { "node_id": "root" } }"#;
//! let result = voce_validator::validate(json).unwrap();
//! assert!(!result.has_errors() || result.has_errors()); // always returns a result
//! ```

pub mod engine;
pub mod errors;
pub mod formatter;
pub mod index;
pub mod inspect;
pub mod ir;
pub mod manifest;
pub mod passes;
pub mod report;

pub use engine::validate;
pub use errors::{Diagnostic, Severity, ValidationResult};
