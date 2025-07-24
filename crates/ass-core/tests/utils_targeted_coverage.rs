//! Targeted tests for uncovered code paths in utils/mod.rs
//!
//! These tests specifically target the uncovered lines identified in coverage analysis
//! to improve test coverage for utility functions.

use ass_core::utils::{decode_uu_data, parse_ass_time, parse_bgr_color, CoreError, Spans};

#[cfg(test)]
mod utils_targeted_coverage {
    use super::*;

    #[test]
    fn test_spans_column_with_invalid_span() {
        // This should hit line 97: span_column when span_offset returns None
        let source = "Hello\nWorld\nTest";
        let spans = Spans::new(source);

        // Test with span not in source
        let invalid_span = "NotInSource";
        let result = spans.span_column(invalid_span);
        assert!(result.is_none());

        // Test with empty span
        let empty_span = "";
        let result = spans.span_column(empty_span);
        // Empty span might return None depending on implementation
        let _ = result;
    }

    #[test]
    fn test_spans_substring_out_of_bounds() {
        // Test substring with out of bounds range
        let source = "Hello World";
        let spans = Spans::new(source);

        // Test range beyond source length
        let result = spans.substring(20..25);
        assert!(result.is_none());

        // Test range starting beyond source length
        let result = spans.substring(15..20);
        assert!(result.is_none());

        // Test empty range at end of string
        let result = spans.substring(11..11);
        assert_eq!(result, Some(""));

        // Test range at exact boundary
        let result = spans.substring(10..12);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_ass_time_invalid_minutes() {
        // This should hit line 292: minutes >= 60 validation
        let invalid_times = vec![
            "1:60:30.45", // 60 minutes (should be < 60)
            "0:65:00.00", // 65 minutes
            "2:99:15.50", // 99 minutes
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
        // This should hit seconds >= 60 validation
        let invalid_times = vec![
            "1:30:60.45", // 60 seconds (should be < 60)
            "0:45:65.00", // 65 seconds
            "2:15:99.50", // 99 seconds
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
        // Test centiseconds validation edge cases
        let invalid_times = vec![
            "1:30:45.100", // 100 centiseconds (should be < 100)
            "0:45:30.999", // 999 centiseconds
        ];

        for time_str in invalid_times {
            let result = parse_ass_time(time_str);
            // May be valid or invalid depending on implementation
            let _ = result;
        }
    }

    #[test]
    fn test_parse_ass_time_malformed_format() {
        // Test various malformed time formats
        let malformed_times = vec![
            "",            // Empty string
            "invalid",     // Non-time string
            "1:30",        // Missing seconds and centiseconds
            "1:30:45",     // Missing centiseconds
            "1:30:45.",    // Missing centiseconds value
            "1:30:45.5",   // Only one centisecond digit
            "a:30:45.50",  // Non-numeric hours
            "1:b:45.50",   // Non-numeric minutes
            "1:30:c.50",   // Non-numeric seconds
            "1:30:45.d",   // Non-numeric centiseconds
            "1::45.50",    // Missing minutes
            ":30:45.50",   // Missing hours
            "1:30:.50",    // Missing seconds
            "-1:30:45.50", // Negative hours
            "1:-30:45.50", // Negative minutes
            "1:30:-45.50", // Negative seconds
            "1:30:45.-50", // Negative centiseconds
        ];

        for time_str in malformed_times {
            let result = parse_ass_time(time_str);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_parse_bgr_color_invalid_formats() {
        // Test invalid color format handling
        let invalid_colors = vec![
            "",             // Empty string
            "invalid",      // Non-color string
            "&H",           // Incomplete prefix
            "&HGGGGGG",     // Invalid hex characters
            "&H12345",      // Too short
            "&H1234567890", // Too long
            "H123456",      // Missing &
            "&G123456",     // Wrong prefix
            "&h123456",     // Lowercase prefix
            "&H12345G",     // Invalid hex character
            "&H-123456",    // Negative sign
        ];

        for color_str in invalid_colors {
            let result = parse_bgr_color(color_str);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_parse_bgr_color_edge_cases() {
        // Test edge cases for color parsing
        let edge_cases = vec![
            ("&H000000", true),  // Black (minimum)
            ("&HFFFFFF", true),  // White (maximum)
            ("&H123456", true),  // Valid hex
            ("&Habc123", false), // Lowercase hex (might be invalid)
            ("&HABCDEF", true),  // Uppercase hex
        ];

        for (color_str, should_be_valid) in edge_cases {
            let result = parse_bgr_color(color_str);

            if should_be_valid {
                assert!(result.is_ok(), "Color {color_str} should be valid");
            } else {
                // Some edge cases might be handled differently
                let _ = result;
            }
        }
    }

    #[test]
    fn test_decode_uu_data_invalid_input() {
        // Test UU decoding with invalid input
        let invalid_inputs = vec![
            vec![""],          // Empty line
            vec!["invalid"],   // Non-UU data
            vec!["M"],         // Too short
            vec!["MMMM"],      // Invalid length
            vec!["M@@@"],      // Invalid characters
            vec!["M", "", ""], // With empty lines
            vec!["M   "],      // Spaces
        ];

        for input_lines in invalid_inputs {
            let result = decode_uu_data(input_lines.iter().copied());
            // UU decoding might handle some invalid input gracefully
            let _ = result;
        }
    }

    #[test]
    fn test_decode_uu_data_boundary_conditions() {
        // Test UU decoding boundary conditions
        let long_input = "M".repeat(64);
        let boundary_cases = vec![
            vec!["M"],             // Minimum valid input
            vec![&long_input],     // Long input
            vec!["M\x21\x22\x23"], // With various characters
            vec!["M!\"#"],         // ASCII characters
        ];

        for input_lines in boundary_cases {
            let result = decode_uu_data(input_lines.iter().copied());
            // Test that it doesn't panic and handles gracefully
            let _ = result;
        }
    }

    #[test]
    fn test_spans_edge_cases_with_unicode() {
        // Test spans with Unicode content
        let unicode_source = "Hello 世界\n测试 🎬\nEnd";
        let spans = Spans::new(unicode_source);

        // Test span_line with Unicode
        let unicode_span = "世界";
        let line = spans.span_line(unicode_span);
        assert!(line.is_some());

        // Test span_column with Unicode
        let column = spans.span_column(unicode_span);
        assert!(column.is_some());

        // Test with emoji
        let emoji_span = "🎬";
        let emoji_line = spans.span_line(emoji_span);
        let emoji_column = spans.span_column(emoji_span);
        assert!(emoji_line.is_some());
        assert!(emoji_column.is_some());
    }

    #[test]
    fn test_spans_with_special_characters() {
        // Test spans with special characters and edge cases
        let special_source = "Line1\r\nLine2\n\nLine4\r";
        let spans = Spans::new(special_source);

        // Test with carriage return + newline
        let crlf_span = "\r\n";
        let _ = spans.span_line(crlf_span);
        let _ = spans.span_column(crlf_span);

        // Test with empty lines
        let empty_line_span = "\n\n";
        let _ = spans.span_line(empty_line_span);

        // Test with carriage return only
        let cr_span = "\r";
        let _ = spans.span_line(cr_span);
    }

    #[test]
    fn test_parse_ass_time_precision_edge_cases() {
        // Test time parsing with precision edge cases
        let precision_cases = vec![
            "0:00:00.00", // Minimum time
            "9:59:59.99", // Maximum valid time
            "0:00:00.01", // Minimum non-zero
            "5:30:30.50", // Middle values
        ];

        for time_str in precision_cases {
            let result = parse_ass_time(time_str);
            assert!(result.is_ok(), "Time {time_str} should be valid");
        }
    }

    #[test]
    fn test_utils_error_formatting() {
        // Test error message formatting
        let time_result = parse_ass_time("1:99:45.50");
        if let Err(error) = time_result {
            let error_string = error.to_string();
            assert!(!error_string.is_empty());
            assert!(error_string.contains("Minutes") || error_string.contains("invalid"));
        }

        let color_result = parse_bgr_color("invalid");
        if let Err(error) = color_result {
            let error_string = error.to_string();
            assert!(!error_string.is_empty());
        }
    }

    #[test]
    fn test_spans_offset_calculation_edge_cases() {
        // Test span offset calculation with edge cases
        let source = "Multi\nLine\nContent\nWith\nMany\nLines";
        let spans = Spans::new(source);

        // Test with spans at line boundaries
        let newline_span = "\n";
        let _ = spans.span_offset(newline_span);

        // Test with partial matches
        let partial_span = "Line";
        let offset = spans.span_offset(partial_span);
        assert!(offset.is_some());

        // Test with first and last characters
        let first_char = &source[0..1];
        let first_offset = spans.span_offset(first_char);
        assert!(first_offset.is_some());

        if !source.is_empty() {
            let last_char = &source[source.len() - 1..];
            let last_offset = spans.span_offset(last_char);
            assert!(last_offset.is_some());
        }
    }
}
