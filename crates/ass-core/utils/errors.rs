//! Core error types for ASS-RS utilities and cross-module error handling
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

use alloc::{format, string::String, string::ToString};
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
    Parse(String),

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
        Self::Parse(format!("{}", message))
    }

    /// Create color error from invalid format
    pub fn invalid_color<T: fmt::Display>(format: T) -> Self {
        Self::InvalidColor(format!("{}", format))
    }

    /// Create numeric error from parsing failure
    pub fn invalid_numeric<T: fmt::Display>(value: T, reason: &str) -> Self {
        Self::InvalidNumeric(format!("'{}': {}", value, reason))
    }

    /// Create time error from invalid format
    pub fn invalid_time<T: fmt::Display>(time: T, reason: &str) -> Self {
        Self::InvalidTime(format!("'{}': {}", time, reason))
    }

    /// Create UTF-8 error with position
    pub fn utf8_error(position: usize, message: String) -> Self {
        Self::Utf8Error { position, message }
    }

    /// Create feature not supported error
    pub fn feature_not_supported(feature: &str, required_feature: &str) -> Self {
        Self::FeatureNotSupported {
            feature: feature.to_string(),
            required_feature: required_feature.to_string(),
        }
    }

    /// Create resource limit error
    pub fn resource_limit_exceeded(resource: &str, current: usize, limit: usize) -> Self {
        Self::ResourceLimitExceeded {
            resource: resource.to_string(),
            current,
            limit,
        }
    }

    /// Create internal error (indicates a bug)
    pub fn internal<T: fmt::Display>(message: T) -> Self {
        Self::Internal(format!("{}", message))
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Parse(_)
            | Self::Tokenization(_)
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

    /// Get error category for filtering/grouping
    pub fn category(&self) -> ErrorCategory {
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
    pub fn suggestion(&self) -> Option<&'static str> {
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
            _ => None,
        }
    }
}

/// no_std compatible Display implementation
#[cfg(not(feature = "std"))]
impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Parse(msg) => write!(f, "Parse error: {}", msg),
            CoreError::Tokenization(msg) => write!(f, "Tokenization error: {}", msg),
            CoreError::Analysis(msg) => write!(f, "Analysis error: {}", msg),
            CoreError::Plugin(msg) => write!(f, "Plugin error: {}", msg),
            CoreError::InvalidColor(msg) => write!(f, "Invalid color format: {}", msg),
            CoreError::InvalidNumeric(msg) => write!(f, "Invalid numeric value: {}", msg),
            CoreError::InvalidTime(msg) => write!(f, "Invalid time format: {}", msg),
            CoreError::Utf8Error { position, message } => {
                write!(
                    f,
                    "UTF-8 encoding error at position {}: {}",
                    position, message
                )
            }
            CoreError::Io(msg) => write!(f, "I/O error: {}", msg),
            CoreError::OutOfMemory(msg) => write!(f, "Memory allocation failed: {}", msg),
            CoreError::Config(msg) => write!(f, "Configuration error: {}", msg),
            CoreError::Validation(msg) => write!(f, "Validation error: {}", msg),
            CoreError::FeatureNotSupported {
                feature,
                required_feature,
            } => {
                write!(
                    f,
                    "Feature not supported: {} (requires feature '{}')",
                    feature, required_feature
                )
            }
            CoreError::VersionIncompatible { message } => {
                write!(f, "Version incompatibility: {}", message)
            }
            CoreError::ResourceLimitExceeded {
                resource,
                current,
                limit,
            } => {
                write!(
                    f,
                    "Resource limit exceeded: {} ({}/{})",
                    resource, current, limit
                )
            }
            CoreError::SecurityViolation(msg) => write!(f, "Security policy violation: {}", msg),
            CoreError::Internal(msg) => {
                write!(f, "Internal error: {} (this is a bug, please report)", msg)
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
            CoreError::Parse(msg) => write!(f, "Parse error: {}", msg),
            CoreError::Tokenization(msg) => write!(f, "Tokenization error: {}", msg),
            CoreError::Analysis(msg) => write!(f, "Analysis error: {}", msg),
            CoreError::Plugin(msg) => write!(f, "Plugin error: {}", msg),
            CoreError::InvalidColor(msg) => write!(f, "Invalid color format: {}", msg),
            CoreError::InvalidNumeric(msg) => write!(f, "Invalid numeric value: {}", msg),
            CoreError::InvalidTime(msg) => write!(f, "Invalid time format: {}", msg),
            CoreError::Utf8Error { position, message } => {
                write!(
                    f,
                    "UTF-8 encoding error at position {}: {}",
                    position, message
                )
            }
            CoreError::Io(msg) => write!(f, "I/O error: {}", msg),
            CoreError::OutOfMemory(msg) => write!(f, "Memory allocation failed: {}", msg),
            CoreError::Config(msg) => write!(f, "Configuration error: {}", msg),
            CoreError::Validation(msg) => write!(f, "Validation error: {}", msg),
            CoreError::FeatureNotSupported {
                feature,
                required_feature,
            } => {
                write!(
                    f,
                    "Feature not supported: {} (requires feature '{}')",
                    feature, required_feature
                )
            }
            CoreError::VersionIncompatible { message } => {
                write!(f, "Version incompatibility: {}", message)
            }
            CoreError::ResourceLimitExceeded {
                resource,
                current,
                limit,
            } => {
                write!(
                    f,
                    "Resource limit exceeded: {} ({}/{})",
                    resource, current, limit
                )
            }
            CoreError::SecurityViolation(msg) => write!(f, "Security policy violation: {}", msg),
            CoreError::Internal(msg) => {
                write!(f, "Internal error: {} (this is a bug, please report)", msg)
            }
        }
    }
}

