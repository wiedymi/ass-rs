//! Inline unit tests for [`CoreError`] construction and property accessors.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};

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
