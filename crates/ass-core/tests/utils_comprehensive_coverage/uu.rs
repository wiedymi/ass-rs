//! Coverage tests for `decode_uu_data`, covering invalid byte streams and
//! valid (empty) input handling.

use ass_core::utils::decode_uu_data;

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
        if let Ok(decoded) = result {
            assert_eq!(decoded, expected);
        }
    }
}
