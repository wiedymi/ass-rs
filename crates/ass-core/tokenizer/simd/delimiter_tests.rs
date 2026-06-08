//! Unit tests for SIMD-accelerated delimiter scanning

use super::*;
#[cfg(not(feature = "std"))]
use alloc::{format, vec};

#[test]
fn scan_delimiters_finds_colon() {
    let text = "key: value";
    assert_eq!(scan_delimiters(text), Some(3));
}

#[test]
fn scan_delimiters_finds_comma() {
    let text = "value1, value2";
    assert_eq!(scan_delimiters(text), Some(6));
}

#[test]
fn scan_delimiters_finds_brace() {
    let text = "text{override}";
    assert_eq!(scan_delimiters(text), Some(4));
}

#[test]
fn scan_delimiters_no_match() {
    let text = "plain text";
    assert_eq!(scan_delimiters(text), None);
}

#[test]
fn scan_delimiters_long_text() {
    let text = format!("{}:value", "a".repeat(50));
    assert_eq!(scan_delimiters(&text), Some(50));
}

#[test]
fn scan_delimiters_all_delimiter_types() {
    // Test each delimiter type individually
    let delimiters = vec![
        ("text:more", 4, ':'),
        ("text,more", 4, ','),
        ("text{more", 4, '{'),
        ("text}more", 4, '}'),
        ("text[more", 4, '['),
        ("text]more", 4, ']'),
        ("text\nmore", 4, '\n'),
        ("text\rmore", 4, '\r'),
    ];

    for (text, expected_pos, _delimiter) in delimiters {
        assert_eq!(scan_delimiters(text), Some(expected_pos));
    }
}

#[test]
fn scan_delimiters_empty_input() {
    assert_eq!(scan_delimiters(""), None);
}

#[test]
fn scan_delimiters_single_char() {
    assert_eq!(scan_delimiters(":"), Some(0));
    assert_eq!(scan_delimiters("a"), None);
}

#[test]
fn scan_delimiters_at_beginning() {
    assert_eq!(scan_delimiters(":text"), Some(0));
    assert_eq!(scan_delimiters(",text"), Some(0));
    assert_eq!(scan_delimiters("{text"), Some(0));
}

#[test]
fn scan_delimiters_at_end() {
    assert_eq!(scan_delimiters("text:"), Some(4));
    assert_eq!(scan_delimiters("text,"), Some(4));
    assert_eq!(scan_delimiters("text}"), Some(4));
}

#[test]
fn scan_delimiters_multiple_delimiters() {
    // Should find the first one
    assert_eq!(scan_delimiters("a:b,c{d"), Some(1));
    assert_eq!(scan_delimiters("text,more:values"), Some(4));
}

#[test]
fn scan_delimiters_exactly_16_bytes() {
    // Test boundary condition for SIMD
    let text = "abcdefghijklmno:"; // 16 chars
    assert_eq!(scan_delimiters(text), Some(15));
}

#[test]
fn scan_delimiters_less_than_16_bytes() {
    // Should use scalar implementation
    let text = "short:text"; // 10 chars
    assert_eq!(scan_delimiters(text), Some(5));
}

#[test]
fn scan_delimiters_much_longer_than_16_bytes() {
    // Test multiple SIMD chunks
    let prefix = "a".repeat(32);
    let text = format!("{prefix}:value");
    assert_eq!(scan_delimiters(&text), Some(32));
}

#[test]
fn scan_delimiters_unicode_text() {
    let text = "café🎭:value";
    let colon_pos = text.find(':').unwrap();
    assert_eq!(scan_delimiters(text), Some(colon_pos));
}

#[test]
fn scan_delimiters_scalar_fallback() {
    // Test scalar implementation directly by using short strings
    let short_texts = vec![
        "a:b",      // 3 chars
        "test,val", // 8 chars
        "x{y}z",    // 5 chars
    ];

    for text in short_texts {
        let result = scan_delimiters(text);
        assert!(result.is_some());
    }
}

#[test]
fn scan_delimiters_boundary_at_chunk_edge() {
    // Test delimiter exactly at 16-byte boundary
    let text = format!("{}:", "a".repeat(15)); // 15 'a's + ':'
    assert_eq!(scan_delimiters(&text), Some(15));

    // Test delimiter just after 16-byte boundary
    let text2 = format!("{}:", "a".repeat(16)); // 16 'a's + ':'
    assert_eq!(scan_delimiters(&text2), Some(16));
}

#[test]
fn scan_delimiters_no_false_positives() {
    // Ensure similar characters don't trigger false positives
    let text = "abcdefghijklmnopqrstuvwxyz"; // No delimiters
    assert_eq!(scan_delimiters(text), None);

    let text2 = "0123456789"; // No delimiters
    assert_eq!(scan_delimiters(text2), None);
}

#[test]
fn scan_delimiters_all_positions() {
    // Test delimiter at every position in a string
    for i in 0..10 {
        let mut chars: Vec<char> = "abcdefghij".chars().collect();
        chars[i] = ':';
        let text: String = chars.iter().collect();
        assert_eq!(scan_delimiters(&text), Some(i));
    }
}
