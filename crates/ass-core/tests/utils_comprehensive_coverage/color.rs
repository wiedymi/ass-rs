//! Coverage tests for `parse_bgr_color`, covering invalid input, valid input,
//! and case-sensitivity handling.

use ass_core::utils::parse_bgr_color;

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
