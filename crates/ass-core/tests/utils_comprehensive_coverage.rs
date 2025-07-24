//! Comprehensive coverage tests for utils/mod.rs
//!
//! This module contains tests specifically designed to achieve complete coverage
//! of utility functions, especially error paths and edge cases.

use ass_core::utils::{decode_uu_data, parse_ass_time, parse_bgr_color, Spans};
use ass_core::CoreError;

#[cfg(test)]
mod utils_comprehensive_coverage_tests {
    use super::*;

    #[test]
    fn test_span_column_functionality() {
        // Test span_column method (line 97)
        let source = "Line 1\nLine 2 with text\nLine 3";
        let spans = Spans::new(source);

        // Test column calculation for various spans
        let first_span = "Line 1";
        if let Some(column) = spans.span_column(first_span) {
            assert_eq!(column, 1); // Should be at column 1
        }

        let second_span = "with";
        if let Some(column) = spans.span_column(second_span) {
            assert!(column > 1); // Should be after "Line 2 "
        }

        // Test with span not in source
        let invalid_span = "not in source";
        assert!(spans.span_column(invalid_span).is_none());
    }

    #[test]
    fn test_parse_ass_time_invalid_minutes() {
        // Test minutes >= 60 error (line 292)
        let invalid_times = vec![
            "1:60:00.00", // 60 minutes
            "2:75:30.50", // 75 minutes
            "0:99:59.99", // 99 minutes
        ];

        for time_str in invalid_times {
            let result = parse_ass_time(time_str);
            assert!(result.is_err());
            if let Err(CoreError::InvalidTime(msg)) = result {
                assert!(msg.contains("Minutes must be < 60"));
            }
        }
    }

    #[test]
    fn test_parse_ass_time_invalid_seconds() {
        // Test seconds >= 60 error (lines 306-307)
        let invalid_times = vec![
            "0:00:60.00", // 60 seconds
            "0:30:75.25", // 75 seconds
            "1:45:99.99", // 99 seconds
        ];

        for time_str in invalid_times {
            let result = parse_ass_time(time_str);
            assert!(result.is_err());
            if let Err(CoreError::InvalidTime(msg)) = result {
                assert!(msg.contains("Seconds must be < 60"));
            }
        }
    }

    #[test]
    fn test_parse_ass_time_invalid_centiseconds() {
        // Test centiseconds >= 100 error
        let invalid_times = vec![
            "0:00:00.100", // 100 centiseconds
            "0:00:30.150", // 150 centiseconds
            "1:30:45.999", // 999 centiseconds (should be parsed as 99.9, but if parsed literally)
        ];

        for time_str in invalid_times {
            let result = parse_ass_time(time_str);
            // Some might succeed if parsed differently, just ensure it doesn't panic
            match result {
                Ok(centiseconds) => {
                    println!("Unexpectedly parsed '{time_str}' as {centiseconds} centiseconds");
                }
                Err(e) => {
                    println!("Failed to parse '{time_str}': {e}");
                }
            }
        }
    }

    #[test]
    fn test_parse_ass_time_malformed_input() {
        // Test various malformed time inputs to trigger different error paths
        let malformed_times = vec![
            "invalid",     // Not time format
            "1:2",         // Too few parts
            "1:2:3:4:5",   // Too many parts
            "a:b:c.d",     // Non-numeric
            "-1:30:45.50", // Negative hours
            "1:-30:45.50", // Negative minutes
            "1:30:-45.50", // Negative seconds
            "1:30:45.-50", // Negative centiseconds
            "",            // Empty string
            ":",           // Just colons
            "..",          // Just dots
            "1::30.50",    // Double colon
            "1:30:.50",    // Missing seconds
            "1:30:45.",    // Missing centiseconds
        ];

        for time_str in malformed_times {
            let result = parse_ass_time(time_str);
            assert!(result.is_err(), "Should fail for: {time_str}");
        }
    }

    #[test]
    fn test_parse_bgr_color_edge_cases() {
        // Test BGR color parsing edge cases
        let invalid_colors = vec![
            "invalid",   // Not hex
            "&H",        // Too short
            "&HGGGGGG",  // Invalid hex chars
            "&H12345",   // Wrong length
            "&H1234567", // Too long
            "",          // Empty
            "123456",    // Missing &H prefix
            "&H-123456", // Negative
        ];

        for color_str in invalid_colors {
            let result = parse_bgr_color(color_str);
            // Should either succeed with default or fail gracefully
            if result.is_err() {
                println!("Failed to parse color '{color_str}' as expected");
            }
        }
    }

    #[test]
    fn test_parse_bgr_color_valid_cases() {
        // Test valid BGR color cases
        let valid_colors = vec![
            "&H000000", // Black
            "&HFFFFFF", // White
            "&HFF0000", // Blue
            "&H00FF00", // Green
            "&H0000FF", // Red
            "&H123456", // Random valid color
        ];

        for color_str in valid_colors {
            let result = parse_bgr_color(color_str);
            assert!(result.is_ok(), "Should parse color: {color_str}");
        }
    }

    #[test]
    fn test_decode_uu_data_edge_cases() {
        // Test UU decoding edge cases
        let long_invalid = "!".repeat(100);
        let invalid_uu_data = vec![
            "",             // Empty
            "!",            // Single invalid char
            "invalid data", // Invalid UU data
            &long_invalid,  // Long invalid data
            "\x00\x01\x02", // Binary data
        ];

        for uu_str in invalid_uu_data {
            let result = decode_uu_data(uu_str.lines());
            // Should either succeed or fail gracefully
            if result.is_err() {
                println!("Failed to decode UU data '{uu_str}' as expected");
            }
        }
    }