/// Error category for filtering and user interface organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// Parsing and tokenization errors
    Parsing,

    /// Analysis and linting errors
    Analysis,

    /// Plugin system errors
    Plugin,

    /// Format validation errors
    Format,

    /// Text encoding errors
    Encoding,

    /// I/O and file system errors
    Io,

    /// Resource and memory errors
    Resource,

    /// Configuration errors
    Configuration,

    /// Data validation errors
    Validation,

    /// Compatibility and version errors
    Compatibility,

    /// Security policy violations
    Security,

    /// Internal library bugs
    Internal,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::Parsing => write!(f, "parsing"),
            ErrorCategory::Analysis => write!(f, "analysis"),
            ErrorCategory::Plugin => write!(f, "plugin"),
            ErrorCategory::Format => write!(f, "format"),
            ErrorCategory::Encoding => write!(f, "encoding"),
            ErrorCategory::Io => write!(f, "io"),
            ErrorCategory::Resource => write!(f, "resource"),
            ErrorCategory::Configuration => write!(f, "configuration"),
            ErrorCategory::Validation => write!(f, "validation"),
            ErrorCategory::Compatibility => write!(f, "compatibility"),
            ErrorCategory::Security => write!(f, "security"),
            ErrorCategory::Internal => write!(f, "internal"),
        }
    }
}

/// Convert from parser errors
impl From<crate::parser::ParseError> for CoreError {
    fn from(err: crate::parser::ParseError) -> Self {
        Self::Parse(format!("{}", err))
    }
}

/// Convert from standard I/O errors (when std is available)
#[cfg(feature = "std")]
impl From<std::io::Error> for CoreError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(format!("{}", err))
    }
}

/// Convert from core::str::Utf8Error
impl From<core::str::Utf8Error> for CoreError {
    fn from(err: core::str::Utf8Error) -> Self {
        Self::Utf8Error {
            position: 0, // Position not available from Utf8Error
            message: format!("{}", err),
        }
    }
}

/// Convert from alloc string parse errors
impl From<core::num::ParseIntError> for CoreError {
    fn from(err: core::num::ParseIntError) -> Self {
        Self::InvalidNumeric(format!("Integer parse error: {}", err))
    }
}

impl From<core::num::ParseFloatError> for CoreError {
    fn from(err: core::num::ParseFloatError) -> Self {
        Self::InvalidNumeric(format!("Float parse error: {}", err))
    }
}

