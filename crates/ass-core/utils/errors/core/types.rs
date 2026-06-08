//! `CoreError` enum definition and the crate `Result` alias.
//!
//! Holds the unified error type wrapping every module-specific error along
//! with the convenience `Result` alias used throughout the crate.

use alloc::string::String;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(feature = "std")]
use thiserror::Error;

/// Main error type for ASS-RS core operations
///
/// Wraps all error types from different modules to provide a unified
/// error handling interface. Can be converted from module-specific errors.
#[cfg_attr(feature = "std", derive(Error))]
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Result type alias for convenience
pub type Result<T> = core::result::Result<T, CoreError>;
