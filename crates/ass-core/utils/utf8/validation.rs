//! UTF-8 validation and recovery utilities for ASS subtitle processing
//!
//! Provides detailed UTF-8 validation with position-specific error reporting
//! and recovery mechanisms for handling invalid UTF-8 sequences. Designed
//! for robust processing of subtitle files with various encoding issues.
//!
//! # Examples
//!
//! ```rust
//! use ass_core::utils::utf8::{validate_utf8, recover_utf8};
//!
//! let valid_text = "Hello, 世界! 🎵";
//! assert!(validate_utf8(valid_text.as_bytes()).is_ok());
//!
//! let invalid_bytes = &[b'H', b'i', 0xFF, b'!'];
//! let (recovered, replacements) = recover_utf8(invalid_bytes);
//! assert_eq!(recovered, "Hi�!");
//! assert_eq!(replacements, 1);
//! ```

use crate::utils::CoreError;
use alloc::{
    format,
    string::{String, ToString},
};
use core::str;

/// Validate UTF-8 with detailed error information
///
/// Provides more detailed error reporting than standard UTF-8 validation,
/// including the position and nature of encoding errors. Essential for
/// processing subtitle files with encoding issues.
///
/// # Arguments
///
/// * `bytes` - Byte sequence to validate
///
/// # Returns
///
/// `Ok(())` if valid UTF-8, detailed error with position if invalid
///
/// # Examples
///
/// ```rust
/// # use ass_core::utils::utf8::validate_utf8;
/// let valid_text = "Hello, 世界!";
/// assert!(validate_utf8(valid_text.as_bytes()).is_ok());
///
/// let invalid_bytes = &[0xFF, 0xFE, 0x80];
/// assert!(validate_utf8(invalid_bytes).is_err());
/// ```
///
/// # Errors
///
/// Returns an error if the byte slice contains invalid UTF-8 sequences.
pub fn validate_utf8(bytes: &[u8]) -> Result<(), CoreError> {
    match str::from_utf8(bytes) {
        Ok(_) => Ok(()),
        Err(err) => {
            let position = err.valid_up_to();
            let message = err.error_len().map_or_else(
                || format!("Incomplete UTF-8 sequence at position {position}"),
                |len| format!("Invalid UTF-8 sequence of {len} bytes at position {position}"),
            );

            Err(CoreError::utf8_error(position, message))
        }
    }
}

/// Attempt to recover from UTF-8 errors by replacing invalid sequences
///
/// Returns valid UTF-8 text with invalid sequences replaced by the Unicode
/// replacement character (�). Also returns the number of replacements made
/// for diagnostic purposes.
///
/// # Arguments
///
/// * `bytes` - Byte sequence that may contain invalid UTF-8
///
/// # Returns
///
/// Tuple of (`recovered_text`, `replacement_count`)
///
/// # Examples
///
/// ```rust
/// # use ass_core::utils::utf8::recover_utf8;
/// let valid_text = "Hello, World!";
/// let (recovered, replacements) = recover_utf8(valid_text.as_bytes());
/// assert_eq!(recovered, "Hello, World!");
/// assert_eq!(replacements, 0);
///
/// let invalid_bytes = &[b'H', b'i', 0xFF, b'!'];
/// let (recovered, replacements) = recover_utf8(invalid_bytes);
/// assert_eq!(recovered, "Hi�!");
/// assert_eq!(replacements, 1);
/// ```
#[must_use]
pub fn recover_utf8(bytes: &[u8]) -> (String, usize) {
    str::from_utf8(bytes).map_or_else(
        |_| {
            let recovered = String::from_utf8_lossy(bytes);
            let replacements = recovered.matches('\u{FFFD}').count();
            (recovered.into_owned(), replacements)
        },
        |s| (s.to_string(), 0),
    )
}

/// Check if text contains only valid ASS characters
///
/// ASS files should generally contain only printable characters plus
/// specific control characters like tabs and newlines. This function
/// validates character content according to ASS specification guidelines.
///
/// # Arguments
///
/// * `text` - Text content to validate
///
/// # Returns
///
/// `true` if all characters are valid for ASS content
#[must_use]
pub fn is_valid_ass_text(text: &str) -> bool {
    text.chars().all(|c| {
        c.is_ascii_graphic()  // Printable ASCII
            || c == ' '       // Space
            || c == '\t'      // Tab
            || c == '\n'      // Newline
            || c == '\r'      // Carriage return
            || (!c.is_ascii() && !c.is_control()) // Non-ASCII printable (Unicode)
    })
}

/// Truncate text at UTF-8 character boundary
///
/// Safely truncates text to the specified byte length without breaking
/// UTF-8 character sequences. Essential for handling length limits
/// while maintaining valid UTF-8 encoding.
///
/// # Arguments
///
/// * `text` - Input text to truncate
/// * `max_bytes` - Maximum byte length
///
/// # Returns
///
/// Tuple of (`truncated_text`, `was_truncated`)
///
/// # Examples
///
/// ```rust
/// # use ass_core::utils::utf8::truncate_at_char_boundary;
/// let text = "Hello World";
/// let (truncated, was_truncated) = truncate_at_char_boundary(text, 5);
/// assert_eq!(truncated, "Hello");
/// assert!(was_truncated);
///
/// let text = "Hello 世界";
/// let (truncated, was_truncated) = truncate_at_char_boundary(text, 8);
/// assert_eq!(truncated, "Hello "); // Stops before the Unicode character
/// assert!(was_truncated);
/// ```
#[must_use]
pub fn truncate_at_char_boundary(text: &str, max_bytes: usize) -> (&str, bool) {
    if text.len() <= max_bytes {
        return (text, false);
    }

    let mut boundary = max_bytes;
    while boundary > 0 && !text.is_char_boundary(boundary) {
        boundary -= 1;
    }

    (&text[..boundary], true)
}

/// Count replacement characters in text
///
/// Counts the number of Unicode replacement characters (�) in text,
/// which typically indicate encoding errors or data corruption.
/// Useful for assessing text quality and encoding issues.
///
/// # Arguments
///
/// * `text` - Text to analyze
///
/// # Returns
///
/// Number of replacement characters found
#[must_use]
pub fn count_replacement_chars(text: &str) -> usize {
    text.matches('\u{FFFD}').count()
}

#[cfg(test)]
mod tests {
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
}
