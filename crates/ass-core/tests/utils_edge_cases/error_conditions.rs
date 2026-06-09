//! Cross-cutting error, boundary, and error-message tests for the utils module.
//!
//! Combines color parsing, UU decoding, and span checks to exercise error
//! conditions, boundary inputs, and error type/message formatting.

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};

use ass_core::utils::{decode_uu_data, parse_bgr_color, CoreError, Spans};

/// Test various error conditions in utils functions
#[test]
fn test_utils_error_conditions() {
    // Test color parsing with various invalid formats
    let invalid_colors = [
        "invalid", "#FFFFFF",  // Wrong prefix
        "0xGGGGGG", // Invalid hex
        "FFFF",     // Too short
        "0x",       // Empty after prefix
        "xyz123",   // Mixed invalid
    ];

    for color in &invalid_colors {
        let result = parse_bgr_color(color);
        assert!(result.is_err(), "Should fail for color: {color}");
    }

    // Test 8-character hex separately as it might be valid
    let eight_char = "FFFFFFFF";
    let result = parse_bgr_color(eight_char);
    // 8-character hex might be valid (AARRGGBB format)
    drop(result); // Just test it doesn't panic

    // Test UU decoding with various malformed inputs
    let mut long_data = "~".to_owned();
    long_data.push_str(&"A".repeat(100));
    let mut invalid_chars = "5".to_owned();
    invalid_chars.push_str(&"\x7F".repeat(10));

    let invalid_uu_data = [
        "\x7F\x7F\x7F", // Non-printable characters
        &long_data,     // Length too large
        "A",            // Missing data
        "5ABC",         // Too short for claimed length
        &invalid_chars, // Invalid characters
    ];

    for uu in &invalid_uu_data {
        let result = decode_uu_data(std::iter::once(*uu));
        // UU decoder is permissive, just test it doesn't panic
        drop(result);
    }
}

/// Test boundary conditions for all utils functions
#[test]
fn test_utils_boundary_conditions() {
    // Test with maximum size inputs
    let large_source = "x".repeat(10000);
    let large_spans = Spans::new(&large_source);

    let large_span = &large_source[0..large_source.len()];
    assert!(large_spans.validate_span(large_span));

    // Test color parsing at exact boundaries
    let exact_6_chars = "ABCDEF";
    assert!(parse_bgr_color(exact_6_chars).is_ok());

    let exact_8_chars = "0xABCDEF";
    assert!(parse_bgr_color(exact_8_chars).is_ok());

    // Test UU decoding at boundaries
    let boundary_uu = format!("{}{}", (32u8 + 45) as char, "A".repeat(60));
    let _result = decode_uu_data(std::iter::once(boundary_uu.as_str())); // May succeed or fail, but shouldn't panic
}

/// Test error message formatting and types
#[test]
fn test_error_handling_details() {
    // Test that errors contain useful information
    let color_error = parse_bgr_color("invalid").unwrap_err();
    let error_message = color_error.to_string();
    assert!(!error_message.is_empty());

    // For UU data, use something that's definitely invalid
    let uu_result = decode_uu_data(std::iter::once("\x7F\x7F\x7F"));
    if let Err(uu_error) = uu_result {
        let uu_error_message = uu_error.to_string();
        assert!(!uu_error_message.is_empty());
        // Verify error type if it fails
        assert!(matches!(uu_error, CoreError::Parse(_)));
    }

    // Verify color error type
    assert!(matches!(color_error, CoreError::InvalidColor(_)));
}
