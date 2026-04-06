//! WebGPU compilation pipeline — ingests IR, builds scene graph, emits HTML.

pub mod ingest;

use anyhow::Result;

use crate::emit::webgpu_html;
use crate::scene::Scene;

/// Options for WebGPU compilation.
#[derive(Debug, Default)]
pub struct WebGpuCompileOptions {
    /// Canvas width in pixels.
    pub width: u32,
    /// Canvas height in pixels.
    pub height: u32,
    /// Whether to enable orbit camera controls.
    pub orbit_controls: bool,
}

/// Result of WebGPU compilation.
#[derive(Debug)]
pub struct WebGpuCompileResult {
    /// The compiled HTML string containing WebGPU initialization.
    pub html: String,
    /// Output size in bytes.
    pub size_bytes: usize,
}

/// Compile IR JSON containing Scene3D nodes to a WebGPU HTML output.
pub fn compile_webgpu(json: &str, options: &WebGpuCompileOptions) -> Result<WebGpuCompileResult> {
    // Phase 1: Ingest — extract Scene3D data from IR
    let scene: Scene = ingest::ingest_scene(json)?;

    // Phase 2: Emit — generate HTML with WebGPU
    let opts = webgpu_html::EmitOptions {
        width: if options.width > 0 {
            options.width
        } else {
            800
        },
        height: if options.height > 0 {
            options.height
        } else {
            600
        },
        orbit_controls: options.orbit_controls,
    };

    let html = webgpu_html::emit(&scene, &opts);
    let size_bytes = html.len();

    Ok(WebGpuCompileResult { html, size_bytes })
}
