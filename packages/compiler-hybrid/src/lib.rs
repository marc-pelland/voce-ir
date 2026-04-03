//! Voce IR Hybrid Compiler — orchestrates DOM, WebGPU, and WASM compilers.
//!
//! Analyzes each IR node to determine the optimal compile target:
//! - Layout nodes (Container, Surface, TextNode) → DOM
//! - 3D nodes (Scene3D, MeshNode, ParticleSystem) → WebGPU
//! - Compute-heavy nodes (complex StateMachine, ComputeNode) → WASM
//!
//! Produces a unified HTML output with embedded DOM + WebGPU canvas + WASM module.

pub mod analysis;
pub mod bundle;
pub mod device;
pub mod fallback;

use anyhow::Result;

/// Hybrid compilation options.
#[derive(Debug, Clone)]
pub struct HybridCompileOptions {
    /// Device profile for target selection.
    pub device: device::DeviceProfile,
    /// Force a specific target for all nodes (overrides analysis).
    pub force_target: Option<CompileTarget>,
}

impl Default for HybridCompileOptions {
    fn default() -> Self {
        Self {
            device: device::DeviceProfile::desktop(),
            force_target: None,
        }
    }
}

/// Possible compilation targets for a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileTarget {
    Dom,
    WebGpu,
    Wasm,
}

/// Result of hybrid compilation.
#[derive(Debug)]
pub struct HybridCompileResult {
    /// The unified HTML output.
    pub html: String,
    /// Size in bytes.
    pub size_bytes: usize,
    /// Per-target breakdown.
    pub target_summary: TargetSummary,
}

/// Summary of what was compiled to each target.
#[derive(Debug, Default)]
pub struct TargetSummary {
    pub dom_nodes: usize,
    pub webgpu_nodes: usize,
    pub wasm_functions: usize,
}

/// Compile IR with automatic per-component target selection.
pub fn compile_hybrid(json: &str, options: &HybridCompileOptions) -> Result<HybridCompileResult> {
    // Phase 1: Analyze each node's optimal target
    let node_targets = analysis::analyze_targets(json, &options.device, options.force_target)?;

    // Phase 2: Compile DOM nodes
    let dom_result = voce_compiler_dom::compile(
        json,
        &voce_compiler_dom::CompileOptions::default(),
    )?;

    // Phase 3: Check for WebGPU/WASM nodes
    let has_webgpu = node_targets.iter().any(|t| t.target == CompileTarget::WebGpu);
    let has_wasm = node_targets.iter().any(|t| t.target == CompileTarget::Wasm);

    let wasm_result = if has_wasm {
        Some(voce_compiler_wasm::compile_to_wat(json)?)
    } else {
        None
    };

    let webgpu_result = if has_webgpu {
        Some(voce_compiler_webgpu::compile_webgpu(
            json,
            &voce_compiler_webgpu::WebGpuCompileOptions::default(),
        )?)
    } else {
        None
    };

    // Phase 4: Bundle into unified output
    let html = bundle::bundle(
        &dom_result.html,
        webgpu_result.as_ref().map(|r| r.html.as_str()),
        wasm_result.as_ref().map(|r| r.js_bridge.as_str()),
    );

    let size_bytes = html.len();

    let target_summary = TargetSummary {
        dom_nodes: node_targets.iter().filter(|t| t.target == CompileTarget::Dom).count(),
        webgpu_nodes: node_targets.iter().filter(|t| t.target == CompileTarget::WebGpu).count(),
        wasm_functions: wasm_result.as_ref().map_or(0, |r| r.function_count),
    };

    Ok(HybridCompileResult {
        html,
        size_bytes,
        target_summary,
    })
}
