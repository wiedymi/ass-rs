//! Core error type for ASS-RS operations
//!
//! Provides the main `CoreError` enum that wraps all error types from different
//! modules in the crate. Designed for easy error propagation and conversion.
//!
//! # Error Philosophy
//!
//! - Use `thiserror` for structured error handling (no `anyhow` bloat)
//! - Provide detailed context for debugging and user feedback
//! - Support error chains with source information
//! - Include suggestions for common error scenarios
//! - Maintain zero-cost error handling where possible

use alloc::{format, string::String};
use core::fmt;

#[cfg(feature = "std")]
use thiserror::Error;

/// Main error type for ASS-RS core operations
///
/// Wraps all error types from different modules to provide a unified
/// error handling interface. Can be converted from module-specific errors.
#[cfg_attr(feature = "std", derive(Error))]
#[derive(Debug, Clone, PartialEq)]
pub enum CoreError {
    /// Parsing errors from parser module
    Parse(crate::parser::ParseError),

    /// Tokenization errors
    Tokenization(String),

    /// Analysis errors
    Analysis(String),

    /// Plugin system errors
    Plugin(String),

    /// Color format parsing errors
    InvalidColor(String),

    /// Numeric value parsing errors
    InvalidNumeric(String),

    /// Time format parsing errors
    InvalidTime(String),

    /// UTF-8 encoding errors
    Utf8Error { position: usize, message: String },

    /// File I/O errors
    Io(String),

    /// Memory allocation errors
    OutOfMemory(String),

    /// Configuration errors
    Config(String),

    /// Validation errors
    Validation(String),

    /// Feature not supported in current configuration
    FeatureNotSupported {
        feature: String,
        required_feature: String,
    },

    /// Version compatibility errors
    VersionIncompatible { message: String },

    /// Resource limit exceeded
    ResourceLimitExceeded {
        resource: String,
        current: usize,
        limit: usize,
    },

    /// Security policy violation
    SecurityViolation(String),

    /// Internal consistency error (should not happen)
    Internal(String),
}

impl CoreError {
    /// Create parse error from message
    pub fn parse<T: fmt::Display>(message: T) -> Self {
        Self::Parse(crate::parser::ParseError::IoError {
            message: format!("{message}"),
        })
    }

    /// Create internal error (indicates a bug)
    pub fn internal<T: fmt::Display>(message: T) -> Self {
        Self::Internal(format!("{message}"))
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Parse(parse_err) => !matches!(
                parse_err,
                crate::parser::ParseError::OutOfMemory { .. }
                    | crate::parser::ParseError::InputTooLarge { .. }
                    | crate::parser::ParseError::InternalError { .. }
            ),
            Self::Tokenization(_)
            | Self::InvalidColor(_)
            | Self::InvalidNumeric(_)
            | Self::InvalidTime(_)
            | Self::Validation(_) => true,

            Self::OutOfMemory(_)
            | Self::ResourceLimitExceeded { .. }
            | Self::SecurityViolation(_)
            | Self::Internal(_) => false,

            Self::Analysis(_)
            | Self::Plugin(_)
            | Self::Utf8Error { .. }
            | Self::Io(_)
            | Self::Config(_)
            | Self::FeatureNotSupported { .. }
            | Self::VersionIncompatible { .. } => true,
        }
    }

    /// Check if error indicates a bug in the library
    pub fn is_internal_bug(&self) -> bool {
        matches!(self, Self::Internal(_))
    }

    /// Get the underlying parse error if this is a parse error
    pub fn as_parse_error(&self) -> Option<&crate::parser::ParseError> {
        match self {
            Self::Parse(parse_err) => Some(parse_err),
            _ => None,
        }
    }

    /// Get line number for errors that have location information
    pub fn line_number(&self) -> Option<usize> {
        match self {
            Self::Parse(parse_err) => match parse_err {
                crate::parser::ParseError::ExpectedSectionHeader { line } => Some(*line),
                crate::parser::ParseError::UnclosedSectionHeader { line } => Some(*line),
                crate::parser::ParseError::UnknownSection { line, .. } => Some(*line),
                crate::parser::ParseError::InvalidFieldFormat { line } => Some(*line),
                crate::parser::ParseError::InvalidFormatLine { line, .. } => Some(*line),
                crate::parser::ParseError::FieldCountMismatch { line, .. } => Some(*line),
                crate::parser::ParseError::InvalidTimeFormat { line, .. } => Some(*line),
                crate::parser::ParseError::InvalidColorFormat { line, .. } => Some(*line),
                crate::parser::ParseError::InvalidNumericValue { line, .. } => Some(*line),
                crate::parser::ParseError::InvalidStyleOverride { line, .. } => Some(*line),
                crate::parser::ParseError::InvalidDrawingCommand { line, .. } => Some(*line),
                crate::parser::ParseError::UuDecodeError { line, .. } => Some(*line),
                crate::parser::ParseError::MaxNestingDepth { line, .. } => Some(*line),
                crate::parser::ParseError::InternalError { line, .. } => Some(*line),
                _ => None,
            },
            Self::Utf8Error { position, .. } => Some(*position),
            _ => None,
        }
    }

    /// Check if this is a specific type of parse error
    #[must_use]
    pub fn is_parse_error_type(&self, error_type: &str) -> bool {
        match self {
            Self::Parse(parse_err) => matches!(
                (error_type, parse_err),
                (
                    "section_header",
                    crate::parser::ParseError::ExpectedSectionHeader { .. }
                ) | (
                    "unclosed_header",
                    crate::parser::ParseError::UnclosedSectionHeader { .. }
                ) | (
                    "unknown_section",
                    crate::parser::ParseError::UnknownSection { .. }
                ) | (
                    "field_format",
                    crate::parser::ParseError::InvalidFieldFormat { .. }
                ) | (
                    "time_format",
                    crate::parser::ParseError::InvalidTimeFormat { .. }
                ) | (
                    "color_format",
                    crate::parser::ParseError::InvalidColorFormat { .. }
                ) | (
                    "numeric_value",
                    crate::parser::ParseError::InvalidNumericValue { .. }
                ) | ("utf8", crate::parser::ParseError::Utf8Error { .. })
            ),
            _ => false,
        }
    }
}

