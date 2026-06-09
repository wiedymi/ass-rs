//! Tests for `CoreError` classification helpers: recoverability, internal-bug
//! detection, parse-error extraction, and parse-error type checking.

use crate::parser::ParseError;
use crate::utils::errors::CoreError;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn error_recoverability() {
    // Recoverable errors
    assert!(CoreError::parse("test").is_recoverable());
    assert!(CoreError::Tokenization("test".to_string()).is_recoverable());
    assert!(CoreError::InvalidColor("test".to_string()).is_recoverable());
    assert!(CoreError::InvalidNumeric("test".to_string()).is_recoverable());
    assert!(CoreError::InvalidTime("test".to_string()).is_recoverable());
    assert!(CoreError::Validation("test".to_string()).is_recoverable());
    assert!(CoreError::Analysis("test".to_string()).is_recoverable());
    assert!(CoreError::Plugin("test".to_string()).is_recoverable());
    assert!(CoreError::Utf8Error {
        position: 0,
        message: "test".to_string(),
    }
    .is_recoverable());
    assert!(CoreError::Io("test".to_string()).is_recoverable());
    assert!(CoreError::Config("test".to_string()).is_recoverable());
    assert!(CoreError::FeatureNotSupported {
        feature: "test".to_string(),
        required_feature: "test".to_string(),
    }
    .is_recoverable());
    assert!(CoreError::VersionIncompatible {
        message: "test".to_string(),
    }
    .is_recoverable());

    // Non-recoverable errors
    assert!(!CoreError::OutOfMemory("test".to_string()).is_recoverable());
    assert!(!CoreError::ResourceLimitExceeded {
        resource: "test".to_string(),
        current: 100,
        limit: 50,
    }
    .is_recoverable());
    assert!(!CoreError::SecurityViolation("test".to_string()).is_recoverable());
    assert!(!CoreError::Internal("test".to_string()).is_recoverable());

    // Parse errors - check specific variants
    let recoverable_parse = CoreError::Parse(ParseError::IoError {
        message: "test".to_string(),
    });
    assert!(recoverable_parse.is_recoverable());

    let non_recoverable_parse = CoreError::Parse(ParseError::OutOfMemory {
        message: "test".to_string(),
    });
    assert!(!non_recoverable_parse.is_recoverable());

    let non_recoverable_large = CoreError::Parse(ParseError::InputTooLarge {
        size: 1000,
        limit: 500,
    });
    assert!(!non_recoverable_large.is_recoverable());

    let non_recoverable_internal = CoreError::Parse(ParseError::InternalError {
        line: 1,
        message: "test".to_string(),
    });
    assert!(!non_recoverable_internal.is_recoverable());
}

#[test]
fn internal_bug_detection() {
    assert!(CoreError::Internal("test".to_string()).is_internal_bug());
    assert!(!CoreError::parse("test").is_internal_bug());
    assert!(!CoreError::Tokenization("test".to_string()).is_internal_bug());
    assert!(!CoreError::Analysis("test".to_string()).is_internal_bug());
    assert!(!CoreError::Plugin("test".to_string()).is_internal_bug());
    assert!(!CoreError::InvalidColor("test".to_string()).is_internal_bug());
    assert!(!CoreError::OutOfMemory("test".to_string()).is_internal_bug());
}

#[test]
fn as_parse_error() {
    let parse_error = CoreError::Parse(ParseError::IoError {
        message: "test".to_string(),
    });
    assert!(parse_error.as_parse_error().is_some());

    let non_parse_error = CoreError::Tokenization("test".to_string());
    assert!(non_parse_error.as_parse_error().is_none());
}

#[test]
fn parse_error_type_checking() {
    let section_header_error = CoreError::Parse(ParseError::ExpectedSectionHeader { line: 1 });
    assert!(section_header_error.is_parse_error_type("section_header"));
    assert!(!section_header_error.is_parse_error_type("time_format"));

    let unclosed_header_error = CoreError::Parse(ParseError::UnclosedSectionHeader { line: 1 });
    assert!(unclosed_header_error.is_parse_error_type("unclosed_header"));

    let unknown_section_error = CoreError::Parse(ParseError::UnknownSection {
        line: 1,
        section: "Bad".to_string(),
    });
    assert!(unknown_section_error.is_parse_error_type("unknown_section"));

    let field_format_error = CoreError::Parse(ParseError::InvalidFieldFormat { line: 1 });
    assert!(field_format_error.is_parse_error_type("field_format"));

    let time_format_error = CoreError::Parse(ParseError::InvalidTimeFormat {
        line: 1,
        time: "bad".to_string(),
        reason: "invalid".to_string(),
    });
    assert!(time_format_error.is_parse_error_type("time_format"));

    let color_format_error = CoreError::Parse(ParseError::InvalidColorFormat {
        line: 1,
        color: "bad".to_string(),
        reason: "invalid".to_string(),
    });
    assert!(color_format_error.is_parse_error_type("color_format"));

    let numeric_error = CoreError::Parse(ParseError::InvalidNumericValue {
        line: 1,
        value: "bad".to_string(),
        reason: "invalid".to_string(),
    });
    assert!(numeric_error.is_parse_error_type("numeric_value"));

    let utf8_parse_error = CoreError::Parse(ParseError::Utf8Error {
        position: 1,
        reason: "bad utf8".to_string(),
    });
    assert!(utf8_parse_error.is_parse_error_type("utf8"));

    // Non-parse error should return false
    let tokenization_error = CoreError::Tokenization("test".to_string());
    assert!(!tokenization_error.is_parse_error_type("section_header"));
}
