//! Tests for field value parsing, validation, and normalization.

use super::*;

#[test]
fn validate_ass_names() {
    assert!(validate_ass_name("Default"));
    assert!(validate_ass_name("MyStyle"));
    assert!(validate_ass_name("Style with spaces"));

    assert!(!validate_ass_name("")); // Empty
    assert!(!validate_ass_name("Style,Name")); // Comma
    assert!(!validate_ass_name("Style:Name")); // Colon
    assert!(!validate_ass_name("Style{Name")); // Brace
    assert!(!validate_ass_name("Style\nName")); // Control character
}

#[test]
fn normalize_field_values() {
    assert_eq!(normalize_field_value("  value  "), "value");
    assert_eq!(normalize_field_value("\tvalue\t"), "value");
    assert_eq!(normalize_field_value("value"), "value");
}

#[test]
fn numeric_parsing() {
    assert_eq!(parse_numeric::<i32>("42").unwrap(), 42);
    assert!((parse_numeric::<f32>("3.15").unwrap() - 3.15).abs() < f32::EPSILON);
    assert!(parse_numeric::<i32>("invalid").is_err());
}

#[test]
fn validate_ass_name_edge_cases() {
    // Test with tab character (should be allowed)
    assert!(validate_ass_name("Style\tName"));

    // Test with various control characters (should be rejected)
    assert!(!validate_ass_name("Style\nName")); // Newline
    assert!(!validate_ass_name("Style\rName")); // Carriage return
    assert!(!validate_ass_name("Style\x00Name")); // Null
    assert!(!validate_ass_name("Style\x7FName")); // DEL

    // Test edge cases with separators
    assert!(!validate_ass_name(",Style")); // Leading comma
    assert!(!validate_ass_name("Style,")); // Trailing comma
    assert!(!validate_ass_name(":Style")); // Leading colon
    assert!(!validate_ass_name("Style:")); // Trailing colon
    assert!(!validate_ass_name("{Style")); // Leading brace
    assert!(!validate_ass_name("Style}")); // Trailing brace

    // Test very long names
    let long_name = "a".repeat(1000);
    assert!(validate_ass_name(&long_name));

    // Test Unicode characters
    assert!(validate_ass_name("Style中文"));
    assert!(validate_ass_name("Style🎭"));
    assert!(validate_ass_name("Стиль"));
}

#[test]
fn normalize_field_value_edge_cases() {
    // Test empty string
    assert_eq!(normalize_field_value(""), "");

    // Test only whitespace
    assert_eq!(normalize_field_value("   "), "");
    assert_eq!(normalize_field_value("\t\t\t"), "");
    assert_eq!(normalize_field_value(" \t \t "), "");

    // Test mixed whitespace
    assert_eq!(normalize_field_value(" \t value \t "), "value");
    assert_eq!(normalize_field_value("\n\rvalue\n\r"), "value");

    // Test internal whitespace preservation
    assert_eq!(normalize_field_value("  val ue  "), "val ue");
    assert_eq!(normalize_field_value("  val\tue  "), "val\tue");
}

#[test]
#[allow(clippy::float_cmp, clippy::approx_constant)]
fn parse_numeric_edge_cases() {
    // Test boundary values for different types
    assert_eq!(parse_numeric::<u8>("255").unwrap(), 255u8);
    assert!(parse_numeric::<u8>("256").is_err());
    assert_eq!(parse_numeric::<i8>("127").unwrap(), 127i8);
    assert_eq!(parse_numeric::<i8>("-128").unwrap(), -128i8);
    assert!(parse_numeric::<i8>("128").is_err());

    // Test floating point edge cases
    assert_eq!(parse_numeric::<f32>("0.0").unwrap(), 0.0f32);
    assert_eq!(parse_numeric::<f32>("-0.0").unwrap(), -0.0f32);
    assert!(parse_numeric::<f32>("inf").is_ok());
    assert!(parse_numeric::<f32>("-inf").is_ok());

    // Test whitespace handling
    assert_eq!(parse_numeric::<i32>("  42  ").unwrap(), 42i32);
    assert_eq!(parse_numeric::<f32>(" \t 3.14 \t ").unwrap(), 3.14f32);

    // Test leading zeros
    assert_eq!(parse_numeric::<i32>("00042").unwrap(), 42i32);
    assert_eq!(parse_numeric::<f32>("0003.140").unwrap(), 3.14f32);

    // Test scientific notation
    assert_eq!(parse_numeric::<f32>("1e2").unwrap(), 100.0f32);
    assert_eq!(parse_numeric::<f32>("1.5e-2").unwrap(), 0.015f32);

    // Test invalid formats
    assert!(parse_numeric::<i32>("").is_err());
    assert!(parse_numeric::<i32>("abc").is_err());
    assert!(parse_numeric::<i32>("12.34").is_err()); // Float for int
    assert!(parse_numeric::<f32>("12.34.56").is_err()); // Multiple dots
}
