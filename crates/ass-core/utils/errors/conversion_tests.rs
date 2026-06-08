//! Inline unit tests for [`CoreError`] conversions, re-exports, and display.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString, vec};

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
