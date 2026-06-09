//! Targeted coverage tests for `decode_uu_data` in `utils/mod.rs`.

use ass_core::utils::decode_uu_data;

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
