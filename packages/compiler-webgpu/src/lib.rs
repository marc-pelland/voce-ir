//! Voce IR WebGPU Compiler — compiles 3D scene IR to WebGPU render pipelines.
//!
//! Supports: Scene3D with camera/lighting, MeshNode with PBR materials,
//! ShaderNode → WGSL transpilation, ParticleSystem → GPU compute shaders.

pub mod emit;
pub mod particles;
pub mod pipeline;
pub mod scene;
pub mod shaders;

pub use pipeline::{compile_webgpu, WebGpuCompileOptions, WebGpuCompileResult};
