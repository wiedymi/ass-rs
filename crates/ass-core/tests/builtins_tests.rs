use ass_core::builtins::*;

// Note: This tests the implemented builtin utility functions
// for ASS script processing and manipulation

#[test]
fn test_time_functions() {
    // Test time parsing and formatting functions
    let time_ms = parse_time("0:01:30.50");
    assert!(time_ms.is_ok());
    assert_eq!(time_ms.unwrap(), 90500);

    let formatted = format_time(90500); // 1:30.50 in milliseconds
    assert!(!formatted.is_empty());
}

#[test]
fn test_time_edge_cases() {
    // Test edge cases for time functions
    assert!(parse_time("").is_err());
    assert!(parse_time("invalid").is_err());
    assert!(parse_time("25:00:00.00").is_ok()); // Should handle hours > 24

    // Test zero time
    let zero_formatted = format_time(0);
    assert!(!zero_formatted.is_empty());

    // Test large time values
    let large_formatted = format_time(u64::MAX);
    assert!(!large_formatted.is_empty());
}

#[test]
fn test_color_functions() {
    // Test color parsing and conversion
    let color = parse_color("&H0000FF&"); // Red in ASS format (BBGGRR)
    assert!(color.is_ok());

    let formatted = format_color(0xFF0000); // Red in RGB
    assert!(!formatted.is_empty());
}

#[test]
fn test_color_edge_cases() {
    // Test various color formats
    assert!(parse_color("&HFF0000&").is_ok());
    assert!(parse_color("&H00FF0000&").is_ok());
    assert!(parse_color("").is_err());
    assert!(parse_color("invalid").is_err());
    assert!(parse_color("&HGGGGGG&").is_err()); // Invalid hex
}

#[test]
fn test_string_functions() {
    // Test string manipulation functions
    let trimmed = trim_whitespace("  hello world  ");
    assert_eq!(trimmed, "hello world");

    let replaced = replace_text("hello world", "world", "Rust");
    assert_eq!(replaced, "hello Rust");

    let uppercase = to_uppercase("hello");
    assert_eq!(uppercase, "HELLO");

    let lowercase = to_lowercase("WORLD");
    assert_eq!(lowercase, "world");
}

#[test]
fn test_string_edge_cases() {
    // Test edge cases for string functions
    assert_eq!(trim_whitespace(""), "");
    assert_eq!(trim_whitespace("   "), "");
    assert_eq!(trim_whitespace("no-whitespace"), "no-whitespace");

    assert_eq!(replace_text("", "find", "replace"), "");
    assert_eq!(replace_text("text", "", "replace"), "text");
    assert_eq!(replace_text("text", "notfound", "replace"), "text");

    assert_eq!(to_uppercase(""), "");
    assert_eq!(to_lowercase(""), "");
}

#[test]
fn test_math_functions() {
    // Test mathematical functions
    assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
    assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
    assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);

    assert!((degrees_to_radians(180.0) - std::f64::consts::PI).abs() < f64::EPSILON);
    assert!((radians_to_degrees(std::f64::consts::PI) - 180.0).abs() < f64::EPSILON);

    assert_eq!(clamp(5.0, 0.0, 10.0), 5.0);
    assert_eq!(clamp(-5.0, 0.0, 10.0), 0.0);
    assert_eq!(clamp(15.0, 0.0, 10.0), 10.0);
}

#[test]
fn test_math_edge_cases() {
    // Test edge cases for math functions
    assert!(clamp(f64::NAN, 0.0, 10.0).is_nan());
    assert_eq!(clamp(f64::INFINITY, 0.0, 10.0), 10.0);
    assert_eq!(clamp(f64::NEG_INFINITY, 0.0, 10.0), 0.0);

    let pi_test = degrees_to_radians(360.0);
    assert!((pi_test - 2.0 * std::f64::consts::PI).abs() < f64::EPSILON);
}

#[test]
fn test_unicode_string_functions() {
    // Test string functions with Unicode
    let unicode_text = "こんにちは世界";
    let formatted_unicode = format!("  {unicode_text}  ");
    let trimmed = trim_whitespace(&formatted_unicode);
    assert_eq!(trimmed, unicode_text);

    let replaced = replace_text("こんにちは世界", "世界", "Rust");
    assert_eq!(replaced, "こんにちはRust");

    // Test emoji handling
    let emoji_text = "Hello 🌍 World 🚀";
    let formatted_emoji = format!("  {emoji_text}  ");
    let trimmed_emoji = trim_whitespace(&formatted_emoji);
    assert_eq!(trimmed_emoji, emoji_text);
}

#[test]
fn test_conversion_functions() {
    // Test conversion and utility functions
    let (r, g, b) = get_color_components(0xFF8040);
    assert_eq!(r, 255);
    assert_eq!(g, 128);
    assert_eq!(b, 64);

    let color = create_color(255, 128, 64);
    assert_eq!(color, 0xFF8040);
}

#[test]
fn test_tag_parsing_functions() {
    // Test tag parsing and validation
    let params = parse_tag_parameters("10,20,30");
    assert!(params.is_ok());
    assert_eq!(params.unwrap(), vec!["10", "20", "30"]);

    assert!(is_valid_tag("bold"));
    assert!(is_valid_tag("b1"));
    assert!(!is_valid_tag(""));
    assert!(!is_valid_tag("tag with spaces"));

    let formatted = format_tag("  BOLD  ");
    assert_eq!(formatted, "bold");
}

#[test]
fn test_tag_parsing_edge_cases() {
    // Test edge cases for tag parsing
    assert!(parse_tag_parameters("").is_err());
    assert!(parse_tag_parameters("single").is_ok());

    assert!(!is_valid_tag("tag-with-dash"));
    assert!(is_valid_tag("tag_with_underscore"));
}

#[test]
fn test_validation_functions() {
    // Test validation functions
    assert!(is_valid_timestamp("0:01:30.50"));
    assert!(is_valid_timestamp("10:59:59.99"));
    assert!(!is_valid_timestamp("invalid"));
    assert!(!is_valid_timestamp(""));

    assert!(is_valid_section_name("Script Info"));
    assert!(is_valid_section_name("Events"));
    assert!(!is_valid_section_name(""));
    assert!(!is_valid_section_name("[Invalid]"));
}

#[test]
fn test_utility_functions() {
    // Test utility functions
    let normalized = normalize_line_endings("line1\r\nline2\rline3\nline4");
    assert_eq!(normalized, "line1\nline2\nline3\nline4");

    let escaped = escape_ass_text("text{with}braces\\and\\backslashes");
    assert!(escaped.contains(r"\{"));
    assert!(escaped.contains(r"\}"));

    let unescaped = unescape_ass_text(r"text\{with\}braces");
    assert!(unescaped.contains("{"));
    assert!(unescaped.contains("}"));
}
