//! Comprehensive tests for utils module functionality

use super::*;
use alloc::{format, string::String, vec::Vec};

#[test]
fn time_parsing_valid_formats() {
    // Test standard H:MM:SS.CC format
    assert!(parse_time_string("0:00:30.50").is_ok());
    assert!(parse_time_string("1:23:45.67").is_ok());
    assert!(parse_time_string("12:34:56.78").is_ok());

    // Test single digit components
    assert!(parse_time_string("0:01:02.03").is_ok());

    // Test edge cases
    assert!(parse_time_string("0:00:00.00").is_ok());
    assert!(parse_time_string("23:59:59.99").is_ok());
}

#[test]
fn time_parsing_invalid_formats() {
    // Missing components
    assert!(parse_time_string("").is_err());
    assert!(parse_time_string("1:23").is_err());
    assert!(parse_time_string("1:23:45").is_err());

    // Invalid characters
    assert!(parse_time_string("a:23:45.67").is_err());
    assert!(parse_time_string("1:ab:45.67").is_err());
    assert!(parse_time_string("1:23:cd.67").is_err());
    assert!(parse_time_string("1:23:45.ef").is_err());

    // Out of range values
    assert!(parse_time_string("1:60:45.67").is_err());
    assert!(parse_time_string("1:23:60.67").is_err());
    assert!(parse_time_string("1:23:45.100").is_err());
}

#[test]
fn time_parsing_precision() {
    let result = parse_time_string("1:23:45.67").unwrap();
    assert_eq!(result.hours, 1);
    assert_eq!(result.minutes, 23);
    assert_eq!(result.seconds, 45);
    assert_eq!(result.centiseconds, 67);
}

#[test]
fn color_parsing_hex_formats() {
    // Standard hex colors
    assert!(parse_color_string("#FF0000").is_ok());
    assert!(parse_color_string("#00FF00").is_ok());
    assert!(parse_color_string("#0000FF").is_ok());

    // ASS format colors
    assert!(parse_color_string("&H0000FF&").is_ok());
    assert!(parse_color_string("&HFF0000&").is_ok());
    assert!(parse_color_string("&H00FF00&").is_ok());

    // Without alpha
    assert!(parse_color_string("FF0000").is_ok());
    assert!(parse_color_string("00FF00").is_ok());

    // With alpha
    assert!(parse_color_string("&H80FF0000&").is_ok());
}

#[test]
fn color_parsing_invalid_formats() {
    // Invalid hex characters
    assert!(parse_color_string("&HGGGGGG&").is_err());
    assert!(parse_color_string("#ZZZZZZ").is_err());

    // Wrong length
    assert!(parse_color_string("&HFF&").is_err());
    assert!(parse_color_string("#FF").is_err());
    assert!(parse_color_string("FFFFFFF").is_err());

    // Malformed ASS format
    assert!(parse_color_string("&HFF0000").is_err());
    assert!(parse_color_string("FF0000&").is_err());
}

#[test]
fn numeric_parsing_integers() {
    assert_eq!(parse_numeric_string("0").unwrap(), NumericValue::Integer(0));
    assert_eq!(
        parse_numeric_string("42").unwrap(),
        NumericValue::Integer(42)
    );
    assert_eq!(
        parse_numeric_string("-123").unwrap(),
        NumericValue::Integer(-123)
    );
    assert_eq!(
        parse_numeric_string("1000").unwrap(),
        NumericValue::Integer(1000)
    );
}

#[test]
fn numeric_parsing_floats() {
    assert_eq!(
        parse_numeric_string("3.14").unwrap(),
        NumericValue::Float(3.14)
    );
    assert_eq!(
        parse_numeric_string("-2.5").unwrap(),
        NumericValue::Float(-2.5)
    );
    assert_eq!(
        parse_numeric_string("0.0").unwrap(),
        NumericValue::Float(0.0)
    );
}

#[test]
fn numeric_parsing_invalid() {
    assert!(parse_numeric_string("").is_err());
    assert!(parse_numeric_string("abc").is_err());
    assert!(parse_numeric_string("12.34.56").is_err());
    assert!(parse_numeric_string("1.2.3").is_err());
    assert!(parse_numeric_string("not_a_number").is_err());
}

