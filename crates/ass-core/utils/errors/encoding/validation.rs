//! Validation routines for ASS text and byte encoding
//!
//! Provides detailed UTF-8 validation, ASS-specific character checks, and
//! byte-order-mark (BOM) inspection used to surface encoding problems early.

use super::super::CoreError;
use super::creation::{utf8_error, validation_error};
use alloc::{format, string::String};

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
///
/// # Errors
///
/// Returns an error if the byte slice contains invalid UTF-8 sequences.
pub fn validate_utf8_detailed(bytes: &[u8]) -> Result<(), CoreError> {
    match core::str::from_utf8(bytes) {
        Ok(_) => Ok(()),
        Err(err) => {
            let position = err.valid_up_to();
            let message = err.error_len().map_or_else(
                || format!("Incomplete UTF-8 sequence at position {position}"),
                |len| format!("Invalid UTF-8 sequence of {len} bytes at position {position}"),
            );

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
///
/// # Errors
///
/// Returns an error if the text contains invalid characters for ASS format.
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
pub(super) fn is_valid_ass_char(ch: char) -> bool {
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
///
/// # Errors
///
/// Returns an error if UTF-16 BOM is detected or other BOM issues are found
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

    // Check for partial BOM sequences that could indicate encoding issues
    if bytes.len() >= 2 && bytes[0..2] == [0xEF, 0xBB] {
        return Err(validation_error(
            "Partial UTF-8 BOM detected - file may be corrupted or incorrectly encoded",
        ));
    }

    if !bytes.is_empty() && bytes[0] == 0xEF && (bytes.len() == 1 || bytes[1] != 0xBB) {
        return Err(validation_error(
            "Suspicious byte sequence that could be partial BOM - check file encoding",
        ));
    }

    Ok(())
}
