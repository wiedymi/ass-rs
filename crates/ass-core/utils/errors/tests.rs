//! Comprehensive tests for error handling functionality

use super::core::*;
use crate::parser::ParseError;
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

use alloc::format;
#[test]
fn core_error_parse_creation() {
    let error = CoreError::parse("test parse error");
    assert!(matches!(error, CoreError::Parse(_)));

    if let CoreError::Parse(parse_err) = error {
        match parse_err {
            ParseError::IoError { message } => {
                assert_eq!(message, "test parse error");
            }
            _ => panic!("Expected IoError variant"),
        }
    }
}

#[test]
fn core_error_internal_creation() {
    let error = CoreError::internal("internal bug");
    assert!(matches!(error, CoreError::Internal(_)));

    if let CoreError::Internal(message) = error {
        assert_eq!(message, "internal bug");
    }
}

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
fn line_number_extraction() {
    // Parse errors with line numbers
    let section_header_error = CoreError::Parse(ParseError::ExpectedSectionHeader { line: 42 });
    assert_eq!(section_header_error.line_number(), Some(42));

    let unclosed_header_error = CoreError::Parse(ParseError::UnclosedSectionHeader { line: 100 });
    assert_eq!(unclosed_header_error.line_number(), Some(100));

    let unknown_section_error = CoreError::Parse(ParseError::UnknownSection {
        line: 25,
        section: "BadSection".to_string(),
    });
    assert_eq!(unknown_section_error.line_number(), Some(25));

    let field_format_error = CoreError::Parse(ParseError::InvalidFieldFormat { line: 15 });
    assert_eq!(field_format_error.line_number(), Some(15));

    let format_line_error = CoreError::Parse(ParseError::InvalidFormatLine {
        line: 30,
        reason: "expected Format, found BadFormat".to_string(),
    });
    assert_eq!(format_line_error.line_number(), Some(30));

    let field_count_error = CoreError::Parse(ParseError::FieldCountMismatch {
        line: 50,
        expected: 5,
        found: 3,
    });
    assert_eq!(field_count_error.line_number(), Some(50));

    let time_format_error = CoreError::Parse(ParseError::InvalidTimeFormat {
        line: 60,
        time: "bad_time".to_string(),
        reason: "invalid format".to_string(),
    });
    assert_eq!(time_format_error.line_number(), Some(60));

    let color_format_error = CoreError::Parse(ParseError::InvalidColorFormat {
        line: 70,
        color: "bad_color".to_string(),
        reason: "invalid format".to_string(),
    });
    assert_eq!(color_format_error.line_number(), Some(70));

    let numeric_error = CoreError::Parse(ParseError::InvalidNumericValue {
        line: 80,
        value: "bad_number".to_string(),
        reason: "invalid number".to_string(),
    });
    assert_eq!(numeric_error.line_number(), Some(80));

    let style_override_error = CoreError::Parse(ParseError::InvalidStyleOverride {
        line: 90,
        reason: "bad_tag".to_string(),
    });
    assert_eq!(style_override_error.line_number(), Some(90));

    let drawing_error = CoreError::Parse(ParseError::InvalidDrawingCommand {
        line: 95,
        reason: "bad_command".to_string(),
    });
    assert_eq!(drawing_error.line_number(), Some(95));

    let decode_error = CoreError::Parse(ParseError::UuDecodeError {
        line: 85,
        reason: "decode failed".to_string(),
    });
    assert_eq!(decode_error.line_number(), Some(85));

    let nesting_error = CoreError::Parse(ParseError::MaxNestingDepth {
        line: 75,
        limit: 100,
    });
    assert_eq!(nesting_error.line_number(), Some(75));

    let internal_parse_error = CoreError::Parse(ParseError::InternalError {
        line: 65,
        message: "internal".to_string(),
    });
    assert_eq!(internal_parse_error.line_number(), Some(65));

    // UTF-8 error with position
    let utf8_error = CoreError::Utf8Error {
        position: 123,
        message: "invalid".to_string(),
    };
    assert_eq!(utf8_error.line_number(), Some(123));

    // Errors without line numbers
    let tokenization_error = CoreError::Tokenization("test".to_string());
    assert_eq!(tokenization_error.line_number(), None);

    let analysis_error = CoreError::Analysis("test".to_string());
    assert_eq!(analysis_error.line_number(), None);
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

#[test]
fn error_equality_and_cloning() {
    let error1 = CoreError::Tokenization("test".to_string());
    let error2 = CoreError::Tokenization("test".to_string());
    let error3 = CoreError::Tokenization("different".to_string());

    assert_eq!(error1, error2);
    assert_ne!(error1, error3);

    let cloned = error1.clone();
    assert_eq!(error1, cloned);
}

#[test]
fn error_debug_formatting() {
    let error = CoreError::Analysis("debug test".to_string());
    let debug_output = format!("{error:?}");
    assert!(debug_output.contains("Analysis"));
    assert!(debug_output.contains("debug test"));
}

#[test]
fn result_type_alias() {
    fn test_function() -> i32 {
        42
    }

    fn test_error_function() -> Result<i32> {
        Err(CoreError::Analysis("test error".to_string()))
    }

    assert_eq!(test_function(), 42);
    assert!(test_error_function().is_err());
}

#[cfg(feature = "std")]
#[test]
fn std_error_trait() {
    use std::error::Error;

    let error = CoreError::Analysis("test".to_string());
    assert!(error.source().is_none());

    // Test that it implements the Error trait
    let _: &dyn Error = &error;
}

#[cfg(not(feature = "std"))]
#[test]
fn nostd_error_trait() {
    use core::error::Error;
    let error = CoreError::Analysis("test".to_string());

    // Test that it implements the core Error trait in no_std
    let _: &dyn Error = &error;
}

#[test]
fn complex_error_scenarios() {
    // Test nested parse errors
    let parse_error = ParseError::FieldCountMismatch {
        line: 42,
        expected: 10,
        found: 5,
    };
    let core_error = CoreError::Parse(parse_error);

    assert!(!core_error.is_internal_bug());
    assert!(core_error.is_recoverable());
    assert_eq!(core_error.line_number(), Some(42));
    assert!(core_error.is_parse_error_type("field_format"));

    // Test resource limit error
    let resource_error = CoreError::ResourceLimitExceeded {
        resource: "memory".to_string(),
        current: 1024,
        limit: 512,
    };

    assert!(!resource_error.is_recoverable());
    assert!(!resource_error.is_internal_bug());
    assert!(resource_error.line_number().is_none());

    // Test feature not supported
    let feature_error = CoreError::FeatureNotSupported {
        feature: "wasm_simd".to_string(),
        required_feature: "simd".to_string(),
    };

    assert!(feature_error.is_recoverable());
    assert!(!feature_error.is_internal_bug());
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
