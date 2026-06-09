//! Edge case tests for BGR color parsing in the utils module.
//!
//! Exercises prefix handling, length validation, casing, and invalid input
//! for `parse_bgr_color`.

use ass_core::utils::parse_bgr_color;

/// Test `parse_bgr_color` with different prefixes and formats
#[test]
fn test_parse_bgr_color_edge_cases() {
    // Test with 0x prefix
    let hex_with_prefix = "0xFFFFFF";
    let result = parse_bgr_color(hex_with_prefix);
    assert!(result.is_ok());

    // Test with plain hex (no prefix)
    let plain_hex = "FF00FF";
    let result = parse_bgr_color(plain_hex);
    assert!(result.is_ok());

    // Test with invalid hex characters
    let invalid_hex = "GGHHII";
    let result = parse_bgr_color(invalid_hex);
    assert!(result.is_err());

    // Test with wrong length
    let short_hex = "FF";
    let result = parse_bgr_color(short_hex);
    assert!(result.is_err());

    let long_hex = "FFFFFFFFFF";
    let result = parse_bgr_color(long_hex);
    assert!(result.is_err());

    // Test with mixed case
    let mixed_case = "0xaBcDeF";
    let result = parse_bgr_color(mixed_case);
    assert!(result.is_ok());

    // Test with empty string
    let empty = "";
    let result = parse_bgr_color(empty);
    assert!(result.is_err());

    // Test with just prefix
    let just_prefix = "0x";
    let result = parse_bgr_color(just_prefix);
    assert!(result.is_err());

    // Test with special characters
    let with_special = "0xFF@FF";
    let result = parse_bgr_color(with_special);
    assert!(result.is_err());
}
