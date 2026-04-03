//! Static deployment adapter — produces a self-contained `dist/` folder.
//!
//! The simplest adapter: copies HTML + assets into a flat directory
//! that can be served by any static file server.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use voce_adapter_core::{Adapter, Bundle, CompiledOutput, DeployConfig, DeployResult};

/// Static file deployment adapter.
pub struct StaticAdapter {
    output_dir: PathBuf,
}

impl StaticAdapter {
    /// Create a new static adapter targeting the given output directory.
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }
}

impl Default for StaticAdapter {
    fn default() -> Self {
        Self::new(PathBuf::from("dist"))
    }
}

impl Adapter for StaticAdapter {
    fn name(&self) -> &str {
        "static"
    }

    fn prepare(&self, compiled: &CompiledOutput, _config: &DeployConfig) -> Result<Bundle> {
        let mut files: HashMap<PathBuf, Vec<u8>> = HashMap::new();

        // Main HTML file
        files.insert(
            PathBuf::from("index.html"),
            compiled.html.as_bytes().to_vec(),
        );

        // Assets in assets/ subdirectory
        for (name, data) in &compiled.assets {
            files.insert(PathBuf::from(format!("assets/{name}")), data.clone());
        }

        let asset_count = compiled.assets.len();
        let total_size: usize = files.values().map(|v| v.len()).sum();

        let summary = format!(
            "Static bundle: index.html + {asset_count} assets ({} KB total)",
            total_size / 1024
        );

        Ok(Bundle {
            output_dir: self.output_dir.clone(),
            files,
            summary,
        })
    }

    fn deploy(&self, bundle: &Bundle, _config: &DeployConfig) -> Result<DeployResult> {
        bundle.write_to_disk()?;

        Ok(DeployResult {
            url: None,
            deployment_id: None,
            message: format!(
                "Static files written to {}/ ({} files, {} KB)",
                bundle.output_dir.display(),
                bundle.files.len(),
                bundle.total_size() / 1024
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voce_adapter_core::ProjectMeta;

    fn sample_output() -> CompiledOutput {
        let mut assets = HashMap::new();
        assets.insert("hero-640w.webp".to_string(), vec![0u8; 512]);
        CompiledOutput {
            html: "<html><body>Hello</body></html>".to_string(),
            assets,
            actions: vec![],
            meta: ProjectMeta {
                name: "test-site".to_string(),
                ..Default::default()
            },
        }
    }

    #[test]
    fn prepare_creates_index_and_assets() {
        let adapter = StaticAdapter::default();
        let bundle = adapter
            .prepare(&sample_output(), &DeployConfig::default())
            .unwrap();
        assert!(bundle.files.contains_key(&PathBuf::from("index.html")));
        assert!(bundle
            .files
            .contains_key(&PathBuf::from("assets/hero-640w.webp")));
    }

    #[test]
    fn summary_shows_counts() {
        let adapter = StaticAdapter::default();
        let bundle = adapter
            .prepare(&sample_output(), &DeployConfig::default())
            .unwrap();
        assert!(bundle.summary.contains("1 assets"));
    }
}
