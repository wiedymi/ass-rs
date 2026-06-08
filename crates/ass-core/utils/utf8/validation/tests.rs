//! Unit tests for UTF-8 validation, recovery, and truncation utilities.

use super::*;

#[test]
fn validate_valid_utf8() {
    let text = "Hello, 世界! 🎵";
    assert!(validate_utf8(text.as_bytes()).is_ok());
}

#[test]
fn validate_invalid_utf8() {
    let invalid_bytes = &[0xFF, 0xFE, 0x80]; // Invalid UTF-8
    assert!(validate_utf8(invalid_bytes).is_err());
}

#[test]
fn validate_incomplete_utf8() {
    let incomplete_bytes = &[0xC2]; // Incomplete UTF-8 sequence
    let result = validate_utf8(incomplete_bytes);
    assert!(result.is_err());
}

#[test]
fn recover_valid_utf8() {
    let text = "Hello, World!";
    let (recovered, replacements) = recover_utf8(text.as_bytes());
    assert_eq!(recovered, "Hello, World!");
    assert_eq!(replacements, 0);
}

#[test]
fn recover_invalid_utf8() {
    let invalid_bytes = &[b'H', b'i', 0xFF, b'!'];
    let (recovered, replacements) = recover_utf8(invalid_bytes);
    assert_eq!(recovered, "Hi�!");
    assert_eq!(replacements, 1);
}

#[test]
fn recover_multiple_invalid_sequences() {
    let invalid_bytes = &[b'A', 0xFF, b'B', 0xFE, b'C'];
    let (recovered, replacements) = recover_utf8(invalid_bytes);
    assert_eq!(recovered, "A�B�C");
    assert_eq!(replacements, 2);
}

#[test]
fn valid_ass_text() {
    assert!(is_valid_ass_text("Hello World"));
    assert!(is_valid_ass_text("Hello\tWorld\n"));
    assert!(is_valid_ass_text("Hello 世界"));
    assert!(!is_valid_ass_text("Hello\x00World")); // Null character
    assert!(!is_valid_ass_text("Hello\x1FWorld")); // Control character
}

#[test]
fn truncate_ascii() {
    let text = "Hello World";
    let (truncated, was_truncated) = truncate_at_char_boundary(text, 5);
    assert_eq!(truncated, "Hello");
    assert!(was_truncated);
}

#[test]
fn truncate_unicode() {
    let text = "Hello 世界";
    let (truncated, was_truncated) = truncate_at_char_boundary(text, 8);
    assert_eq!(truncated, "Hello "); // Stops before the Unicode character
    assert!(was_truncated);
}

#[test]
fn truncate_no_change() {
    let text = "Hello";
    let (truncated, was_truncated) = truncate_at_char_boundary(text, 10);
    assert_eq!(truncated, "Hello");
    assert!(!was_truncated);
}

#[test]
fn truncate_at_unicode_boundary() {
    let text = "世界";
    let (truncated, was_truncated) = truncate_at_char_boundary(text, 3);
    assert_eq!(truncated, "世");
    assert!(was_truncated);
}

#[test]
fn count_replacement_characters() {
    assert_eq!(count_replacement_chars("Hello World"), 0);
    assert_eq!(count_replacement_chars("Hello � World"), 1);
    assert_eq!(count_replacement_chars("� Test � Again �"), 3);
}
