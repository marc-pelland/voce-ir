//! Compiler pipeline — orchestrates ingest, optimize, lower, and emit phases.

pub mod ingest;

use std::collections::HashMap;

use anyhow::Result;

use crate::compiler_ir::CompilerIr;
use crate::emit::html::HtmlOutput;

/// Options controlling compilation behavior.
#[derive(Debug, Default)]
pub struct CompileOptions {
    /// Whether to minify the HTML output.
    pub minify: bool,
    /// Whether to include source IR node IDs as data attributes (for debugging).
    pub debug_attrs: bool,
    /// Source image bytes keyed by src path. When provided, the image pipeline
    /// generates responsive WebP/JPEG variants and BlurHash placeholders.
    pub image_assets: HashMap<String, Vec<u8>>,
    /// Output directory for generated image variants (relative path used in HTML).
    pub assets_dir: String,
    /// Skip font processing (no @font-face, no preloads, no subsetting).
    pub skip_fonts: bool,
    /// Source font bytes keyed by font family name. When provided, the font pipeline
    /// generates subsetted WOFF2 files with content-hashed filenames.
    pub font_assets: HashMap<String, Vec<u8>>,
}

/// Result of compilation.
#[derive(Debug)]
pub struct CompileResult {
    /// The compiled HTML string.
    pub html: String,
    /// Output size in bytes.
    pub size_bytes: usize,
}

/// Run the full compilation pipeline: JSON IR → HTML.
pub fn compile(json: &str, options: &CompileOptions) -> Result<CompileResult> {
    // Phase 1: Ingest — parse JSON and build compiler IR
    let ir = ingest::ingest(json)?;

    // Phase 2: Optimize (placeholder — Sprint 12+)
    // optimize::optimize(&mut ir);

    // Phase 3: Lower — convert IR nodes to HTML elements
    let html_output = lower_to_html(&ir, options);

    // Phase 4: Emit — serialize to HTML string
    let html = html_output.to_string();
    let size_bytes = html.len();

    Ok(CompileResult { html, size_bytes })
}

/// Lower compiler IR to an HTML document.
fn lower_to_html(ir: &CompilerIr, options: &CompileOptions) -> HtmlOutput {
    crate::emit::html::emit(ir, options)
}
