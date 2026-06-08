//! Unit tests for the validated style-field parsing helpers.

use super::parsing::{
    parse_bool_flag, parse_color_with_default, parse_float, parse_font_size, parse_percentage,
    parse_u16, parse_u8,
};

#[test]
fn color_parsing() {
    // ASS colors are in BGR format: &HAABBGGRR where AA=alpha, BB=blue, GG=green, RR=red
    assert_eq!(
        crate::utils::parse_bgr_color("&H000000FF").unwrap(),
        [255, 0, 0, 0]
    ); // Red: RR=FF
    assert_eq!(
        crate::utils::parse_bgr_color("&H0000FF00").unwrap(),
        [0, 255, 0, 0]
    ); // Green: GG=FF
    assert_eq!(
        crate::utils::parse_bgr_color("&H00FF0000").unwrap(),
        [0, 0, 255, 0]
    ); // Blue: BB=FF

    // Test case-insensitive prefix
    assert_eq!(
        crate::utils::parse_bgr_color("&h000000FF").unwrap(),
        [255, 0, 0, 0]
    ); // Red with lowercase h

    // Test 6-digit format (no alpha channel)
    assert_eq!(
        crate::utils::parse_bgr_color("&HFF0000").unwrap(),
        [0, 0, 255, 0]
    ); // Blue in 6-digit
    assert_eq!(
        crate::utils::parse_bgr_color("&H00FF00").unwrap(),
        [0, 255, 0, 0]
    ); // Green in 6-digit
    assert_eq!(
        crate::utils::parse_bgr_color("&H0000FF").unwrap(),
        [255, 0, 0, 0]
    ); // Red in 6-digit
}

#[test]
fn parse_font_size_edge_cases() {
    // Test invalid font sizes
    assert!(parse_font_size("-10").is_err()); // Negative
    assert!(parse_font_size("0").is_err()); // Zero
    assert!(parse_font_size("1001").is_err()); // Too large
    assert!(parse_font_size("abc").is_err()); // Non-numeric
    assert!(parse_font_size("").is_err()); // Empty

    // Test valid font sizes
    assert!(parse_font_size("1").is_ok());
    assert!(parse_font_size("72").is_ok());
    assert!(parse_font_size("1000").is_ok());
}

#[test]
fn parse_color_with_default_invalid_formats() {
    // Test invalid color formats
    assert!(parse_color_with_default("invalid").is_err());
    assert!(parse_color_with_default("&H").is_err());
    assert!(parse_color_with_default("&HZZZZZ").is_err());
    assert!(parse_color_with_default("12345G").is_err()); // Invalid hex character

    // Test empty string returns default
    let default_color = parse_color_with_default("").unwrap();
    assert_eq!(default_color, [255, 255, 255, 255]);

    // Test whitespace only returns default
    let whitespace_color = parse_color_with_default("   ").unwrap();
    assert_eq!(whitespace_color, [255, 255, 255, 255]);
}

#[test]
fn parse_bool_flag_invalid_values() {
    // Test invalid boolean flags
    assert!(parse_bool_flag("2").is_err());
    assert!(parse_bool_flag("-1").is_err());
    assert!(parse_bool_flag("true").is_err());
    assert!(parse_bool_flag("false").is_err());
    assert!(parse_bool_flag("yes").is_err());
    assert!(parse_bool_flag("no").is_err());
    assert!(parse_bool_flag("").is_err());

    // Test valid boolean flags
    assert!(!parse_bool_flag("0").unwrap());
    assert!(parse_bool_flag("1").unwrap());
}

#[test]
#[allow(clippy::float_cmp)]
fn parse_percentage_invalid_values() {
    // Test invalid percentages
    assert!(parse_percentage("-10").is_err()); // Negative
    assert!(parse_percentage("1001").is_err()); // Too large
    assert!(parse_percentage("abc").is_err()); // Non-numeric
    assert!(parse_percentage("").is_err()); // Empty

    // Test valid percentages
    assert_eq!(parse_percentage("0").unwrap(), 0.0);
    assert_eq!(parse_percentage("100").unwrap(), 100.0);
    assert_eq!(parse_percentage("1000").unwrap(), 1000.0);
}

#[test]
#[allow(clippy::float_cmp)]
fn parse_float_invalid_values() {
    assert!(parse_float("abc").is_err());
    assert!(parse_float("").is_err());
    assert!(parse_float("1.2.3").is_err());
    assert!(parse_float("1.2.3.4").is_err());
    assert!(parse_float("not_a_number").is_err());

    // Test valid floats
    assert_eq!(parse_float("0").unwrap(), 0.0);
    assert_eq!(parse_float("-10.5").unwrap(), -10.5);
    assert_eq!(parse_float("123.456").unwrap(), 123.456);
}

#[test]
fn parse_u8_invalid_values() {
    assert!(parse_u8("256").is_err()); // Too large
    assert!(parse_u8("-1").is_err()); // Negative
    assert!(parse_u8("abc").is_err()); // Non-numeric
    assert!(parse_u8("").is_err()); // Empty

    // Test valid u8 values
    assert_eq!(parse_u8("0").unwrap(), 0);
    assert_eq!(parse_u8("255").unwrap(), 255);
}

#[test]
fn parse_u16_invalid_values() {
    assert!(parse_u16("65536").is_err()); // Too large
    assert!(parse_u16("-1").is_err()); // Negative
    assert!(parse_u16("abc").is_err()); // Non-numeric
    assert!(parse_u16("").is_err()); // Empty

    // Test valid u16 values
    assert_eq!(parse_u16("0").unwrap(), 0);
    assert_eq!(parse_u16("65535").unwrap(), 65535);
}
