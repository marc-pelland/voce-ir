//! Graceful degradation — fallback chain for 3D content.
//!
//! WebGPU → Canvas 2D → static image → text description.
//! Each level provides a progressively simpler representation
//! of the same content, ensuring the page works everywhere.

pub mod canvas2d;
pub mod capability;

/// JavaScript that detects capabilities and selects the best renderer.
///
/// Injected into the compiled HTML. Runs immediately, before DOMContentLoaded.
/// Sets a data attribute on the container so CSS can adapt.
pub fn capability_detection_js() -> String {
    r#"// Voce IR Capability Detection
(function() {
  const container = document.querySelector('[data-voce-3d]');
  if (!container) return;

  async function detect() {
    // Try WebGPU
    if (navigator.gpu) {
      try {
        const adapter = await navigator.gpu.requestAdapter();
        if (adapter) {
          container.dataset.voceRenderer = 'webgpu';
          container.querySelector('.voce-fallback-canvas')?.remove();
          container.querySelector('.voce-fallback-static')?.remove();
          return;
        }
      } catch(e) {}
    }

    // Try Canvas 2D
    const canvas = container.querySelector('.voce-fallback-canvas');
    if (canvas && canvas.getContext('2d')) {
      container.dataset.voceRenderer = 'canvas2d';
      container.querySelector('.voce-webgpu-canvas')?.remove();
      container.querySelector('.voce-fallback-static')?.remove();
      // Initialize Canvas 2D fallback renderer
      if (window.__voce_canvas2d_init) window.__voce_canvas2d_init(canvas);
      return;
    }

    // Static image fallback (already visible via CSS)
    container.dataset.voceRenderer = 'static';
    container.querySelector('.voce-webgpu-canvas')?.remove();
    container.querySelector('.voce-fallback-canvas')?.remove();
  }

  detect();
})();
"#
    .to_string()
}

/// Generate the HTML structure for a 3D scene with all fallback layers.
///
/// Structure:
/// ```html
/// <div data-voce-3d>
///   <canvas class="voce-webgpu-canvas" ...></canvas>     <!-- WebGPU (preferred) -->
///   <canvas class="voce-fallback-canvas" ...></canvas>    <!-- Canvas 2D fallback -->
///   <div class="voce-fallback-static">                    <!-- Static fallback -->
///     <img src="..." alt="...">
///     <noscript><p>3D scene: ...</p></noscript>
///   </div>
/// </div>
/// ```
pub fn scene_fallback_html(
    width: u32,
    height: u32,
    scene_description: &str,
    static_image_url: Option<&str>,
) -> String {
    let img = static_image_url
        .map(|url| format!("<img src=\"{url}\" alt=\"{scene_description}\" style=\"width:100%;height:auto\">"))
        .unwrap_or_else(|| format!("<div style=\"background:#1a1a2e;color:#888;display:flex;align-items:center;justify-content:center;width:{width}px;height:{height}px;border-radius:8px\"><p>{scene_description}</p></div>"));

    format!(
        r#"<div data-voce-3d style="position:relative;width:{width}px;height:{height}px;max-width:100%">
  <canvas class="voce-webgpu-canvas" width="{width}" height="{height}" style="display:block;width:100%;height:100%"></canvas>
  <canvas class="voce-fallback-canvas" width="{width}" height="{height}" style="display:none;width:100%;height:100%"></canvas>
  <div class="voce-fallback-static" style="display:none">
    {img}
    <noscript><p>3D scene: {scene_description}</p></noscript>
  </div>
</div>"#
    )
}

/// CSS for showing/hiding fallback layers based on detected renderer.
pub fn fallback_css() -> String {
    r#"[data-voce-renderer="canvas2d"] .voce-fallback-canvas { display: block !important; }
[data-voce-renderer="canvas2d"] .voce-webgpu-canvas { display: none !important; }
[data-voce-renderer="static"] .voce-fallback-static { display: block !important; }
[data-voce-renderer="static"] .voce-webgpu-canvas { display: none !important; }
[data-voce-renderer="static"] .voce-fallback-canvas { display: none !important; }
"#
    .to_string()
}
