//! Tests for line-number metadata extraction and combined error scenarios.

use crate::parser::ParseError;
use crate::utils::errors::CoreError;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

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
