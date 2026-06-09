//! Tests for `TokenScanner::is_hex_value` hex string detection.

use crate::tokenizer::scanner::TokenScanner;

#[test]
fn is_hex_value_plain_hex() {
    assert!(TokenScanner::is_hex_value("FF00FF"));
    assert!(TokenScanner::is_hex_value("0000FF"));
    assert!(TokenScanner::is_hex_value("AABBCC"));
    assert!(TokenScanner::is_hex_value("123456"));
}

#[test]
fn is_hex_value_with_prefix_suffix() {
    assert!(TokenScanner::is_hex_value("&HFF0000&"));
    assert!(TokenScanner::is_hex_value("&H00FF00&"));
    assert!(TokenScanner::is_hex_value("&H0000FF&"));
}

#[test]
fn is_hex_value_with_prefix_no_suffix() {
    assert!(TokenScanner::is_hex_value("&HFF0000"));
    assert!(TokenScanner::is_hex_value("&H00FF00"));
    assert!(TokenScanner::is_hex_value("&H0000FF"));
}

#[test]
fn is_hex_value_invalid_cases() {
    assert!(!TokenScanner::is_hex_value(""));
    assert!(!TokenScanner::is_hex_value("G00000"));
    assert!(!TokenScanner::is_hex_value("FF00F")); // Odd length
    assert!(!TokenScanner::is_hex_value("&H&"));
    assert!(!TokenScanner::is_hex_value("&HG0000&"));
    assert!(!TokenScanner::is_hex_value("&HFF00F&")); // Odd length in hex part
    assert!(!TokenScanner::is_hex_value("0x123456")); // Wrong prefix
}

#[test]
fn is_hex_value_edge_cases() {
    assert!(!TokenScanner::is_hex_value("&H&"));
    assert!(!TokenScanner::is_hex_value("&H"));
    assert!(!TokenScanner::is_hex_value("abc")); // Odd length
    assert!(TokenScanner::is_hex_value("ab")); // Even length, valid hex
}
