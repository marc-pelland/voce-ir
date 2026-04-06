//! Capability reporting — tracks what features are active on the device.

/// A capability report for the current device.
#[derive(Debug, Clone)]
pub struct CapabilityReport {
    pub webgpu_available: bool,
    pub canvas2d_available: bool,
    pub wasm_available: bool,
    pub active_renderer: String,
    pub features: Vec<String>,
}

/// Generate JavaScript that produces a capability report object at runtime.
pub fn capability_report_js() -> String {
    r#"// Voce IR Capability Report
window.__voce_capabilities = {
  webgpu: !!navigator.gpu,
  canvas2d: true,
  wasm: typeof WebAssembly !== 'undefined',
  renderer: document.querySelector('[data-voce-renderer]')?.dataset.voceRenderer || 'dom',
  report: function() {
    const c = this;
    return `Voce IR Capabilities:
  WebGPU: ${c.webgpu ? 'available' : 'not available'}
  Canvas 2D: available
  WASM: ${c.wasm ? 'available' : 'not available'}
  Active renderer: ${c.renderer}`;
  }
};
"#
    .to_string()
}
