//! Style validation issue type and constructors.
//!
//! Provides [`StyleValidationIssue`], a description of a single style
//! validation problem with severity, affected field, and optional fix
//! suggestion.

use alloc::string::{String, ToString};

use super::ValidationSeverity;

/// Style validation issue with context and suggestions
#[derive(Debug, Clone)]
pub struct StyleValidationIssue {
    /// Issue severity level
    pub severity: ValidationSeverity,
    /// Human-readable issue description
    pub message: String,
    /// Style field that caused the issue
    pub field: String,
    /// Optional suggested fix or improvement
    pub suggestion: Option<String>,
}

impl StyleValidationIssue {
    /// Create new validation issue
    #[must_use]
    pub fn new(
        severity: ValidationSeverity,
        field: &str,
        message: &str,
        suggestion: Option<&str>,
    ) -> Self {
        Self {
            severity,
            message: message.to_string(),
            field: field.to_string(),
            suggestion: suggestion.map(ToString::to_string),
        }
    }

    /// Create error-level issue
    #[must_use]
    pub fn error(field: &str, message: &str) -> Self {
        Self::new(ValidationSeverity::Error, field, message, None)
    }

    /// Create warning-level issue
    #[must_use]
    pub fn warning(field: &str, message: &str) -> Self {
        Self::new(ValidationSeverity::Warning, field, message, None)
    }

    /// Create info-level issue with suggestion
    #[must_use]
    pub fn info_with_suggestion(field: &str, message: &str, suggestion: &str) -> Self {
        Self::new(ValidationSeverity::Info, field, message, Some(suggestion))
    }
}
