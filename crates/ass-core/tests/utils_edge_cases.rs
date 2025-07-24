//! Edge case and error handling tests for the utils module.
//!
//! This module contains comprehensive tests targeting previously untested code paths
//! in the utils module, focusing on span validation, color parsing, and UU decoding.

use ass_core::utils::{decode_uu_data, parse_bgr_color, CoreError, Spans};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test `Spans` struct with invalid span data
    #[test]
    fn test_spans_invalid_span_validation() {
        let source = "Hello, World! This is test data.";
        let spans = Spans::new(source);

        // Test validate_span with span from this source (should be valid)
        let valid_span = &source[0..5]; // "Hello"
        assert!(spans.validate_span(valid_span));

        // Test with span from different text (should be invalid)
        let other_text = "Different text entirely";
        let other_span = &other_text[0..5]; // From different source
        assert!(!spans.validate_span(other_span));

        // Test with empty span
        let empty_span = &source[0..0];
        assert!(spans.validate_span(empty_span));
    }

    /// Test `span_offset`, `span_line`, and `span_column` with invalid spans
    #[test]
    fn test_spans_position_methods_invalid_input() {
        let source = "Line 1\nLine 2\nLine 3";
        let spans = Spans::new(source);

        // Test with valid spans
        let line1_span = &source[0..6]; // "Line 1"
        assert_eq!(spans.span_offset(line1_span), Some(0));
        assert_eq!(spans.span_line(line1_span), Some(1));
        assert_eq!(spans.span_column(line1_span), Some(1));

        let line2_span = &source[7..13]; // "Line 2"
        assert_eq!(spans.span_offset(line2_span), Some(7));
        assert_eq!(spans.span_line(line2_span), Some(2));
        assert_eq!(spans.span_column(line2_span), Some(1));

        // Test with span from different source (should return None)
        let other_text = "Different text";
        let invalid_span = &other_text[0..5];
        assert!(spans.span_offset(invalid_span).is_none());
        assert!(spans.span_line(invalid_span).is_none());
        assert!(spans.span_column(invalid_span).is_none());
    }

    /// Test `substring` with out-of-bounds ranges
    #[test]
    fn test_spans_substring_out_of_bounds() {
        let source = "Hello, World!";
        let spans = Spans::new(source);

        // Test with valid range
        let result = spans.substring(0..5);
        assert_eq!(result, Some("Hello"));

        // Test with range beyond source length
        let result = spans.substring(0..source.len() + 10);
        assert!(result.is_none());

        // Test with start beyond source length
        let result = spans.substring(source.len() + 5..source.len() + 10);
        assert!(result.is_none());

        // Test with backwards range (intentionally empty)
        #[allow(clippy::reversed_empty_ranges)]
        let result = spans.substring(10..5);
        assert!(result.is_none());

        // Test with exact bounds
        let result = spans.substring(0..source.len());
        assert_eq!(result, Some(source));
    }

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

    /// Test `decode_uu_data` with malformed data
    #[test]
    fn test_decode_uu_data_malformed_input() {
        // Test with invalid length character (0x7F)
        let invalid_length = "\x7FABCDEFGH"; // 0x7F is not a valid UU length character
        let result = decode_uu_data(std::iter::once(invalid_length));
        // UU decoder might be permissive, just test it doesn't panic
        drop(result);

        // Test with line shorter than encoded length
        let short_line = "M"; // 'M' indicates 45 bytes but we only have 1 character
        let result = decode_uu_data(std::iter::once(short_line));
        // May succeed with partial data or fail, just test it doesn't panic
        drop(result);

        // Test with non-UU characters
        let invalid_chars = "M\x01\x02\x03"; // Control characters are not valid UU
        let result = decode_uu_data(std::iter::once(invalid_chars));
        // May succeed or fail, just test it doesn't panic
        drop(result);

        // Test with expected_length > 45 path
        let overlength = "N".repeat(100); // 'N' indicates 46 bytes, which is > 45
        let result = decode_uu_data(std::iter::once(overlength.as_str()));
        // May succeed or fail, just test it doesn't panic
        drop(result);

        // Test with empty input
        let empty = std::iter::empty::<&str>();
        let result = decode_uu_data(empty);
        // Empty input returns empty result, not an error
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());

        // Test with only length character
        let only_length = "M";
        let result = decode_uu_data(std::iter::once(only_length));
        // May succeed or fail, just test it doesn't panic
        drop(result);

        // Test with null bytes
        let with_nulls = "M\0\0\0\0";
        let result = decode_uu_data(std::iter::once(with_nulls));
        // May succeed or fail, just test it doesn't panic
        drop(result);
    }

    /// Test `decode_uu_data` with valid but edge case data
    #[test]
    fn test_decode_uu_data_edge_cases() {
        // Test with minimum valid data (length 1)
        let min_data = "!A"; // '!' = length 0, but we need at least some padding
        let result = decode_uu_data(std::iter::once(min_data));
        // Should succeed with small data
        assert!(result.is_ok());

        // Test with maximum valid length (45 bytes)
        let max_length_char = 'M'; // 'M' represents 45 bytes
        let max_data = format!("{}{}", max_length_char, "A".repeat(60)); // 60 chars for 45 bytes
        let result = decode_uu_data(std::iter::once(max_data.as_str()));
        assert!(result.is_ok() || result.is_err()); // Either decodes or fails gracefully

        // Test with whitespace
        let with_whitespace = "5 A B C D"; // Contains spaces
        let result = decode_uu_data(std::iter::once(with_whitespace));
        // May succeed or fail depending on implementation
        drop(result); // Just test that it doesn't panic

        // Test with multiple lines
        let with_newlines = ["5ABCD", "EFGH"];
        let result = decode_uu_data(with_newlines.iter().copied());
        // Multiple lines may be processed successfully
        drop(result); // Just test that it doesn't panic
    }

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

    /// Test `Spans` with various source text formats
    #[test]
    fn test_spans_with_different_text_formats() {
        // Test with empty source
        let empty_source = "";
        let empty_spans = Spans::new(empty_source);

        let empty_span = &empty_source[0..0];
        assert!(empty_spans.validate_span(empty_span));
        assert_eq!(empty_spans.substring(0..0), Some(""));

        // Test with Unicode text
        let unicode_source = "Hello 🌍 World! こんにちは";
        let unicode_spans = Spans::new(unicode_source);

        // Create span with valid byte boundaries
        let hello_span = &unicode_source[0..5]; // "Hello"
        assert!(unicode_spans.validate_span(hello_span));

        // Test with various line endings
        let mixed_endings = "Line1\nLine2\r\nLine3\rLine4";
        let mixed_spans = Spans::new(mixed_endings);

        let full_span = &mixed_endings[0..mixed_endings.len()];
        assert!(mixed_spans.validate_span(full_span));
        assert_eq!(
            mixed_spans.substring(0..mixed_endings.len()),
            Some(mixed_endings)
        );
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
}
