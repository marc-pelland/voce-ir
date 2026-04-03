//! Output bundling — merges DOM, WebGPU, and WASM into a single HTML file.

/// Bundle DOM HTML with optional WebGPU canvas and WASM bridge.
///
/// The DOM output is the base. WebGPU canvas and WASM bridge are
/// injected before `</body>` if present.
pub fn bundle(
    dom_html: &str,
    webgpu_js: Option<&str>,
    wasm_bridge: Option<&str>,
) -> String {
    let mut html = dom_html.to_string();

    // Inject WebGPU canvas + init before </body>
    if let Some(gpu_js) = webgpu_js {
        // Extract just the canvas and script from the WebGPU output
        // (skip the duplicate DOCTYPE/head since DOM already has that)
        if let Some(canvas_start) = gpu_js.find("<canvas") {
            if let Some(script_end) = gpu_js.rfind("</script>") {
                let gpu_fragment = &gpu_js[canvas_start..script_end + "</script>".len()];
                html = html.replace(
                    "</body>",
                    &format!("<!-- WebGPU Scene -->\n{gpu_fragment}\n</body>"),
                );
            }
        }
    }

    // Inject WASM bridge before </body>
    if let Some(bridge) = wasm_bridge {
        if !bridge.is_empty() {
            html = html.replace(
                "</body>",
                &format!("<script>\n// WASM Interop Bridge\n{bridge}</script>\n</body>"),
            );
        }
    }

    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dom_only_passes_through() {
        let dom = "<html><body><p>Hello</p></body></html>";
        let result = bundle(dom, None, None);
        assert_eq!(result, dom);
    }

    #[test]
    fn wasm_bridge_injected_before_body_close() {
        let dom = "<html><body><p>Hi</p></body></html>";
        let result = bundle(dom, None, Some("function wasm_init(){}"));
        assert!(result.contains("WASM Interop Bridge"));
        assert!(result.contains("wasm_init"));
        // Verify it's before </body>
        let bridge_pos = result.find("WASM").unwrap();
        let body_pos = result.find("</body>").unwrap();
        assert!(bridge_pos < body_pos);
    }
}