#[test]
fn text_validation_ascii() {
    assert!(validate_text_content("Hello World").is_ok());
    assert!(validate_text_content("1234567890").is_ok());
    assert!(validate_text_content("!@#$%^&*()").is_ok());
}

#[test]
fn text_validation_unicode() {
    assert!(validate_text_content("こんにちは").is_ok());
    assert!(validate_text_content("Здравствуй").is_ok());
    assert!(validate_text_content("مرحبا").is_ok());
    assert!(validate_text_content("🎵🎬🎭").is_ok());
}

#[test]
fn text_validation_control_chars() {
    // Control characters should be flagged
    let result = validate_text_content("Hello\x00World");
    assert!(result.is_err());

    let result = validate_text_content("Text\x1FTest");
    assert!(result.is_err());
}

#[test]
fn text_validation_line_endings() {
    // Line endings should be allowed
    assert!(validate_text_content("Line1\nLine2").is_ok());
    assert!(validate_text_content("Line1\r\nLine2").is_ok());
    assert!(validate_text_content("Line1\rLine2").is_ok());
}

#[test]
fn text_sanitization() {
    let input = "Hello\x00\x1F World";
    let result = sanitize_text_content(input);
    assert_eq!(result, "Hello World");

    let input = "Good\x0BText\x0CTest";
    let result = sanitize_text_content(input);
    assert_eq!(result, "GoodTextTest");
}

#[test]
fn field_name_normalization() {
    assert_eq!(normalize_field_name("Title"), "title");
    assert_eq!(normalize_field_name("ScriptType"), "scripttype");
    assert_eq!(normalize_field_name("WrapStyle"), "wrapstyle");
    assert_eq!(normalize_field_name("UPPERCASE"), "uppercase");
    assert_eq!(normalize_field_name("MixedCase"), "mixedcase");
}

#[test]
fn field_name_normalization_whitespace() {
    assert_eq!(normalize_field_name(" Title "), "title");
    assert_eq!(normalize_field_name("\tScriptType\t"), "scripttype");
    assert_eq!(normalize_field_name("\nWrapStyle\n"), "wrapstyle");
}

#[test]
fn escape_sequence_parsing() {
    assert_eq!(parse_escape_sequence("\\n"), Some('\n'));
    assert_eq!(parse_escape_sequence("\\r"), Some('\r'));
    assert_eq!(parse_escape_sequence("\\t"), Some('\t'));
    assert_eq!(parse_escape_sequence("`[Events]`"), Some('\\'));
    assert_eq!(parse_escape_sequence("\\{"), Some('{'));
    assert_eq!(parse_escape_sequence("\\}"), Some('}'));
}

#[test]
fn escape_sequence_parsing_invalid() {
    assert_eq!(parse_escape_sequence("\\x"), None);
    assert_eq!(parse_escape_sequence("\\z"), None);
    assert_eq!(parse_escape_sequence("n"), None);
    assert_eq!(parse_escape_sequence(""), None);
}

#[test]
fn unescape_text_basic() {
    assert_eq!(unescape_text("Hello\\nWorld"), "Hello\nWorld");
    assert_eq!(unescape_text("Tab\\tSeparated"), "Tab\tSeparated");
    assert_eq!(unescape_text("Quote\\\"Test"), "Quote\"Test");
    assert_eq!(unescape_text("Brace\\{Test\\}"), "Brace{Test}");
}

#[test]
fn unescape_text_multiple() {
    assert_eq!(
        unescape_text("Line1\\nLine2\\nLine3"),
        "Line1\nLine2\nLine3"
    );
    assert_eq!(unescape_text("\\t\\r\\n"), "\t\r\n");
}

#[test]
fn unescape_text_no_escapes() {
    assert_eq!(unescape_text("Plain text"), "Plain text");
    assert_eq!(unescape_text("No escapes here"), "No escapes here");
    assert_eq!(unescape_text(""), "");
}

#[test]
fn escape_text_basic() {
    assert_eq!(escape_text("Hello\nWorld"), "Hello\\nWorld");
    assert_eq!(escape_text("Tab\tSeparated"), "Tab\\tSeparated");
    assert_eq!(escape_text("Quote\"Test"), "Quote\\\"Test");
    assert_eq!(escape_text("Brace{Test}"), "Brace\\{Test\\}");
}

