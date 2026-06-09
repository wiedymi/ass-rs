//! Memory, timing, error-conversion, and integration tests for the utils module.

use crate::utils::{
    check_memory_usage, escape_text, get_timestamp, normalize_field_name,
    normalize_line_endings, parse_color_string, parse_numeric_string, parse_time_string,
    trim_ass_whitespace, unescape_text, validate_text_content, CoreError,
};
#[cfg(not(feature = "std"))]
use alloc::{format, vec::Vec};

#[test]
fn memory_limit_checking() {
    assert!(check_memory_usage(1000, 2000).is_ok());
    assert!(check_memory_usage(1000, 1000).is_ok());
    assert!(check_memory_usage(2000, 1000).is_err());
}

#[test]
fn performance_measurement() {
    let start = get_timestamp();
    // Small operation
    let _ = format!("test");
    let end = get_timestamp();

    assert!(end >= start);
    assert!(end - start < 1000); // Should be less than 1ms
}

#[test]
fn error_conversion_chain() {
    let parse_error = "not_a_number".parse::<i32>().unwrap_err();
    let core_error: CoreError = parse_error.into();

    assert!(matches!(core_error, CoreError::InvalidNumeric(_)));
    assert!(core_error.is_recoverable());
    assert!(!core_error.is_internal_bug());
}

#[test]
fn utility_function_integration() {
    // Test combination of multiple utility functions
    let input = "  Title:  Test Script  \r\n";
    let normalized = normalize_line_endings(input);
    let trimmed = trim_ass_whitespace(&normalized);
    let parts: Vec<&str> = trimmed.split(':').collect();

    assert_eq!(parts.len(), 2);
    assert_eq!(normalize_field_name(parts[0]), "title");
    assert_eq!(trim_ass_whitespace(parts[1]), "Test Script");
}

#[test]
fn edge_case_empty_inputs() {
    assert!(parse_time_string("").is_err());
    assert!(parse_color_string("").is_err());
    assert!(parse_numeric_string("").is_err());
    assert!(validate_text_content("").is_ok());
    assert_eq!(normalize_field_name(""), "");
    assert_eq!(trim_ass_whitespace(""), "");
    assert_eq!(unescape_text(""), "");
    assert_eq!(escape_text(""), "");
}

#[test]
fn large_input_handling() {
    let large_string = "a".repeat(10000);
    assert!(validate_text_content(&large_string).is_ok());

    let large_field = "field".repeat(1000);
    let normalized = normalize_field_name(&large_field);
    assert_eq!(normalized.len(), large_field.len());

    // Test memory limits
    assert!(check_memory_usage(5000, 10000).is_ok());
    assert!(check_memory_usage(15000, 10000).is_err());
}
