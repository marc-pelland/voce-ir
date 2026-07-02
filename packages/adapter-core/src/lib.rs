//! Shared deployment adapter trait and types for Voce IR.
//!
//! All deployment adapters (static, Vercel, Cloudflare, Netlify)
//! implement the [`Adapter`] trait to produce platform-specific bundles.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// The compiled output from the Voce compiler, ready for deployment.
#[derive(Debug)]
pub struct CompiledOutput {
    /// The main HTML file content.
    pub html: String,
    /// Asset files: filename → bytes (images, fonts, etc.).
    pub assets: HashMap<String, Vec<u8>>,
    /// Server-side action handlers extracted from ActionNodes.
    pub actions: Vec<ActionHandler>,
    /// Project metadata.
    pub meta: ProjectMeta,
}

/// A server-side action handler derived from an ActionNode.
#[derive(Debug, Clone)]
pub struct ActionHandler {
    /// Route path (e.g., "/api/contact").
    pub route: String,
    /// HTTP method (e.g., "POST").
    pub method: String,
    /// Node ID from the IR.
    pub node_id: String,
    /// Handler body — JS/TS code stub for the serverless function.
    pub handler_code: String,
}

/// Project metadata for deployment configuration.
#[derive(Debug, Clone, Default)]
pub struct ProjectMeta {
    /// Project name (used for deployment naming).
    pub name: String,
    /// Custom domain, if configured.
    pub domain: Option<String>,
    /// Environment variables to set.
    pub env_vars: HashMap<String, String>,
}

/// A deployment bundle — the files ready to upload/deploy.
#[derive(Debug)]
pub struct Bundle {
    /// Output directory path.
    pub output_dir: PathBuf,
    /// All files in the bundle: relative path → content bytes.
    pub files: HashMap<PathBuf, Vec<u8>>,
    /// Human-readable summary of what was generated.
    pub summary: String,
}

impl Bundle {
    /// Write all bundle files to the output directory.
    pub fn write_to_disk(&self) -> Result<()> {
        for (rel_path, content) in &self.files {
            Self::ensure_safe_relative(rel_path)?;
            let full_path = self.output_dir.join(rel_path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full_path, content)?;
        }
        Ok(())
    }

    /// Reject a bundle path that is absolute or escapes the output directory.
    /// Asset names originate in untrusted (AI-generated) IR, so a `..`,
    /// absolute, or drive-prefixed path must never be joined onto the output
    /// dir; otherwise a key like `../../../.ssh/authorized_keys` writes
    /// outside it.
    fn ensure_safe_relative(rel_path: &Path) -> Result<()> {
        use std::path::Component;
        for component in rel_path.components() {
            match component {
                Component::Normal(_) | Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    anyhow::bail!("unsafe bundle path: {}", rel_path.display());
                }
            }
        }
        Ok(())
    }

    /// Total size of all files in bytes.
    pub fn total_size(&self) -> usize {
        self.files.values().map(|v| v.len()).sum()
    }
}

/// Result of a deployment operation.
#[derive(Debug)]
pub struct DeployResult {
    /// URL where the site is live (if available).
    pub url: Option<String>,
    /// Platform-specific deployment ID.
    pub deployment_id: Option<String>,
    /// Human-readable status message.
    pub message: String,
}

/// Deployment configuration from `.voce/config.toml`.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DeployConfig {
    /// Default adapter name.
    #[serde(default)]
    pub adapter: String,
    /// Custom domain.
    pub domain: Option<String>,
    /// Environment variables.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Adapter-specific settings.
    #[serde(default)]
    pub settings: HashMap<String, String>,
}

/// Load deployment config from `.voce/config.toml`.
pub fn load_config(project_dir: &Path) -> Result<DeployConfig> {
    let config_path = project_dir.join(".voce/config.toml");
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: DeployConfig = toml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(DeployConfig::default())
    }
}

/// The trait all deployment adapters must implement.
pub trait Adapter {
    /// Human-readable adapter name (e.g., "static", "vercel").
    fn name(&self) -> &str;

    /// Prepare a deployment bundle from compiled output.
    fn prepare(&self, compiled: &CompiledOutput, config: &DeployConfig) -> Result<Bundle>;

    /// Deploy the bundle to the target platform.
    /// Returns `Err` if the platform CLI is not available or deployment fails.
    fn deploy(&self, bundle: &Bundle, config: &DeployConfig) -> Result<DeployResult>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_total_size() {
        let mut files = HashMap::new();
        files.insert(PathBuf::from("index.html"), vec![0u8; 100]);
        files.insert(PathBuf::from("style.css"), vec![0u8; 50]);
        let bundle = Bundle {
            output_dir: PathBuf::from("dist"),
            files,
            summary: "test".to_string(),
        };
        assert_eq!(bundle.total_size(), 150);
    }

    #[test]
    fn default_config() {
        let config = DeployConfig::default();
        assert!(config.adapter.is_empty());
        assert!(config.domain.is_none());
    }

    #[test]
    fn write_to_disk_rejects_path_traversal() {
        let tmp = std::env::temp_dir().join("voce-adapter-core-test-out");
        let mut files = HashMap::new();
        files.insert(PathBuf::from("../../../etc/voce-pwned"), b"x".to_vec());
        let bundle = Bundle {
            output_dir: tmp,
            files,
            summary: "t".into(),
        };
        let err = bundle.write_to_disk().unwrap_err();
        assert!(err.to_string().contains("unsafe bundle path"));
    }

    #[test]
    fn write_to_disk_rejects_absolute_path() {
        let mut files = HashMap::new();
        files.insert(PathBuf::from("/etc/voce-pwned"), b"x".to_vec());
        let bundle = Bundle {
            output_dir: PathBuf::from("dist"),
            files,
            summary: "t".into(),
        };
        assert!(bundle.write_to_disk().is_err());
    }

    #[test]
    fn write_to_disk_accepts_safe_nested_path() {
        let tmp = std::env::temp_dir().join("voce-adapter-core-ok-out");
        let _ = std::fs::remove_dir_all(&tmp);
        let mut files = HashMap::new();
        files.insert(PathBuf::from("assets/img/logo.png"), b"x".to_vec());
        let bundle = Bundle {
            output_dir: tmp.clone(),
            files,
            summary: "t".into(),
        };
        bundle.write_to_disk().unwrap();
        assert!(tmp.join("assets/img/logo.png").exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
