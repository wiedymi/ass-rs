//! UTF-8 and ASS character validation checks
//!
//! Provides detailed UTF-8 validation with position-specific error reporting
//! and content validation against the ASS-permitted character set.

use crate::utils::CoreError;
use alloc::format;
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
