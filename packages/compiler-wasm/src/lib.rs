//! Voce IR WASM Compiler — compiles state machines and compute nodes to WebAssembly.
//!
//! Generates WAT (WebAssembly Text Format) that can be assembled to WASM binary.
//! The output includes a JS interop bridge for calling WASM functions from DOM code.
//!
//! Use case: compute-heavy state machines and pure data transformations run
//! faster in WASM than equivalent JavaScript, with predictable performance.

pub mod codegen;
pub mod interop;

pub use codegen::compile_to_wat;
