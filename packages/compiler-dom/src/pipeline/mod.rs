//! Compiler pipeline — orchestrates ingest, optimize, lower, and emit phases.

pub mod ingest;

use std::collections::HashMap;

use anyhow::Result;

use crate::compiler_ir::CompilerIr;
use crate::emit::html::HtmlOutput;
use crate::perf::{PerfCollector, PerfReport};

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
    /// Collect a PerfReport during compilation (S71 Day 2). Off by default —
    /// the bookkeeping is cheap (a few `Instant::now()` calls) but tests
    /// shouldn't depend on absolute timings, so leaving it opt-in keeps
    /// release builds free of surprise overhead.
    pub collect_perf_report: bool,
}

/// Result of compilation.
#[derive(Debug)]
pub struct CompileResult {
    /// The compiled HTML string.
    pub html: String,
    /// Output size in bytes.
    pub size_bytes: usize,
    /// Per-phase timings + metadata, populated when
    /// `CompileOptions.collect_perf_report` is true.
    pub perf_report: Option<PerfReport>,
}

/// Run the full compilation pipeline: JSON IR → HTML.
pub fn compile(json: &str, options: &CompileOptions) -> Result<CompileResult> {
    let mut perf = options.collect_perf_report.then(PerfCollector::start);

    // Phase 1: Ingest — parse JSON and build compiler IR
    if let Some(p) = perf.as_mut() {
        p.start_phase("ingest");
    }
    let ir = ingest::ingest(json)?;

    // Phase 2: Optimize (placeholder — Sprint 12+)
    // optimize::optimize(&mut ir);

    // Phase 3: Emit — lower IR nodes + serialize to HTML string
    if let Some(p) = perf.as_mut() {
        p.start_phase("emit");
    }
    let html_output = lower_to_html(&ir, options);
    let mut html = html_output.to_string();

    // Phase 4: Minify if requested
    if options.minify {
        if let Some(p) = perf.as_mut() {
            p.start_phase("minify");
        }
        html = minify_html(&html);
    }

    let size_bytes = html.len();
    let node_count = ir.nodes.len();
    let perf_report = perf.map(|p| p.finish(json.len(), size_bytes, node_count));
    Ok(CompileResult {
        html,
        size_bytes,
        perf_report,
    })
}

/// Lower compiler IR to an HTML document.
fn lower_to_html(ir: &CompilerIr, options: &CompileOptions) -> HtmlOutput {
    crate::emit::html::emit(ir, options)
}

/// Minify HTML output — collapse whitespace, remove unnecessary formatting.
fn minify_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_pre = false;
    let mut prev_was_space = false;

    for line in html.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Track <pre>/<script> blocks where whitespace matters
        if trimmed.contains("<pre") || trimmed.contains("<script") {
            in_pre = true;
        }
        if trimmed.contains("</pre>") || trimmed.contains("</script>") {
            in_pre = false;
        }

        if in_pre {
            result.push_str(line);
            result.push('\n');
            prev_was_space = false;
        } else {
            // Collapse leading whitespace
            if !result.is_empty() && !prev_was_space {
                result.push('\n');
            }
            result.push_str(trimmed);
            prev_was_space = trimmed.is_empty();
        }
    }

    result
}
