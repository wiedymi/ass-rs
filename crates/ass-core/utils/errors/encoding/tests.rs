//! Unit tests for the text encoding error and validation utilities.

use super::super::CoreError;
use super::validation::is_valid_ass_char;
use super::*;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn utf8_error_creation() {
    let error = utf8_error(42, "test message".to_string());
    assert!(matches!(error, CoreError::Utf8Error { position: 42, .. }));
}

#[test]
fn validation_error_creation() {
    let error = validation_error("invalid content");
    assert!(matches!(error, CoreError::Validation(_)));
}

#[test]
fn validate_valid_utf8() {
    let text = "Hello, 世界! 🎵";
    assert!(validate_utf8_detailed(text.as_bytes()).is_ok());
}

#[test]
fn validate_invalid_utf8() {
    let invalid_bytes = &[0xFF, 0xFE, 0x80];
    assert!(validate_utf8_detailed(invalid_bytes).is_err());
}

#[test]
fn validate_ass_text_valid() {
    assert!(validate_ass_text_content("Hello World").is_ok());
    assert!(validate_ass_text_content("Hello\tWorld\n").is_ok());
    assert!(validate_ass_text_content("Hello 世界").is_ok());
}

#[test]
fn validate_ass_text_invalid() {
    assert!(validate_ass_text_content("Hello\x00World").is_err()); // Null character
    assert!(validate_ass_text_content("Hello\x1FWorld").is_err()); // Control character
}

#[test]
fn valid_ass_char_check() {
    assert!(is_valid_ass_char('A'));
    assert!(is_valid_ass_char(' '));
    assert!(is_valid_ass_char('\n'));
    assert!(is_valid_ass_char('世'));
    assert!(!is_valid_ass_char('\x00'));
    assert!(!is_valid_ass_char('\x1F'));
}

#[test]
fn bom_validation_utf8() {
    let utf8_bom = &[0xEF, 0xBB, 0xBF, b'H', b'i'];
    assert!(validate_bom_handling(utf8_bom).is_ok());
}

#[test]
fn bom_validation_utf16() {
    let utf16_bom = &[0xFF, 0xFE, b'H', 0x00, b'i', 0x00];
    assert!(validate_bom_handling(utf16_bom).is_err());
}

#[test]
fn bom_validation_no_bom() {
    let no_bom = b"Hello World";
    assert!(validate_bom_handling(no_bom).is_ok());
}

#[test]
fn bom_validation_partial_utf8() {
    let partial_bom = &[0xEF, 0xBB, b'H', b'i'];
    assert!(validate_bom_handling(partial_bom).is_err());
}

#[test]
fn bom_validation_single_ef_byte() {
    let single_ef = &[0xEF, b'H', b'i'];
    assert!(validate_bom_handling(single_ef).is_err());
}

#[test]
fn bom_validation_ef_only() {
    let ef_only = &[0xEF];
    assert!(validate_bom_handling(ef_only).is_err());
}
