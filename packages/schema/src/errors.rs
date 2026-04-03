//! Unified error taxonomy for the Voce IR pipeline.
//!
//! Every error includes: an error code, human-readable message, source location,
//! and an actionable suggestion for fixing the problem.

use serde::{Deserialize, Serialize};

/// Top-level error type for the Voce IR pipeline.
#[derive(Debug, thiserror::Error)]
pub enum VoceError {
    /// Schema-level errors (parsing, format, version mismatch).
    #[error("[{code}] Schema error: {message}")]
    Schema {
        code: ErrorCode,
        message: String,
        suggestion: String,
    },

    /// Validation errors (rule violations found by validation passes).
    #[error("[{code}] Validation error at {node_path}: {message}")]
    Validation {
        code: ErrorCode,
        message: String,
        node_path: String,
        suggestion: String,
        severity: ErrorSeverity,
    },

    /// Compilation errors (failures during IR → output conversion).
    #[error("[{code}] Compilation error at {node_path}: {message}")]
    Compilation {
        code: ErrorCode,
        message: String,
        node_path: String,
        suggestion: String,
    },

    /// Deployment errors (failures during bundle/upload).
    #[error("[{code}] Deployment error: {message}")]
    Deployment {
        code: ErrorCode,
        message: String,
        suggestion: String,
    },

    /// Pipeline errors (orchestration failures).
    #[error("[{code}] Pipeline error: {message}")]
    Pipeline {
        code: ErrorCode,
        message: String,
        suggestion: String,
    },

    /// AI bridge errors (generation, API, timeout).
    #[error("[{code}] AI bridge error: {message}")]
    AiBridge {
        code: ErrorCode,
        message: String,
        suggestion: String,
    },
}

/// Typed error codes for every Voce error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorCode(pub String);

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ErrorCode {
    // Schema errors
    pub fn schema_parse() -> Self { Self("S001".to_string()) }
    pub fn schema_version() -> Self { Self("S002".to_string()) }
    pub fn schema_missing_root() -> Self { Self("S003".to_string()) }

    // Compilation errors
    pub fn compile_node_failed() -> Self { Self("C001".to_string()) }
    pub fn compile_timeout() -> Self { Self("C002".to_string()) }
    pub fn compile_unsupported_node() -> Self { Self("C003".to_string()) }
    pub fn compile_asset_failed() -> Self { Self("C004".to_string()) }

    // Deployment errors
    pub fn deploy_adapter_not_found() -> Self { Self("D001".to_string()) }
    pub fn deploy_bundle_failed() -> Self { Self("D002".to_string()) }
    pub fn deploy_upload_failed() -> Self { Self("D003".to_string()) }
    pub fn deploy_config_invalid() -> Self { Self("D004".to_string()) }

    // Pipeline errors
    pub fn pipeline_timeout() -> Self { Self("P001".to_string()) }
    pub fn pipeline_interrupted() -> Self { Self("P002".to_string()) }

    // AI bridge errors
    pub fn ai_api_error() -> Self { Self("A001".to_string()) }
    pub fn ai_rate_limited() -> Self { Self("A002".to_string()) }
    pub fn ai_timeout() -> Self { Self("A003".to_string()) }
    pub fn ai_incomplete_output() -> Self { Self("A004".to_string()) }
    pub fn ai_key_invalid() -> Self { Self("A005".to_string()) }
}

/// Error severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Blocks compilation — must be fixed.
    Error,
    /// Emits output but flags a potential issue.
    Warning,
    /// Informational only.
    Info,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Error => write!(f, "error"),
            ErrorSeverity::Warning => write!(f, "warning"),
            ErrorSeverity::Info => write!(f, "info"),
        }
    }
}

/// A structured error report for JSON output.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorReport {
    pub code: String,
    pub severity: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_path: Option<String>,
    pub suggestion: String,
}

impl From<&VoceError> for ErrorReport {
    fn from(err: &VoceError) -> Self {
        match err {
            VoceError::Schema { code, message, suggestion } => ErrorReport {
                code: code.0.clone(),
                severity: "error".to_string(),
                message: message.clone(),
                node_path: None,
                suggestion: suggestion.clone(),
            },
            VoceError::Validation { code, message, node_path, suggestion, severity } => ErrorReport {
                code: code.0.clone(),
                severity: severity.to_string(),
                message: message.clone(),
                node_path: Some(node_path.clone()),
                suggestion: suggestion.clone(),
            },
            VoceError::Compilation { code, message, node_path, suggestion } => ErrorReport {
                code: code.0.clone(),
                severity: "error".to_string(),
                message: message.clone(),
                node_path: Some(node_path.clone()),
                suggestion: suggestion.clone(),
            },
            VoceError::Deployment { code, message, suggestion } => ErrorReport {
                code: code.0.clone(),
                severity: "error".to_string(),
                message: message.clone(),
                node_path: None,
                suggestion: suggestion.clone(),
            },
            VoceError::Pipeline { code, message, suggestion } => ErrorReport {
                code: code.0.clone(),
                severity: "error".to_string(),
                message: message.clone(),
                node_path: None,
                suggestion: suggestion.clone(),
            },
            VoceError::AiBridge { code, message, suggestion } => ErrorReport {
                code: code.0.clone(),
                severity: "error".to_string(),
                message: message.clone(),
                node_path: None,
                suggestion: suggestion.clone(),
            },
        }
    }
}

/// CLI exit codes.
pub mod exit_codes {
    pub const SUCCESS: i32 = 0;
    pub const VALIDATION_ERROR: i32 = 1;
    pub const COMPILATION_ERROR: i32 = 2;
    pub const DEPLOYMENT_ERROR: i32 = 3;
    pub const AI_BRIDGE_ERROR: i32 = 4;
    pub const INTERNAL_ERROR: i32 = 5;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_display() {
        let code = ErrorCode::schema_parse();
        assert_eq!(code.to_string(), "S001");
    }

    #[test]
    fn voce_error_display_includes_code() {
        let err = VoceError::Schema {
            code: ErrorCode::schema_parse(),
            message: "Invalid JSON".to_string(),
            suggestion: "Check JSON syntax".to_string(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("S001"));
        assert!(msg.contains("Invalid JSON"));
    }

    #[test]
    fn error_report_from_validation_error() {
        let err = VoceError::Validation {
            code: ErrorCode("STR001".to_string()),
            message: "Missing root".to_string(),
            node_path: "$.root".to_string(),
            suggestion: "Add a root ViewRoot node".to_string(),
            severity: ErrorSeverity::Error,
        };
        let report = ErrorReport::from(&err);
        assert_eq!(report.code, "STR001");
        assert_eq!(report.severity, "error");
        assert!(report.node_path.is_some());
    }

    #[test]
    fn error_report_serializes_to_json() {
        let report = ErrorReport {
            code: "C001".to_string(),
            severity: "error".to_string(),
            message: "Node failed to compile".to_string(),
            node_path: Some("root.children[0]".to_string()),
            suggestion: "Check node structure".to_string(),
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("C001"));
        assert!(json.contains("root.children[0]"));
    }

    #[test]
    fn exit_codes_are_distinct() {
        let codes = [
            exit_codes::SUCCESS,
            exit_codes::VALIDATION_ERROR,
            exit_codes::COMPILATION_ERROR,
            exit_codes::DEPLOYMENT_ERROR,
            exit_codes::AI_BRIDGE_ERROR,
            exit_codes::INTERNAL_ERROR,
        ];
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), codes.len());
    }
}
