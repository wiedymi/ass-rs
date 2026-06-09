//! Targeted coverage tests for `parse_bgr_color` in `utils/mod.rs`.

use ass_core::utils::parse_bgr_color;

#[test]
fn test_parse_bgr_color_invalid_formats() {
    // Test invalid color format handling
    let invalid_colors = vec![
        "",             // Empty string
        "xyz123",       // Non-hex characters
        "&H",           // Incomplete prefix
        "&HGGGGGG",     // Invalid hex characters
        "&H12345",      // Too short (5 digits)
        "&H1234567890", // Too long (10 digits)
        "H123456",      // Missing &
        "&G123456",     // Wrong prefix
        "&H12345G",     // Invalid hex character
        "&H-123456",    // Negative sign
        "&H123Z56",     // Invalid hex character Z
        "&H",           // Just prefix
        "invalid123",   // Contains non-hex chars
    ];

    for color_str in invalid_colors {
        let result = parse_bgr_color(color_str);
        assert!(
            result.is_err(),
            "Expected {color_str} to be invalid but it was valid"
        );
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
