//! Tests for ASS BGR color parsing.

use super::*;

#[test]
fn parse_bgr_colors() {
    assert_eq!(parse_bgr_color("&H000000FF&").unwrap(), [255, 0, 0, 0]);
    assert_eq!(parse_bgr_color("&H0000FF00&").unwrap(), [0, 255, 0, 0]);
    assert_eq!(parse_bgr_color("&H00FF0000&").unwrap(), [0, 0, 255, 0]);

    assert_eq!(parse_bgr_color("&HFF000000&").unwrap(), [0, 0, 0, 255]);

    assert_eq!(parse_bgr_color("0x000000FF").unwrap(), [255, 0, 0, 0]);
    assert_eq!(parse_bgr_color("000000FF").unwrap(), [255, 0, 0, 0]);
}

#[test]
fn parse_bgr_colors_invalid() {
    assert!(parse_bgr_color("invalid").is_err());
    assert!(parse_bgr_color("&HZZZZ&").is_err());
    assert!(parse_bgr_color("").is_err());
}

#[test]
fn parse_bgr_colors_without_trailing_ampersand() {
    assert_eq!(parse_bgr_color("&H000000FF").unwrap(), [255, 0, 0, 0]);
    assert_eq!(parse_bgr_color("&H00FFFFFF").unwrap(), [255, 255, 255, 0]);
    assert_eq!(parse_bgr_color("&H00000000").unwrap(), [0, 0, 0, 0]);
    assert_eq!(parse_bgr_color("&HFF000000").unwrap(), [0, 0, 0, 255]);
}

#[test]
fn parse_bgr_color_edge_cases() {
    // Test lowercase hex prefix
    assert_eq!(parse_bgr_color("&h000000").unwrap(), [0, 0, 0, 0]);
    assert_eq!(parse_bgr_color("&hFFFFFF").unwrap(), [255, 255, 255, 0]);

    // Test 0x prefix
    assert_eq!(parse_bgr_color("0x000000").unwrap(), [0, 0, 0, 0]);
    assert_eq!(parse_bgr_color("0xFFFFFF").unwrap(), [255, 255, 255, 0]);

    // Test plain hex without prefix
    assert_eq!(parse_bgr_color("000000").unwrap(), [0, 0, 0, 0]);
    assert_eq!(parse_bgr_color("FFFFFF").unwrap(), [255, 255, 255, 0]);

    // Test with extra whitespace
    assert_eq!(parse_bgr_color("  &H000000  ").unwrap(), [0, 0, 0, 0]);
    assert_eq!(parse_bgr_color("\t&H000000\t").unwrap(), [0, 0, 0, 0]);

    // Test with trailing ampersand variations
    assert_eq!(parse_bgr_color("&H000000&").unwrap(), [0, 0, 0, 0]);
    assert_eq!(parse_bgr_color("&h000000&").unwrap(), [0, 0, 0, 0]);

    // Test mixed case hex digits
    assert_eq!(parse_bgr_color("&HaAbBcC").unwrap(), [204, 187, 170, 0]);
    assert_eq!(parse_bgr_color("&HFFaaBBcc").unwrap(), [204, 187, 170, 255]);

    // Test invalid lengths
    assert!(parse_bgr_color("&H00000").is_err()); // 5 chars
    assert!(parse_bgr_color("&H0000000").is_err()); // 7 chars
    assert!(parse_bgr_color("&H000000000").is_err()); // 9 chars

    // Test invalid characters in hex
    assert!(parse_bgr_color("&H00000G").is_err());
    assert!(parse_bgr_color("&H00Z000").is_err());

    // Test empty after prefix
    assert!(parse_bgr_color("&H").is_err());
    assert!(parse_bgr_color("0x").is_err());

    // Test malformed prefixes
    assert!(parse_bgr_color("&H000000X").is_err());
    assert!(parse_bgr_color("X&H000000").is_err());
}
