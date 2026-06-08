//! `CoreError` categorization and suggestion helpers.
//!
//! Maps each `CoreError` variant onto an [`ErrorCategory`] and provides
//! actionable suggestions for common error scenarios.

use super::super::CoreError;
use super::ErrorCategory;

impl CoreError {
    /// Get error category for filtering/grouping
    ///
    /// Returns the category that best describes the type of error,
    /// useful for organizing errors in user interfaces or logs.
    #[must_use]
    pub const fn category(&self) -> ErrorCategory {
        match self {
            Self::Parse(_) | Self::Tokenization(_) => ErrorCategory::Parsing,
            Self::Analysis(_) => ErrorCategory::Analysis,
            Self::Plugin(_) => ErrorCategory::Plugin,
            Self::InvalidColor(_) | Self::InvalidNumeric(_) | Self::InvalidTime(_) => {
                ErrorCategory::Format
            }
            Self::Utf8Error { .. } => ErrorCategory::Encoding,
            Self::Io(_) => ErrorCategory::Io,
            Self::OutOfMemory(_) | Self::ResourceLimitExceeded { .. } => ErrorCategory::Resource,
            Self::Config(_) => ErrorCategory::Configuration,
            Self::Validation(_) => ErrorCategory::Validation,
            Self::FeatureNotSupported { .. } | Self::VersionIncompatible { .. } => {
                ErrorCategory::Compatibility
            }
            Self::SecurityViolation(_) => ErrorCategory::Security,
            Self::Internal(_) => ErrorCategory::Internal,
        }
    }

    /// Get suggested action for this error
    ///
    /// Provides actionable advice for resolving common error scenarios.
    /// Returns `None` for errors that don't have standard solutions.
    #[must_use]
    pub const fn suggestion(&self) -> Option<&'static str> {
        match self {
            Self::InvalidColor(_) => Some("Use format like '&H00FF00FF&' for colors"),
            Self::InvalidTime(_) => Some("Use format like '0:01:30.50' for times"),
            Self::InvalidNumeric(_) => Some("Check numeric format and range"),
            Self::FeatureNotSupported { .. } => Some("Enable required feature in Cargo.toml"),
            Self::OutOfMemory(_) => Some("Reduce input size or enable 'arena' feature"),
            Self::ResourceLimitExceeded { .. } => {
                Some("Reduce input complexity or increase limits")
            }
            Self::SecurityViolation(_) => Some("Review script content for security issues"),
            Self::Internal(_) => Some("Please report this bug to the maintainers"),
            Self::Utf8Error { .. } => Some("Check file encoding - ASS files should be UTF-8"),
            Self::Config(_) => Some("Review configuration settings and feature flags"),
            _ => None,
        }
    }
}