/// Result type alias for convenience
pub type Result<T> = core::result::Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_creation() {
        let parse_err = CoreError::parse("test message");
        assert!(matches!(parse_err, CoreError::Parse(_)));
        assert_eq!(format!("{}", parse_err), "Parse error: test message");
    }

    #[test]
    fn color_error() {
        let color_err = CoreError::invalid_color("invalid");
        assert!(matches!(color_err, CoreError::InvalidColor(_)));
        assert_eq!(format!("{}", color_err), "Invalid color format: invalid");
    }

    #[test]
    fn numeric_error() {
        let num_err = CoreError::invalid_numeric("abc", "not a number");
        assert!(matches!(num_err, CoreError::InvalidNumeric(_)));
        assert_eq!(
            format!("{}", num_err),
            "Invalid numeric value: 'abc': not a number"
        );
    }

    #[test]
    fn time_error() {
        let time_err = CoreError::invalid_time("invalid", "wrong format");
        assert!(matches!(time_err, CoreError::InvalidTime(_)));
        assert_eq!(
            format!("{}", time_err),
            "Invalid time format: 'invalid': wrong format"
        );
    }

    #[test]
    fn utf8_error() {
        let utf8_err = CoreError::utf8_error(42, "invalid sequence".to_string());
        assert!(matches!(utf8_err, CoreError::Utf8Error { .. }));
        assert_eq!(
            format!("{}", utf8_err),
            "UTF-8 encoding error at position 42: invalid sequence"
        );
    }

    #[test]
    fn feature_not_supported() {
        let feature_err = CoreError::feature_not_supported("simd", "simd");
        assert!(matches!(feature_err, CoreError::FeatureNotSupported { .. }));
        assert_eq!(
            format!("{}", feature_err),
            "Feature not supported: simd (requires feature 'simd')"
        );
    }

    #[test]
    fn resource_limit() {
        let limit_err = CoreError::resource_limit_exceeded("memory", 1000, 500);
        assert!(matches!(limit_err, CoreError::ResourceLimitExceeded { .. }));
        assert_eq!(
            format!("{}", limit_err),
            "Resource limit exceeded: memory (1000/500)"
        );
    }

    #[test]
    fn internal_error() {
        let internal_err = CoreError::internal("something went wrong");
        assert!(matches!(internal_err, CoreError::Internal(_)));
        assert_eq!(
            format!("{}", internal_err),
            "Internal error: something went wrong (this is a bug, please report)"
        );
    }

    #[test]
    fn error_recoverability() {
        assert!(CoreError::parse("test").is_recoverable());
        assert!(CoreError::invalid_color("test").is_recoverable());
        assert!(!CoreError::internal("test").is_recoverable());
        assert!(!CoreError::OutOfMemory("test".to_string()).is_recoverable());
    }

    #[test]
    fn error_categories() {
        assert_eq!(CoreError::parse("test").category(), ErrorCategory::Parsing);
        assert_eq!(
            CoreError::invalid_color("test").category(),
            ErrorCategory::Format
        );
        assert_eq!(
            CoreError::internal("test").category(),
            ErrorCategory::Internal
        );
    }

    #[test]
    fn error_suggestions() {
        assert!(CoreError::invalid_color("test").suggestion().is_some());
        assert!(CoreError::invalid_time("test", "reason")
            .suggestion()
            .is_some());
        assert!(CoreError::internal("test").suggestion().is_some());
        assert!(CoreError::parse("test").suggestion().is_none());
    }

    #[test]
    fn category_display() {
        assert_eq!(format!("{}", ErrorCategory::Parsing), "parsing");
        assert_eq!(format!("{}", ErrorCategory::Format), "format");
        assert_eq!(format!("{}", ErrorCategory::Internal), "internal");
    }

    #[test]
    fn error_conversion() {
        let parse_int_err: core::num::ParseIntError = "abc".parse::<i32>().unwrap_err();
        let core_err: CoreError = parse_int_err.into();
        assert!(matches!(core_err, CoreError::InvalidNumeric(_)));

        let parse_float_err: core::num::ParseFloatError = "xyz".parse::<f32>().unwrap_err();
        let core_err: CoreError = parse_float_err.into();
        assert!(matches!(core_err, CoreError::InvalidNumeric(_)));
    }

    #[test]
    fn internal_bug_detection() {
        assert!(CoreError::internal("test").is_internal_bug());
        assert!(!CoreError::parse("test").is_internal_bug());
        assert!(!CoreError::invalid_color("test").is_internal_bug());
    }
}
