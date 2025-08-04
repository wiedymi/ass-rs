//! Format validation error utilities for ASS-RS
//!
//! Provides specialized error creation and validation functions for common
//! ASS format types including colors, numeric values, and time formats.
//! Focuses on providing detailed error context for user feedback.

use super::CoreError;
use alloc::format;
use core::fmt;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{format};

/// Create color format error with detailed context
///
/// Generates a `CoreError::InvalidColor` with descriptive message about
/// the invalid color format encountered.
///
/// # Arguments
///
/// * `format` - The invalid color format string
///
/// # Examples
///
/// ```rust
/// use ass_core::utils::errors::{invalid_color, CoreError};
///
/// let error = invalid_color("invalid_color");
/// assert!(matches!(error, CoreError::InvalidColor(_)));
/// ```
pub fn invalid_color<T: fmt::Display>(format: T) -> CoreError {
    CoreError::InvalidColor(format!("{format}"))
}

/// Create numeric parsing error with value and reason
///
/// Generates a `CoreError::InvalidNumeric` with both the invalid value
/// and the reason parsing failed for better debugging.
///
/// # Arguments
///
/// * `value` - The value that failed to parse
/// * `reason` - Description of why parsing failed
pub fn invalid_numeric<T: fmt::Display>(value: T, reason: &str) -> CoreError {
    CoreError::InvalidNumeric(format!("'{value}': {reason}"))
}

/// Create time format error with time and reason
///
/// Generates a `CoreError::InvalidTime` with the invalid time string
/// and explanation of the format issue.
///
/// # Arguments
///
/// * `time` - The invalid time format string
/// * `reason` - Description of the format issue
pub fn invalid_time<T: fmt::Display>(time: T, reason: &str) -> CoreError {
    CoreError::InvalidTime(format!("'{time}': {reason}"))
}

/// Validate ASS color format
///
/// Checks if a string matches valid ASS color formats:
/// - &HBBGGRR (hexadecimal with transparency)
/// - &HBBGGRR& (alternate format)
/// - Decimal color values
///
/// # Returns
///
/// `Ok(())` if valid, error with suggestion if invalid
///
/// # Errors
///
/// Returns an error if the color format is invalid or cannot be parsed.
pub fn validate_color_format(color: &str) -> Result<(), CoreError> {
    let trimmed = color.trim();

    if trimmed.is_empty() {
        return Err(invalid_color("Empty color value"));
    }

    // Check for hex format (&H...)
    if trimmed.starts_with("&H") || trimmed.starts_with("&h") {
        let hex_part = if trimmed.ends_with('&') {
            &trimmed[2..trimmed.len() - 1]
        } else {
            &trimmed[2..]
        };

        if hex_part.len() != 6 && hex_part.len() != 8 {
            return Err(invalid_color(format!(
                "Hex color '{trimmed}' must be 6 or 8 characters after &H"
            )));
        }

        if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(invalid_color(format!(
                "Invalid hex digits in color '{trimmed}'"
            )));
        }
    } else {
        // Try parsing as decimal
        if trimmed.parse::<u32>().is_err() {
            return Err(invalid_color(format!(
                "Color '{trimmed}' is neither valid hex (&HBBGGRR) nor decimal"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_error_creation() {
        let error = invalid_color("invalid");
        assert!(matches!(error, CoreError::InvalidColor(_)));
    }

    #[test]
    fn numeric_error_creation() {
        let error = invalid_numeric("abc", "not a number");
        assert!(matches!(error, CoreError::InvalidNumeric(_)));
    }

    #[test]
    fn time_error_creation() {
        let error = invalid_time("invalid", "wrong format");
        assert!(matches!(error, CoreError::InvalidTime(_)));
    }

    #[test]
    fn validate_hex_color() {
        assert!(validate_color_format("&H00FF00FF").is_ok());
        assert!(validate_color_format("&H00FF00FF&").is_ok());
        assert!(validate_color_format("&h00ff00ff").is_ok());
        assert!(validate_color_format("&HFFFFFF").is_ok());
    }

    #[test]
    fn validate_decimal_color() {
        assert!(validate_color_format("16777215").is_ok()); // White
        assert!(validate_color_format("0").is_ok()); // Black
    }

    #[test]
    fn invalid_hex_color() {
        assert!(validate_color_format("&HGG0000").is_err());
        assert!(validate_color_format("&H123").is_err()); // Too short
        assert!(validate_color_format("&H123456789").is_err()); // Too long
    }

    #[test]
    fn invalid_color_format() {
        assert!(validate_color_format("").is_err());
        assert!(validate_color_format("invalid").is_err());
        assert!(validate_color_format("#FF0000").is_err()); // Wrong prefix
    }
}
