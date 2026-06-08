//! Unit tests for SIMD-accelerated hexadecimal parsing

use super::*;

#[test]
fn parse_hex_valid() {
    assert_eq!(parse_hex_u32("FF"), Some(0xFF));
    assert_eq!(parse_hex_u32("00FF00FF"), Some(0x00FF_00FF));
    assert_eq!(parse_hex_u32("12345678"), Some(0x1234_5678));
    assert_eq!(parse_hex_u32("abcdef"), Some(0x00ab_cdef));
    assert_eq!(parse_hex_u32("ABCDEF"), Some(0x00AB_CDEF));
}

#[test]
fn parse_hex_invalid() {
    assert_eq!(parse_hex_u32("GG"), None);
    assert_eq!(parse_hex_u32("123456789"), None); // Too long
    assert_eq!(parse_hex_u32(""), None);
    assert_eq!(parse_hex_u32("XYZ"), None);
}

#[test]
fn parse_hex_edge_cases() {
    // Test minimum and maximum lengths
    assert_eq!(parse_hex_u32("0"), Some(0));
    assert_eq!(parse_hex_u32("F"), Some(15));
    assert_eq!(parse_hex_u32("FFFFFFFF"), Some(0xFFFF_FFFF));

    // Test mixed case
    assert_eq!(parse_hex_u32("aBcDeF"), Some(0x00ab_cdef));
    assert_eq!(parse_hex_u32("AbCdEf"), Some(0x00ab_cdef));

    // Test leading zeros
    assert_eq!(parse_hex_u32("00000001"), Some(1));
    assert_eq!(parse_hex_u32("0000FF00"), Some(0xFF00));
}

#[test]
fn parse_hex_invalid_length() {
    // Too long
    assert_eq!(parse_hex_u32("123456789"), None);
    assert_eq!(parse_hex_u32("FFFFFFFFF"), None);
}

#[test]
fn parse_hex_invalid_characters() {
    assert_eq!(parse_hex_u32("GHIJ"), None);
    assert_eq!(parse_hex_u32("123G"), None);
    assert_eq!(parse_hex_u32("12 34"), None); // Space
    assert_eq!(parse_hex_u32("12-34"), None); // Hyphen
    assert_eq!(parse_hex_u32("FF\n"), None); // Newline
}

#[test]
fn parse_hex_overflow_handling() {
    // Test values that would overflow if not handled properly
    assert_eq!(parse_hex_u32("FFFFFFFF"), Some(u32::MAX));
}

#[test]
fn parse_hex_scalar_fallback() {
    // Test scalar implementation with short hex strings
    assert_eq!(parse_hex_u32("A"), Some(10));
    assert_eq!(parse_hex_u32("FF"), Some(255));
    assert_eq!(parse_hex_u32("123"), Some(0x123));
}

#[test]
fn parse_hex_case_sensitivity() {
    // Ensure both cases produce same result
    assert_eq!(parse_hex_u32("abcdef"), parse_hex_u32("ABCDEF"));
    assert_eq!(parse_hex_u32("deadbeef"), parse_hex_u32("DEADBEEF"));
}

#[test]
fn parse_hex_maximum_value() {
    // Test parsing maximum u32 value
    assert_eq!(parse_hex_u32("FFFFFFFF"), Some(u32::MAX));
    assert_eq!(parse_hex_u32("ffffffff"), Some(u32::MAX));
}
