//! Text content validation and sanitization tests for the utils module.

use crate::utils::{sanitize_text_content, validate_text_content};

#[test]
fn text_validation_ascii() {
    assert!(validate_text_content("Hello World").is_ok());
    assert!(validate_text_content("1234567890").is_ok());
    assert!(validate_text_content("!@#$%^&*()").is_ok());
}

#[test]
fn text_validation_unicode() {
    assert!(validate_text_content("こんにちは").is_ok());
    assert!(validate_text_content("Здравствуй").is_ok());
    assert!(validate_text_content("مرحبا").is_ok());
    assert!(validate_text_content("🎵🎬🎭").is_ok());
}

#[test]
fn text_validation_control_chars() {
    // Control characters should be flagged
    let result = validate_text_content("Hello\x00World");
    assert!(result.is_err());

    let result = validate_text_content("Text\x1FTest");
    assert!(result.is_err());
}

#[test]
fn text_validation_line_endings() {
    // Line endings should be allowed
    assert!(validate_text_content("Line1\nLine2").is_ok());
    assert!(validate_text_content("Line1\r\nLine2").is_ok());
    assert!(validate_text_content("Line1\rLine2").is_ok());
}

#[test]
fn text_sanitization() {
    let input = "Hello\x00\x1F World";
    let result = sanitize_text_content(input);
    assert_eq!(result, "Hello World");

    let input = "Good\x0BText\x0CTest";
    let result = sanitize_text_content(input);
    assert_eq!(result, "GoodTextTest");
}
