//! Voce IR DOM Compiler — Compiles validated IR to optimized HTML output.
//!
//! Takes validated IR and emits minimal, zero-framework HTML with inline
//! styles, surgical DOM mutations, and ARIA attributes.
//!
//! # Usage
//!
//! ```
//! use voce_compiler_dom::pipeline::{compile, CompileOptions};
//!
//! let json = r#"{ "root": { "node_id": "root", "children": [
//!     { "value_type": "TextNode", "value": { "node_id": "t", "content": "Hello" } }
//! ] } }"#;
//!
//! let result = compile(json, &CompileOptions::default()).unwrap();
//! assert!(result.html.contains("Hello"));
//! ```

pub mod animation;
pub mod assets;
pub mod compiler_ir;
pub mod emit;
pub mod pipeline;

pub use pipeline::{CompileOptions, CompileResult, compile};
