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

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::format;
mod category;
mod core;
pub mod encoding;
mod format;
pub mod resource;

// Re-export all public types to maintain API compatibility
pub use category::ErrorCategory;
pub use core::{CoreError, Result};

// Re-export utility functions from sub-modules
pub use encoding::{
    utf8_error, validate_ass_text_content, validate_bom_handling, validate_utf8_detailed,
};
pub use format::{invalid_color, invalid_numeric, invalid_time, validate_color_format};
pub use resource::{
    check_memory_limit, feature_not_supported, out_of_memory, resource_limit_exceeded,
};

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
    #[cfg(not(feature = "std"))]
    use alloc::{format, string::ToString, vec};

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

    #[test]
    fn core_error_invalid_color_convenience() {
        let color_err = CoreError::invalid_color("red");
        assert!(matches!(color_err, CoreError::InvalidColor(_)));
        if let CoreError::InvalidColor(msg) = color_err {
            assert!(msg.contains("red"));
        }
    }

    #[test]
    fn core_error_invalid_numeric_convenience() {
        let numeric_err = CoreError::invalid_numeric("abc", "not a number");
        assert!(matches!(numeric_err, CoreError::InvalidNumeric(_)));
        if let CoreError::InvalidNumeric(msg) = numeric_err {
            assert!(msg.contains("abc"));
            assert!(msg.contains("not a number"));
        }
    }

    #[test]
    fn core_error_invalid_time_convenience() {
        let time_err = CoreError::invalid_time("25:00:00", "hours out of range");
        assert!(matches!(time_err, CoreError::InvalidTime(_)));
        if let CoreError::InvalidTime(msg) = time_err {
            assert!(msg.contains("25:00:00"));
            assert!(msg.contains("hours out of range"));
        }
    }

    #[test]
    fn core_error_utf8_error_convenience() {
        let utf8_err = CoreError::utf8_error(42, "invalid sequence".to_string());
        assert!(matches!(utf8_err, CoreError::Utf8Error { .. }));
        if let CoreError::Utf8Error { position, message } = utf8_err {
            assert_eq!(position, 42);
            assert_eq!(message, "invalid sequence");
        }
    }

    #[test]
    fn core_error_feature_not_supported_convenience() {
        let feature_err = CoreError::feature_not_supported("simd", "cpu-features");
        assert!(matches!(feature_err, CoreError::FeatureNotSupported { .. }));
        if let CoreError::FeatureNotSupported {
            feature,
            required_feature,
        } = feature_err
        {
            assert_eq!(feature, "simd");
            assert_eq!(required_feature, "cpu-features");
        }
    }

    #[test]
    fn core_error_resource_limit_exceeded_convenience() {
        let resource_err = CoreError::resource_limit_exceeded("memory", 1024, 512);
        assert!(matches!(
            resource_err,
            CoreError::ResourceLimitExceeded { .. }
        ));
        if let CoreError::ResourceLimitExceeded {
            resource,
            current,
            limit,
        } = resource_err
        {
            assert_eq!(resource, "memory");
            assert_eq!(current, 1024);
            assert_eq!(limit, 512);
        }
    }

    #[test]
    fn utf8_error_conversion() {
        let invalid_bytes = vec![0xFF, 0xFE];
        let utf8_err = ::core::str::from_utf8(&invalid_bytes).unwrap_err();
        let core_err: CoreError = utf8_err.into();
        assert!(matches!(core_err, CoreError::Utf8Error { .. }));
        if let CoreError::Utf8Error { position, message } = core_err {
            assert_eq!(position, 0); // Position not available from Utf8Error
            assert!(message.contains("invalid utf-8"));
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn io_error_conversion() {
        use std::io::{Error, ErrorKind};

        let io_err = Error::new(ErrorKind::NotFound, "file not found");
        let core_err: CoreError = io_err.into();
        assert!(matches!(core_err, CoreError::Io(_)));
        if let CoreError::Io(msg) = core_err {
            assert!(msg.contains("file not found"));
        }
    }

    #[test]
    fn parse_error_conversion() {
        let parse_err = crate::parser::ParseError::IoError {
            message: "io failure".to_string(),
        };
        let core_err: CoreError = parse_err.into();
        assert!(matches!(core_err, CoreError::Parse(_)));
    }

    #[test]
    fn numeric_conversion_edge_cases() {
        // Test different numeric parsing errors
        let int_overflow: ::core::num::ParseIntError =
            "999999999999999999999999999".parse::<i32>().unwrap_err();
        let core_err: CoreError = int_overflow.into();
        assert!(matches!(core_err, CoreError::InvalidNumeric(_)));
        if let CoreError::InvalidNumeric(msg) = core_err {
            assert!(msg.contains("Integer parse error"));
        }

        let float_err: ::core::num::ParseFloatError = "not_a_float".parse::<f64>().unwrap_err();
        let core_err: CoreError = float_err.into();
        assert!(matches!(core_err, CoreError::InvalidNumeric(_)));
        if let CoreError::InvalidNumeric(msg) = core_err {
            assert!(msg.contains("Float parse error"));
        }
    }

    #[test]
    fn module_re_exports() {
        // Test that all re-exported functions are accessible
        let _color_err = invalid_color("test");
        let _numeric_err = invalid_numeric("test", "reason");
        let _time_err = invalid_time("test", "reason");
        let _utf8_err = utf8_error(0, "message".to_string());
        let _feature_err = feature_not_supported("feature", "required");
        let _resource_err = resource_limit_exceeded("resource", 100, 50);
        let _memory_err = out_of_memory("test");
        let _limit_err = check_memory_limit(1000, 500, 800);

        // Test validation functions
        let _validated = validate_utf8_detailed(b"test");
        let _color_validated = validate_color_format("&H000000");
        let _ass_validated = validate_ass_text_content("test");
        let _bom_validated = validate_bom_handling(b"test");
    }

    #[test]
    fn error_display_consistency() {
        // Test that all error types have consistent display formatting
        let errors = vec![
            CoreError::Tokenization("test".to_string()),
            CoreError::Analysis("test".to_string()),
            CoreError::Plugin("test".to_string()),
            CoreError::InvalidColor("test".to_string()),
            CoreError::InvalidNumeric("test".to_string()),
            CoreError::InvalidTime("test".to_string()),
            CoreError::Io("test".to_string()),
            CoreError::OutOfMemory("test".to_string()),
            CoreError::Config("test".to_string()),
            CoreError::Validation("test".to_string()),
            CoreError::SecurityViolation("test".to_string()),
            CoreError::Internal("test".to_string()),
        ];

        for error in errors {
            let display_str = format!("{error}");
            assert!(!display_str.is_empty());
            assert!(display_str.contains("test"));
        }

        // Test complex error types
        let utf8_err = CoreError::Utf8Error {
            position: 42,
            message: "test".to_string(),
        };
        let display_str = format!("{utf8_err}");
        assert!(display_str.contains("42"));
        assert!(display_str.contains("test"));

        let feature_err = CoreError::FeatureNotSupported {
            feature: "feature1".to_string(),
            required_feature: "feature2".to_string(),
        };
        let display_str = format!("{feature_err}");
        assert!(display_str.contains("feature1"));
        assert!(display_str.contains("feature2"));

        let version_err = CoreError::VersionIncompatible {
            message: "version mismatch".to_string(),
        };
        let display_str = format!("{version_err}");
        assert!(display_str.contains("version mismatch"));

        let resource_err = CoreError::ResourceLimitExceeded {
            resource: "memory".to_string(),
            current: 100,
            limit: 50,
        };
        let display_str = format!("{resource_err}");
        assert!(display_str.contains("memory"));
        assert!(display_str.contains("100"));
        assert!(display_str.contains("50"));
    }
}
