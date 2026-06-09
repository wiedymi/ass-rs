//! Tests for `Display` and `Debug` formatting of every `CoreError` variant.

use crate::parser::ParseError;
use crate::utils::errors::CoreError;
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};

#[test]
fn all_error_variants_display() {
    let parse_error = CoreError::Parse(ParseError::IoError {
        message: "io error".to_string(),
    });
    assert!(format!("{parse_error}").contains("Parse error"));

    let tokenization_error = CoreError::Tokenization("token error".to_string());
    assert_eq!(
        format!("{tokenization_error}"),
        "Tokenization error: token error"
    );

    let analysis_error = CoreError::Analysis("analysis error".to_string());
    assert_eq!(
        format!("{analysis_error}"),
        "Analysis error: analysis error"
    );

    let plugin_error = CoreError::Plugin("plugin error".to_string());
    assert_eq!(format!("{plugin_error}"), "Plugin error: plugin error");

    let color_error = CoreError::InvalidColor("bad color".to_string());
    assert_eq!(format!("{color_error}"), "Invalid color format: bad color");

    let numeric_error = CoreError::InvalidNumeric("bad number".to_string());
    assert_eq!(
        format!("{numeric_error}"),
        "Invalid numeric value: bad number"
    );

    let time_error = CoreError::InvalidTime("bad time".to_string());
    assert_eq!(format!("{time_error}"), "Invalid time format: bad time");

    let utf8_error = CoreError::Utf8Error {
        position: 42,
        message: "invalid utf8".to_string(),
    };
    assert_eq!(
        format!("{utf8_error}"),
        "UTF-8 encoding error at position 42: invalid utf8"
    );

    let io_error = CoreError::Io("file not found".to_string());
    assert_eq!(format!("{io_error}"), "I/O error: file not found");

    let memory_error = CoreError::OutOfMemory("allocation failed".to_string());
    assert_eq!(
        format!("{memory_error}"),
        "Memory allocation failed: allocation failed"
    );

    let config_error = CoreError::Config("bad config".to_string());
    assert_eq!(format!("{config_error}"), "Configuration error: bad config");

    let validation_error = CoreError::Validation("validation failed".to_string());
    assert_eq!(
        format!("{validation_error}"),
        "Validation error: validation failed"
    );

    let feature_error = CoreError::FeatureNotSupported {
        feature: "advanced_rendering".to_string(),
        required_feature: "gpu".to_string(),
    };
    assert_eq!(
        format!("{feature_error}"),
        "Feature not supported: advanced_rendering (requires feature 'gpu')"
    );

    let version_error = CoreError::VersionIncompatible {
        message: "version mismatch".to_string(),
    };
    assert_eq!(
        format!("{version_error}"),
        "Version incompatibility: version mismatch"
    );

    let resource_error = CoreError::ResourceLimitExceeded {
        resource: "memory".to_string(),
        current: 100,
        limit: 50,
    };
    assert_eq!(
        format!("{resource_error}"),
        "Resource limit exceeded: memory (100/50)"
    );

    let security_error = CoreError::SecurityViolation("unauthorized access".to_string());
    assert_eq!(
        format!("{security_error}"),
        "Security policy violation: unauthorized access"
    );

    let internal_error = CoreError::Internal("bug detected".to_string());
    assert_eq!(
        format!("{internal_error}"),
        "Internal error: bug detected (this is a bug, please report)"
    );
}

#[test]
fn error_debug_formatting() {
    let error = CoreError::Analysis("debug test".to_string());
    let debug_output = format!("{error:?}");
    assert!(debug_output.contains("Analysis"));
    assert!(debug_output.contains("debug test"));
}

#[test]
fn utf8_error_formatting() {
    let error = CoreError::Utf8Error {
        position: 256,
        message: "Invalid UTF-8 sequence".to_string(),
    };

    let formatted = format!("{error}");
    assert!(formatted.contains("UTF-8 encoding error"));
    assert!(formatted.contains("position 256"));
    assert!(formatted.contains("Invalid UTF-8 sequence"));
}

#[test]
fn version_incompatible_error() {
    let error = CoreError::VersionIncompatible {
        message: "Script requires v4.00+, got v3.00".to_string(),
    };

    assert!(error.is_recoverable());
    assert!(!error.is_internal_bug());
    assert!(error.line_number().is_none());

    let formatted = format!("{error}");
    assert!(formatted.contains("Version incompatibility"));
    assert!(formatted.contains("Script requires v4.00+"));
}

#[test]
fn security_violation_error() {
    let error = CoreError::SecurityViolation("Attempt to access restricted file path".to_string());

    assert!(!error.is_recoverable());
    assert!(!error.is_internal_bug());
    assert!(error.line_number().is_none());

    let formatted = format!("{error}");
    assert!(formatted.contains("Security policy violation"));
    assert!(formatted.contains("restricted file path"));
}
