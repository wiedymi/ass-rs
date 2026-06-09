//! Edge case tests for UU data decoding in the utils module.
//!
//! Verifies that `decode_uu_data` handles malformed and boundary inputs
//! gracefully without panicking.

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::format;

use ass_core::utils::decode_uu_data;

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
