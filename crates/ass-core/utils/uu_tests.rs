//! Tests for UU-encoded data decoding.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::{format, vec};

#[test]
fn decode_uu_data_empty_input() {
    let lines: Vec<&str> = vec![];
    let decoded = decode_uu_data(lines.iter().copied()).unwrap();
    assert_eq!(decoded, Vec::<u8>::new());
}

#[test]
fn decode_uu_data_known_encoding() {
    // Test known UU-encoded data: "Cat" -> "#0V%T"
    let lines = ["#0V%T"];
    let decoded = decode_uu_data(lines.iter().copied()).unwrap();
    assert_eq!(decoded, b"Cat");
}

#[test]
fn decode_uu_data_known_encoding_png() {
    // Test known UU-encoded data: "PNG" -> "#4$Y'"
    let lines = ["#4$Y'"];
    let decoded = decode_uu_data(lines.iter().copied()).unwrap();
    assert_eq!(decoded, b"PNG");
}

#[test]
fn decode_uu_data_multiline() {
    // Test multi-line UU-encoded data
    let lines = ["#0V%T", "#0V%T"];
    let decoded = decode_uu_data(lines.iter().copied()).unwrap();
    assert_eq!(decoded, b"CatCat");
}

#[test]
fn decode_uu_data_with_end_marker() {
    let lines = ["#0V%T", "end"];
    let decoded = decode_uu_data(lines.iter().copied()).unwrap();
    assert_eq!(decoded, b"Cat");
}

#[test]
fn decode_uu_data_with_end_marker_spaced() {
    let lines = ["#0V%T", "end 644"];
    let decoded = decode_uu_data(lines.iter().copied()).unwrap();
    assert_eq!(decoded, b"Cat");
}

#[test]
fn decode_uu_data_zero_length_line() {
    // Zero-length line should terminate decoding
    let lines = ["#0V%T", " "];
    let decoded = decode_uu_data(lines.iter().copied()).unwrap();
    assert_eq!(decoded, b"Cat");
}

#[test]
fn decode_uu_data_whitespace_lines() {
    let lines = ["  #0V%T  ", "\t", ""];
    let decoded = decode_uu_data(lines.iter().copied()).unwrap();
    assert_eq!(decoded, b"Cat");
}

#[test]
fn decode_uu_data_length_validation() {
    // Test that length encoding is respected
    let lines = ["!    "]; // '!' encodes length 1, but provides 4 characters of data
    let decoded = decode_uu_data(lines.iter().copied()).unwrap();
    assert_eq!(decoded.len(), 1); // Should be truncated to declared length
}

#[test]
fn decode_uu_data_partial_chunks() {
    // Test handling of incomplete 4-character groups
    let lines = ["\"``"]; // Only 3 characters after length byte
    let decoded = decode_uu_data(lines.iter().copied()).unwrap();
    assert_eq!(decoded.len(), 2); // Should decode what's available
}

#[test]
fn decode_uu_data_large_line() {
    // Test handling of max-length UU line (45 bytes -> 60 characters + length)
    let line = format!("M{}", "!!!!".repeat(15)); // 45 bytes of data
    let lines = [line.as_str()];
    let decoded = decode_uu_data(lines.iter().copied()).unwrap();
    assert_eq!(decoded.len(), 45);
}

#[test]
fn decode_uu_data_mixed_content() {
    let lines = [
        "begin 644 test.txt", // Should be ignored
        "#0V%T",              // Should be decoded
        "| comment",          // Should be ignored as it doesn't start with valid length
        "#4$Y'",              // Should be decoded
        "end",                // Should terminate
    ];
    let decoded = decode_uu_data(lines.iter().copied()).unwrap();
    assert_eq!(decoded, b"CatPNG");
}

#[test]
fn decode_uu_data_all_printable_chars() {
    // Test that decoder handles all valid UU characters (space to underscore)
    let lines = ["@ !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_"];
    let _decoded = decode_uu_data(lines.iter().copied()).unwrap();
    // Should not panic, exact output depends on UU encoding rules
}

#[test]
fn decode_uu_data_boundary_lengths() {
    // Test boundary cases for line lengths
    let single_byte = ["!   "]; // Length 1
    let two_bytes = ["\"`` "]; // Length 2
    let three_bytes = ["#```"]; // Length 3

    let decoded1 = decode_uu_data(single_byte.iter().copied()).unwrap();
    assert_eq!(decoded1.len(), 1);

    let decoded2 = decode_uu_data(two_bytes.iter().copied()).unwrap();
    assert_eq!(decoded2.len(), 2);

    let decoded3 = decode_uu_data(three_bytes.iter().copied()).unwrap();
    assert_eq!(decoded3.len(), 3);
}

#[test]
fn decode_uu_data_handles_invalid_gracefully() {
    // Decoder should not panic on invalid characters
    let lines = ["#\x01\x02\x03"]; // Non-printable characters
    let _result = decode_uu_data(lines.iter().copied());
    // Should not panic, may return unexpected data or error
}

#[test]
fn decode_uu_data_error_conditions() {
    // Test with only invalid lines
    let invalid_lines = ["invalid", "also invalid", "still invalid"];
    let result = decode_uu_data(invalid_lines.iter().copied()).unwrap();
    assert!(result.is_empty());

    // Test with malformed length indicators
    let malformed_length = ["\x7F!!!!"]; // Length > 45
    let _result = decode_uu_data(malformed_length.iter().copied());
    // Should handle gracefully

    // Test with very short lines after valid length
    let short_lines = ["!"]; // Length 1 but no data
    let result = decode_uu_data(short_lines.iter().copied()).unwrap();
    assert!(result.is_empty() || result.len() <= 1);

    // Test with unicode in data
    let unicode_lines = ["#🎭🎭🎭"];
    let _result = decode_uu_data(unicode_lines.iter().copied());
    // Should handle gracefully without panicking
}
