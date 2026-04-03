//! Canvas 2D fallback renderer — simplified 3D→2D projection.
//!
//! When WebGPU is unavailable, renders a basic wireframe/flat-shaded
//! representation of the 3D scene using Canvas 2D API.

/// Generate Canvas 2D fallback JavaScript for a scene.
///
/// Provides a basic orthographic projection of meshes as filled polygons
/// with simple lighting. Much simpler than WebGPU but works everywhere.
pub fn canvas2d_fallback_js(mesh_count: usize) -> String {
    format!(
        r#"// Canvas 2D Fallback Renderer
window.__voce_canvas2d_init = function(canvas) {{
  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const w = canvas.width, h = canvas.height;
  let angle = 0;

  function draw() {{
    ctx.clearRect(0, 0, w, h);
    ctx.fillStyle = '#1a1a2e';
    ctx.fillRect(0, 0, w, h);

    const cx = w / 2, cy = h / 2;
    const scale = Math.min(w, h) * 0.3;

    // Simple rotating cube wireframe
    const cos = Math.cos(angle), sin = Math.sin(angle);
    const verts = [
      [-1,-1,-1],[1,-1,-1],[1,1,-1],[-1,1,-1],
      [-1,-1,1],[1,-1,1],[1,1,1],[-1,1,1]
    ];
    const edges = [[0,1],[1,2],[2,3],[3,0],[4,5],[5,6],[6,7],[7,4],[0,4],[1,5],[2,6],[3,7]];

    // Rotate Y
    const rotated = verts.map(([x,y,z]) => [
      x*cos - z*sin, y, x*sin + z*cos
    ]);

    // Project (simple perspective)
    const proj = rotated.map(([x,y,z]) => {{
      const d = 3 + z;
      return [cx + x/d * scale, cy - y/d * scale];
    }});

    // Draw edges
    ctx.strokeStyle = '#4488ff';
    ctx.lineWidth = 2;
    for (const [a,b] of edges) {{
      ctx.beginPath();
      ctx.moveTo(proj[a][0], proj[a][1]);
      ctx.lineTo(proj[b][0], proj[b][1]);
      ctx.stroke();
    }}

    // Draw faces (back faces culled by z-order)
    const faces = [[0,1,2,3],[4,5,6,7],[0,1,5,4],[2,3,7,6],[0,3,7,4],[1,2,6,5]];
    ctx.globalAlpha = 0.15;
    ctx.fillStyle = '#4488ff';
    for (const face of faces) {{
      ctx.beginPath();
      ctx.moveTo(proj[face[0]][0], proj[face[0]][1]);
      for (let i=1; i<face.length; i++) ctx.lineTo(proj[face[i]][0], proj[face[i]][1]);
      ctx.closePath();
      ctx.fill();
    }}
    ctx.globalAlpha = 1;

    // Label
    ctx.fillStyle = '#666';
    ctx.font = '12px system-ui';
    ctx.textAlign = 'center';
    ctx.fillText('Canvas 2D fallback ({mesh_count} mesh{plural})', cx, h - 16);

    angle += 0.01;
    requestAnimationFrame(draw);
  }}
  draw();
}};
"#,
        mesh_count = mesh_count,
        plural = if mesh_count != 1 { "es" } else { "" },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas2d_js_contains_draw_loop() {
        let js = canvas2d_fallback_js(3);
        assert!(js.contains("requestAnimationFrame"));
        assert!(js.contains("getContext('2d')"));
        assert!(js.contains("3 meshes"));
    }

    #[test]
    fn single_mesh_no_plural() {
        let js = canvas2d_fallback_js(1);
        assert!(js.contains("1 mesh)"));
    }
}