#[test]
fn escape_unescape_round_trip() {
    let original = "Hello\nWorld\tWith\"Quotes{And}Braces";
    let escaped = escape_text(original);
    let unescaped = unescape_text(&escaped);
    assert_eq!(original, unescaped);
}

#[test]
fn percentage_parsing() {
    assert_eq!(parse_percentage("50%").unwrap(), 0.5);
    assert_eq!(parse_percentage("100%").unwrap(), 1.0);
    assert_eq!(parse_percentage("0%").unwrap(), 0.0);
    assert_eq!(parse_percentage("25.5%").unwrap(), 0.255);
}

#[test]
fn percentage_parsing_invalid() {
    assert!(parse_percentage("50").is_err()); // Missing %
    assert!(parse_percentage("%50").is_err()); // % at start
    assert!(parse_percentage("abc%").is_err()); // Invalid number
    assert!(parse_percentage("").is_err()); // Empty
}

#[test]
fn boolean_parsing() {
    assert_eq!(parse_boolean_value("1").unwrap(), true);
    assert_eq!(parse_boolean_value("-1").unwrap(), true);
    assert_eq!(parse_boolean_value("0").unwrap(), false);
    assert_eq!(parse_boolean_value("true").unwrap(), true);
    assert_eq!(parse_boolean_value("false").unwrap(), false);
    assert_eq!(parse_boolean_value("yes").unwrap(), true);
    assert_eq!(parse_boolean_value("no").unwrap(), false);
}

#[test]
fn boolean_parsing_case_insensitive() {
    assert_eq!(parse_boolean_value("TRUE").unwrap(), true);
    assert_eq!(parse_boolean_value("False").unwrap(), false);
    assert_eq!(parse_boolean_value("YES").unwrap(), true);
    assert_eq!(parse_boolean_value("No").unwrap(), false);
}

#[test]
fn boolean_parsing_invalid() {
    assert!(parse_boolean_value("maybe").is_err());
    assert!(parse_boolean_value("2").is_err());
    assert!(parse_boolean_value("").is_err());
    assert!(parse_boolean_value("on").is_err());
    assert!(parse_boolean_value("off").is_err());
}

#[test]
fn memory_limit_checking() {
    assert!(check_memory_usage(1000, 2000).is_ok());
    assert!(check_memory_usage(1000, 1000).is_ok());
    assert!(check_memory_usage(2000, 1000).is_err());
}

#[test]
fn string_trimming() {
    assert_eq!(trim_ass_whitespace("  hello  "), "hello");
    assert_eq!(trim_ass_whitespace("\thello\t"), "hello");
    assert_eq!(trim_ass_whitespace("\nhello\n"), "hello");
    assert_eq!(trim_ass_whitespace(" \t\nhello\n\t "), "hello");
}

#[test]
fn string_trimming_unicode() {
    assert_eq!(trim_ass_whitespace("  こんにちは  "), "こんにちは");
    assert_eq!(trim_ass_whitespace("\t🎵\t"), "🎵");
}

#[test]
fn string_splitting_csv() {
    let result = split_csv_line("a,b,c");
    assert_eq!(result, vec!["a", "b", "c"]);

    let result = split_csv_line("field1, field2 , field3");
    assert_eq!(result, vec!["field1", "field2", "field3"]);
}

#[test]
fn string_splitting_csv_empty() {
    let result = split_csv_line("");
    assert_eq!(result, vec![""]);

    let result = split_csv_line(",");
    assert_eq!(result, vec!["", ""]);

    let result = split_csv_line("a,,c");
    assert_eq!(result, vec!["a", "", "c"]);
}

#[test]
fn string_splitting_csv_quoted() {
    let result = split_csv_line("\"quoted field\",normal,\"another quoted\"");
    assert_eq!(result, vec!["quoted field", "normal", "another quoted"]);

    let result = split_csv_line("\"field with, comma\",other");
    assert_eq!(result, vec!["field with, comma", "other"]);
}

#[test]
fn line_ending_normalization() {
    assert_eq!(normalize_line_endings("line1\r\nline2"), "line1\nline2");
    assert_eq!(normalize_line_endings("line1\rline2"), "line1\nline2");
    assert_eq!(normalize_line_endings("line1\nline2"), "line1\nline2");

    let mixed = "line1\r\nline2\rline3\nline4";
    assert_eq!(normalize_line_endings(mixed), "line1\nline2\nline3\nline4");
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
