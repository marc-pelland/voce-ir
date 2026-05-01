//! Project-level validator configuration loaded from `.voce/validator.toml`.
//!
//! Today's only setting is severity escalation — projects can promote a
//! warning code to an error so their CI fails on issues the default schema
//! considers advisory. Future settings (rule disable lists, custom hint
//! overrides) live here.
//!
//! Format:
//!
//! ```toml
//! [severity]
//! SEO007 = "error"   # promote warning -> error
//! A11Y005 = "error"
//! # SEC001 = "warning"  # could also demote, though we don't recommend it
//! ```
//!
//! Config is searched in the IR file's parent directory and each ancestor up
//! to filesystem root, the first found wins. Missing config is a no-op.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::errors::Severity;

/// Loaded validator config. Default is empty (no overrides).
#[derive(Debug, Default, Clone)]
pub struct ValidatorConfig {
    /// Map of diagnostic code → overridden severity. Applied to every
    /// diagnostic the engine emits whose code matches.
    pub severity_overrides: HashMap<String, Severity>,
}

#[derive(Debug, Deserialize, Default)]
struct RawConfig {
    #[serde(default)]
    severity: HashMap<String, String>,
}

impl ValidatorConfig {
    /// Walk up from `start_dir` looking for a `.voce/validator.toml`. Returns
    /// the first one found, or `Default::default()` if none exists or parsing
    /// fails. Parsing errors are logged to stderr but never fatal — config is
    /// optional.
    pub fn load_from_dir(start_dir: &Path) -> Self {
        let mut current: Option<&Path> = Some(start_dir);
        while let Some(dir) = current {
            let candidate = dir.join(".voce").join("validator.toml");
            if candidate.exists() {
                match std::fs::read_to_string(&candidate)
                    .and_then(|s| toml::from_str::<RawConfig>(&s).map_err(io_error))
                {
                    Ok(raw) => return Self::from_raw(raw, &candidate),
                    Err(e) => {
                        eprintln!(
                            "voce: warning — failed to read {}: {e}; ignoring",
                            candidate.display()
                        );
                        return Self::default();
                    }
                }
            }
            current = dir.parent();
        }
        Self::default()
    }

    fn from_raw(raw: RawConfig, source: &Path) -> Self {
        let mut overrides = HashMap::new();
        for (code, level) in raw.severity {
            match level.to_ascii_lowercase().as_str() {
                "error" => {
                    overrides.insert(code, Severity::Error);
                }
                "warning" => {
                    overrides.insert(code, Severity::Warning);
                }
                other => {
                    eprintln!(
                        "voce: warning — {} sets {code} to unknown severity {other:?}; \
                         expected \"error\" or \"warning\"; ignoring entry",
                        source.display()
                    );
                }
            }
        }
        Self {
            severity_overrides: overrides,
        }
    }
}

fn io_error(e: toml::de::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_severity_overrides() {
        let raw: RawConfig = toml::from_str(
            r#"
            [severity]
            SEO007 = "error"
            A11Y005 = "warning"
        "#,
        )
        .unwrap();
        let cfg = ValidatorConfig::from_raw(raw, Path::new("/tmp/test.toml"));
        assert_eq!(cfg.severity_overrides.get("SEO007"), Some(&Severity::Error));
        assert_eq!(
            cfg.severity_overrides.get("A11Y005"),
            Some(&Severity::Warning)
        );
    }

    #[test]
    fn unknown_severity_is_ignored() {
        let raw: RawConfig = toml::from_str(
            r#"
            [severity]
            STR002 = "fatal"
        "#,
        )
        .unwrap();
        let cfg = ValidatorConfig::from_raw(raw, Path::new("/tmp/test.toml"));
        assert!(cfg.severity_overrides.is_empty());
    }

    #[test]
    fn missing_config_is_default() {
        let cfg = ValidatorConfig::load_from_dir(Path::new("/this/path/does/not/exist"));
        assert!(cfg.severity_overrides.is_empty());
    }
}
