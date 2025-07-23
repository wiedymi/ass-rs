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
//!
//! # Examples
//!
//! ```rust
//! use ass_core::utils::errors::{CoreError, Result, ErrorCategory};
//!
//! // Create specific error types
//! let color_err = CoreError::invalid_color("invalid");
//! let time_err = CoreError::invalid_time("1:23", "missing seconds");
//!
//! // Check error properties
//! assert_eq!(color_err.category(), ErrorCategory::Format);
//! assert!(color_err.suggestion().is_some());
//! ```

mod category;
mod core;
pub mod encoding;
mod format;
pub mod resource;

// Re-export all public types to maintain API compatibility
pub use category::ErrorCategory;
pub use core::{CoreError, Result};

// Re-export utility functions from sub-modules
pub use encoding::{utf8_error, validate_ass_text_content, validate_utf8_detailed};
pub use format::{invalid_color, invalid_numeric, invalid_time, validate_color_format};
pub use resource::{
    check_memory_limit, feature_not_supported, out_of_memory, resource_limit_exceeded,
};

use alloc::format;

impl CoreError {
    /// Create color error from invalid format
    pub fn invalid_color<T: ::core::fmt::Display>(format: T) -> Self {
        format::invalid_color(format)
    }

    /// Create numeric error from parsing failure
    pub fn invalid_numeric<T: ::core::fmt::Display>(value: T, reason: &str) -> Self {
        format::invalid_numeric(value, reason)
    }

    /// Create time error from invalid format
    pub fn invalid_time<T: ::core::fmt::Display>(time: T, reason: &str) -> Self {
        format::invalid_time(time, reason)
    }

    /// Create UTF-8 error with position
    #[must_use]
    pub const fn utf8_error(position: usize, message: alloc::string::String) -> Self {
        encoding::utf8_error(position, message)
    }

    /// Create feature not supported error
    #[must_use]
    pub fn feature_not_supported(feature: &str, required_feature: &str) -> Self {
        resource::feature_not_supported(feature, required_feature)
    }

    /// Create resource limit error
    #[must_use]
    pub fn resource_limit_exceeded(resource: &str, current: usize, limit: usize) -> Self {
        resource::resource_limit_exceeded(resource, current, limit)
    }
}

/// Convert from parser errors
impl From<crate::parser::ParseError> for CoreError {
    fn from(err: crate::parser::ParseError) -> Self {
        Self::Parse(err)
    }
}

/// Convert from standard I/O errors (when std is available)
#[cfg(feature = "std")]
impl From<std::io::Error> for CoreError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(format!("{err}"))
    }
}

/// Convert from `core::str::Utf8Error`
impl From<::core::str::Utf8Error> for CoreError {
    fn from(err: ::core::str::Utf8Error) -> Self {
        Self::Utf8Error {
            position: 0, // Position not available from Utf8Error
            message: format!("{err}"),
        }
    }
}

/// Convert from integer parse errors
impl From<::core::num::ParseIntError> for CoreError {
    fn from(err: ::core::num::ParseIntError) -> Self {
        Self::InvalidNumeric(format!("Integer parse error: {err}"))
    }
}

/// Convert from float parse errors
impl From<::core::num::ParseFloatError> for CoreError {
    fn from(err: ::core::num::ParseFloatError) -> Self {
        Self::InvalidNumeric(format!("Float parse error: {err}"))
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod inline_tests {
    use super::*;

    #[test]
    fn error_creation_methods() {
        let parse_err = CoreError::parse("test message");
        assert!(matches!(parse_err, CoreError::Parse(_)));

        let color_err = CoreError::invalid_color("invalid");
        assert!(matches!(color_err, CoreError::InvalidColor(_)));

        let time_err = CoreError::invalid_time("invalid", "wrong format");
        assert!(matches!(time_err, CoreError::InvalidTime(_)));
    }

    #[test]
    fn error_display() {
        let error = CoreError::invalid_color("test");
        let display_str = format!("{error}");
        assert!(display_str.contains("Invalid color format"));
        assert!(display_str.contains("test"));
    }

    #[test]
    fn error_conversion() {
        let parse_int_err: ::core::num::ParseIntError = "abc".parse::<i32>().unwrap_err();
        let core_err: CoreError = parse_int_err.into();
        assert!(matches!(core_err, CoreError::InvalidNumeric(_)));

        let parse_float_err: ::core::num::ParseFloatError = "xyz".parse::<f32>().unwrap_err();
        let core_err: CoreError = parse_float_err.into();
        assert!(matches!(core_err, CoreError::InvalidNumeric(_)));
    }

    #[test]
    fn error_properties() {
        let error = CoreError::invalid_color("test");
        assert_eq!(error.category(), ErrorCategory::Format);
        assert!(error.suggestion().is_some());
        assert!(error.is_recoverable());
        assert!(!error.is_internal_bug());
    }

    #[test]
    fn result_type_alias() {
        fn test_function() -> i32 {
            42
        }

        assert_eq!(test_function(), 42);
    }
}
