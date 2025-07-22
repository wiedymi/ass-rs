//! Text encoding error utilities for ASS-RS
//!
//! Provides specialized error creation and validation functions for text
//! encoding issues including UTF-8 validation, encoding detection, and
//! character conversion errors. Focuses on providing detailed context.

use super::CoreError;
use alloc::{format, string::String};
use core::fmt;

/// Create UTF-8 encoding error with position information
///
/// Generates a CoreError::Utf8Error with detailed position and context
/// information about the encoding failure.
///
/// # Arguments
///
/// * `position` - Byte position where the error occurred
/// * `message` - Descriptive error message
///
/// # Examples
///
/// ```rust
/// use ass_core::utils::errors::{utf8_error, CoreError};
///
/// let error = utf8_error(42, "Invalid UTF-8 sequence".to_string());
/// assert!(matches!(error, CoreError::Utf8Error { .. }));
/// ```
pub fn utf8_error(position: usize, message: String) -> CoreError {
    CoreError::Utf8Error { position, message }
}

/// Create validation error for text content
///
/// Generates a CoreError::Validation for content that fails ASS-specific
/// text validation rules (e.g., contains invalid control characters).
///
/// # Arguments
///
/// * `message` - Description of the validation failure
pub fn validation_error<T: fmt::Display>(message: T) -> CoreError {
    CoreError::Validation(format!("{}", message))
}

/// Validate UTF-8 with detailed error reporting
///
/// Provides more detailed error information than standard UTF-8 validation,
/// including the exact position and nature of encoding errors.
///
/// # Arguments
///
/// * `bytes` - Byte sequence to validate
///
/// # Returns
///
/// `Ok(())` if valid UTF-8, detailed error if invalid
pub fn validate_utf8_detailed(bytes: &[u8]) -> Result<(), CoreError> {
    match core::str::from_utf8(bytes) {
        Ok(_) => Ok(()),
        Err(err) => {
            let position = err.valid_up_to();
            let message = if let Some(len) = err.error_len() {
                format!(
                    "Invalid UTF-8 sequence of {} bytes at position {}",
                    len, position
                )
            } else {
                format!("Incomplete UTF-8 sequence at position {}", position)
            };

            Err(utf8_error(position, message))
        }
    }
}

/// Validate text contains only valid ASS characters
///
/// Checks that text contains only characters appropriate for ASS subtitle
/// content, rejecting problematic control characters and sequences.
///
/// # Arguments
///
/// * `text` - Text content to validate
///
/// # Returns
///
/// `Ok(())` if valid, validation error if invalid characters found
pub fn validate_ass_text_content(text: &str) -> Result<(), CoreError> {
    for (pos, ch) in text.char_indices() {
        if !is_valid_ass_char(ch) {
            return Err(validation_error(format!(
                "Invalid character '{}' (U+{:04X}) at position {}",
                ch.escape_default().collect::<String>(),
                ch as u32,
                pos
            )));
        }
    }
    Ok(())
}

/// Check if character is valid in ASS content
///
/// Determines whether a character is acceptable in ASS subtitle content
/// based on ASS specification guidelines.
fn is_valid_ass_char(ch: char) -> bool {
    match ch {
        // Allow printable ASCII
        c if c.is_ascii_graphic() => true,
        // Allow whitespace
        ' ' | '\t' | '\n' | '\r' => true,
        // Allow non-ASCII printable characters (Unicode)
        c if !c.is_ascii() && !c.is_control() => true,
        // Reject control characters and other problematic chars
        _ => false,
    }
}

/// Validate BOM (Byte Order Mark) handling
///
/// Ensures that BOM is properly handled or warns if unexpected BOM found.
/// ASS files should typically use UTF-8 without BOM for compatibility.
///
/// # Arguments
///
/// * `bytes` - Input bytes that may contain BOM
///
/// # Returns
///
/// `Ok(())` if BOM handling is appropriate, warning if issues found
#[allow(dead_code)]
pub fn validate_bom_handling(bytes: &[u8]) -> Result<(), CoreError> {
    if bytes.len() >= 3 && bytes[0..3] == [0xEF, 0xBB, 0xBF] {
        // UTF-8 BOM found - this is acceptable but not ideal
        return Ok(());
    }

    if bytes.len() >= 2 && (bytes[0..2] == [0xFF, 0xFE] || bytes[0..2] == [0xFE, 0xFF]) {
        return Err(validation_error(
            "UTF-16 BOM detected - ASS files should be UTF-8",
        ));
    }

    if bytes.len() >= 4
        && (bytes[0..4] == [0xFF, 0xFE, 0x00, 0x00] || bytes[0..4] == [0x00, 0x00, 0xFE, 0xFF])
    {
        return Err(validation_error(
            "UTF-32 BOM detected - ASS files should be UTF-8",
        ));
    }

    Ok(())
}

/// Check for common encoding issues in ASS content
///
/// Performs heuristic checks for common encoding problems that can occur
/// when ASS files are saved with incorrect encoding settings.
///
/// # Arguments
///
/// * `text` - Text content to analyze
///
/// # Returns
///
/// `Ok(())` if no issues detected, validation error with suggestions if problems found
#[allow(dead_code)]
pub fn detect_encoding_issues(text: &str) -> Result<(), CoreError> {
    // Check for common mojibake patterns that indicate encoding issues
    if text.contains('\u{FFFD}') {
        return Err(validation_error(
            "Replacement characters (�) detected - possible encoding corruption",
        ));
    }

    // Check for suspicious character patterns
    let suspicious_count = text.chars().filter(|&c| {
        // Characters often seen in encoding mishaps
        matches!(c, '\u{00C0}'..='\u{00FF}' if text.chars().filter(|&x| x == c).count() > text.len() / 20)
    }).count();

    if suspicious_count > text.len() / 10 {
        return Err(validation_error(
            "Suspicious character patterns detected - possible encoding mismatch",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn encoding_issues_replacement_chars() {
        let text_with_replacement = "Hello � World";
        assert!(detect_encoding_issues(text_with_replacement).is_err());
    }

    #[test]
    fn encoding_issues_clean_text() {
        let clean_text = "Hello World";
        assert!(detect_encoding_issues(clean_text).is_ok());
    }
}