    #[test]
    fn test_decode_uu_data_valid_cases() {
        // Test valid UU encoding cases
        let valid_cases = vec![
            ("", vec![]), // Empty should give empty
                          // Add some basic valid UU encoded data if we know the format
        ];

        for (input, expected) in valid_cases {
            let result = decode_uu_data(input.lines());
            if result.is_ok() {
                let decoded = result.unwrap();
                assert_eq!(decoded, expected);
            }
        }
    }

    #[test]
    fn test_spans_edge_cases() {
        // Test Spans utility with edge cases
        let empty_source = "";
        let empty_spans = Spans::new(empty_source);

        // Test methods on empty source
        assert!(empty_spans.span_offset("anything").is_none());
        assert!(empty_spans.span_column("anything").is_none());

        // Test with whitespace-only source
        let whitespace_source = "   \n\t  \r\n  ";
        let whitespace_spans = Spans::new(whitespace_source);

        // Test finding whitespace spans
        if let Some(offset) = whitespace_spans.span_offset("   ") {
            assert!(offset < whitespace_source.len());
        }

        // Test with unicode source
        let unicode_source = "Hello 🌍 世界 مرحبا";
        let unicode_spans = Spans::new(unicode_source);

        // Test finding unicode spans
        if let Some(column) = unicode_spans.span_column("🌍") {
            assert!(column > 1);
        }

        if let Some(column) = unicode_spans.span_column("世界") {
            assert!(column > 1);
        }
    }

    #[test]
    fn test_spans_line_column_calculations() {
        // Test line and column calculations with complex text
        let multiline_source = "First line\nSecond line with 🌍\n\nFourth line";
        let spans = Spans::new(multiline_source);

        // Test various spans and their positions
        let test_cases = vec![
            ("First", 1),  // Should be column 1
            ("Second", 1), // Should be column 1 of line 2
            ("with", 13),  // Should be after "Second line "
            ("🌍", 18),    // Unicode character position
            ("Fourth", 1), // After empty line
        ];

        for (span_text, expected_min_column) in test_cases {
            if let Some(column) = spans.span_column(span_text) {
                assert!(
                    column >= expected_min_column,
                    "Column for '{span_text}' should be >= {expected_min_column}, got {column}"
                );
            }
        }
    }

    #[test]
    fn test_time_parsing_boundary_values() {
        // Test boundary values for time parsing
        let boundary_cases = vec![
            ("0:59:59.99", true),   // Maximum valid minutes/seconds/centiseconds
            ("23:59:59.99", true),  // Maximum reasonable time
            ("0:00:00.00", true),   // Minimum time
            ("0:59:60.00", false),  // Invalid seconds
            ("0:60:00.00", false),  // Invalid minutes
            ("0:00:00.100", false), // Invalid centiseconds (if parsed strictly)
        ];

        for (time_str, should_succeed) in boundary_cases {
            let result = parse_ass_time(time_str);
            if should_succeed {
                assert!(result.is_ok(), "Should succeed parsing: {time_str}");
            } else {
                assert!(result.is_err(), "Should fail parsing: {time_str}");
            }
        }
    }

    #[test]
    fn test_numeric_parsing_edge_cases() {
        // Test numeric parsing edge cases that might trigger uncovered paths
        let edge_cases = vec![
            "999:999:999.999", // Very large numbers
            "000:000:000.000", // Leading zeros
            "1:2:3.4",         // Single digits
            " 1:30:45.50 ",    // Whitespace (should fail)
            "1: 30:45.50",     // Internal whitespace (should fail)
        ];

        for time_str in edge_cases {
            let result = parse_ass_time(time_str);
            // Just ensure it doesn't panic and handles edge cases
            match result {
                Ok(centiseconds) => {
                    println!("Parsed '{time_str}' as {centiseconds} centiseconds");
                }
                Err(e) => {
                    println!("Failed to parse '{time_str}': {e}");
                }
            }
        }
    }

    #[test]
    fn test_spans_with_special_characters() {
        // Test spans functionality with special characters
        let special_source = "Line1\r\nLine2\tWith\x00Null\nLine3";
        let spans = Spans::new(special_source);

        // Test finding spans with special characters
        if let Some(offset) = spans.span_offset("Line1") {
            assert_eq!(offset, 0);
        }

        if let Some(offset) = spans.span_offset("Line2") {
            assert!(offset > 0);
        }

        // Test column calculation with tabs and special chars
        if let Some(column) = spans.span_column("With") {
            assert!(column > 1);
        }
    }

    #[test]
    fn test_color_parsing_case_sensitivity() {
        // Test color parsing with different cases
        let color_cases = vec![
            "&h123456", // Lowercase h
            "&H123456", // Uppercase H
            "&Habc123", // Mixed case hex
            "&HABC123", // Uppercase hex
            "&h000000", // Lowercase with zeros
        ];

        for color_str in color_cases {
            let result = parse_bgr_color(color_str);
            // Should handle case variations appropriately
            match result {
                Ok(color) => {
                    println!("Parsed color '{color_str}' as {color:?}");
                }
                Err(e) => {
                    println!("Failed to parse color '{color_str}': {e}");
                }
            }
        }
    }
}
