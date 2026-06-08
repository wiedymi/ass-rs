//! Unit tests for SIMD-accelerated UTF-8 validation

use super::*;
#[cfg(not(feature = "std"))]
use alloc::format;

#[test]
fn validate_utf8_valid() {
    assert!(validate_utf8_batch(b"Hello, World!").is_ok());
    assert!(validate_utf8_batch("Hello, 世界! 🎵".as_bytes()).is_ok());
    assert!(validate_utf8_batch("a".repeat(50).as_bytes()).is_ok());
}

#[test]
fn validate_utf8_invalid() {
    assert!(validate_utf8_batch(&[0xFF, 0xFE]).is_err());
}

#[test]
fn validate_utf8_empty_input() {
    assert!(validate_utf8_batch(&[]).is_ok());
}

#[test]
fn validate_utf8_ascii_only() {
    let ascii_text = "Hello, World! 123 @#$%";
    assert!(validate_utf8_batch(ascii_text.as_bytes()).is_ok());
}

#[test]
fn validate_utf8_exactly_16_bytes() {
    let text = "1234567890123456"; // Exactly 16 ASCII chars
    assert!(validate_utf8_batch(text.as_bytes()).is_ok());
}

#[test]
fn validate_utf8_less_than_16_bytes() {
    let text = "short"; // 5 ASCII chars
    assert!(validate_utf8_batch(text.as_bytes()).is_ok());
}

#[test]
fn validate_utf8_much_longer() {
    let text = "a".repeat(100);
    assert!(validate_utf8_batch(text.as_bytes()).is_ok());
}

#[test]
fn validate_utf8_mixed_unicode() {
    let text = "ASCII中文🎵عربي";
    assert!(validate_utf8_batch(text.as_bytes()).is_ok());
}

#[test]
fn validate_utf8_invalid_sequences() {
    // Invalid UTF-8 sequences
    assert!(validate_utf8_batch(&[0xC0, 0x80]).is_err()); // Overlong encoding
    assert!(validate_utf8_batch(&[0xED, 0xA0, 0x80]).is_err()); // Surrogate
    assert!(validate_utf8_batch(&[0xF4, 0x90, 0x80, 0x80]).is_err()); // Too large
}

#[test]
fn validate_utf8_incomplete_sequences() {
    // Incomplete UTF-8 sequences
    assert!(validate_utf8_batch(&[0xC2]).is_err()); // Missing continuation
    assert!(validate_utf8_batch(&[0xE0, 0x80]).is_err()); // Missing second continuation
    assert!(validate_utf8_batch(&[0xF0, 0x90, 0x80]).is_err()); // Missing third continuation
}

#[test]
fn validate_utf8_non_ascii_in_chunks() {
    // Test UTF-8 validation when non-ASCII appears in SIMD chunks
    let text = format!("{}café", "a".repeat(12)); // Should trigger SIMD with non-ASCII
    assert!(validate_utf8_batch(text.as_bytes()).is_ok());
}

#[test]
fn validate_utf8_chunk_remainder_handling() {
    // Test that remainder after 16-byte chunks is handled correctly
    let text = format!("{}café", "a".repeat(17)); // 17 ASCII + UTF-8
    assert!(validate_utf8_batch(text.as_bytes()).is_ok());

    let text2 = format!("{}🎵", "a".repeat(18)); // 18 ASCII + emoji
    assert!(validate_utf8_batch(text2.as_bytes()).is_ok());
}