/// Result type alias for convenience
pub type Result<T> = core::result::Result<T, CoreError>;

/// no_std compatible Display implementation
#[cfg(not(feature = "std"))]
impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(parse_err) => write!(f, "Parse error: {parse_err}"),
            Self::Tokenization(msg) => write!(f, "Tokenization error: {msg}"),
            Self::Analysis(msg) => write!(f, "Analysis error: {msg}"),
            Self::Plugin(msg) => write!(f, "Plugin error: {msg}"),
            Self::InvalidColor(msg) => write!(f, "Invalid color format: {msg}"),
            Self::InvalidNumeric(msg) => write!(f, "Invalid numeric value: {msg}"),
            Self::InvalidTime(msg) => write!(f, "Invalid time format: {msg}"),
            Self::Utf8Error { position, message } => {
                write!(f, "UTF-8 encoding error at position {position}: {message}")
            }
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
            Self::OutOfMemory(msg) => write!(f, "Memory allocation failed: {msg}"),
            Self::Config(msg) => write!(f, "Configuration error: {msg}"),
            Self::Validation(msg) => write!(f, "Validation error: {msg}"),
            Self::FeatureNotSupported {
                feature,
                required_feature,
            } => {
                write!(
                    f,
                    "Feature not supported: {feature} (requires feature '{required_feature}')"
                )
            }
            Self::VersionIncompatible { message } => {
                write!(f, "Version incompatibility: {message}")
            }
            Self::ResourceLimitExceeded {
                resource,
                current,
                limit,
            } => {
                write!(f, "Resource limit exceeded: {resource} ({current}/{limit})")
            }
            Self::SecurityViolation(msg) => write!(f, "Security policy violation: {msg}"),
            Self::Internal(msg) => {
                write!(f, "Internal error: {msg} (this is a bug, please report)")
            }
        }
    }
}

/// no_std compatible Error implementation
#[cfg(not(feature = "std"))]
impl core::error::Error for CoreError {}

/// std compatible Display implementation
#[cfg(feature = "std")]
impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(parse_err) => write!(f, "Parse error: {parse_err}"),
            Self::Tokenization(msg) => write!(f, "Tokenization error: {msg}"),
            Self::Analysis(msg) => write!(f, "Analysis error: {msg}"),
            Self::Plugin(msg) => write!(f, "Plugin error: {msg}"),
            Self::InvalidColor(msg) => write!(f, "Invalid color format: {msg}"),
            Self::InvalidNumeric(msg) => write!(f, "Invalid numeric value: {msg}"),
            Self::InvalidTime(msg) => write!(f, "Invalid time format: {msg}"),
            Self::Utf8Error { position, message } => {
                write!(f, "UTF-8 encoding error at position {position}: {message}")
            }
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
            Self::OutOfMemory(msg) => write!(f, "Memory allocation failed: {msg}"),
            Self::Config(msg) => write!(f, "Configuration error: {msg}"),
            Self::Validation(msg) => write!(f, "Validation error: {msg}"),
            Self::FeatureNotSupported {
                feature,
                required_feature,
            } => {
                write!(
                    f,
                    "Feature not supported: {feature} (requires feature '{required_feature}')"
                )
            }
            Self::VersionIncompatible { message } => {
                write!(f, "Version incompatibility: {message}")
            }
            Self::ResourceLimitExceeded {
                resource,
                current,
                limit,
            } => {
                write!(f, "Resource limit exceeded: {resource} ({current}/{limit})")
            }
            Self::SecurityViolation(msg) => write!(f, "Security policy violation: {msg}"),
            Self::Internal(msg) => {
                write!(f, "Internal error: {msg} (this is a bug, please report)")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_creation() {
        let parse_err = CoreError::parse("test message");
        assert!(matches!(parse_err, CoreError::Parse(_)));
    }

    #[test]
    fn internal_error() {
        let internal_err = CoreError::internal("something went wrong");
        assert!(matches!(internal_err, CoreError::Internal(_)));
        assert!(internal_err.is_internal_bug());
        assert!(!internal_err.is_recoverable());
    }

    #[test]
    fn error_recoverability() {
        assert!(CoreError::parse("test").is_recoverable());
        assert!(!CoreError::internal("test").is_recoverable());
    }

    #[test]
    fn internal_bug_detection() {
        assert!(CoreError::internal("test").is_internal_bug());
        assert!(!CoreError::parse("test").is_internal_bug());
    }
}
