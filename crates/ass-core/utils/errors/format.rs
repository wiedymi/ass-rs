//! Format validation error utilities for ASS-RS
//!
//! Provides specialized error creation and validation functions for common
//! ASS format types including colors, numeric values, and time formats.
//! Focuses on providing detailed error context for user feedback.

use super::CoreError;
use alloc::format;
use core::fmt;

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

/// Validate numeric value within range
///
/// Checks if a string parses to a valid number within the specified range.
/// Provides detailed error messages for common parsing failures.
#[allow(dead_code)]
pub fn validate_numeric_range<T>(value: &str, min: T, max: T) -> Result<T, CoreError>
where
    T: core::str::FromStr + PartialOrd + fmt::Display + Copy,
    T::Err: fmt::Display,
{
    let parsed = value
        .parse::<T>()
        .map_err(|e| invalid_numeric(value, &format!("Parse error: {e}")))?;

    if parsed < min {
        return Err(invalid_numeric(
            value,
            &format!("Value {parsed} below minimum {min}"),
        ));
    }

    if parsed > max {
        return Err(invalid_numeric(
            value,
            &format!("Value {parsed} above maximum {max}"),
        ));
    }

    Ok(parsed)
}

/// Validate ASS time format (H:MM:SS.CS)
///
/// Checks if string matches ASS time format and components are valid.
/// Provides specific feedback about which part of the format is wrong.
#[allow(dead_code)]
pub fn validate_time_format(time: &str) -> Result<(), CoreError> {
    let trimmed = time.trim();

    if trimmed.is_empty() {
        return Err(invalid_time(trimmed, "Empty time value"));
    }

    let parts: Vec<&str> = trimmed.split(':').collect();
    if parts.len() != 3 {
        return Err(invalid_time(
            trimmed,
            "Must be in format H:MM:SS.CS (3 parts separated by ':')",
        ));
    }

    // Validate hours
    if parts[0].parse::<u32>().is_err() {
        return Err(invalid_time(trimmed, "Invalid hours component"));
    }

    // Validate minutes
    let minutes = parts[1]
        .parse::<u32>()
        .map_err(|_| invalid_time(trimmed, "Invalid minutes component"))?;
    if minutes >= 60 {
        return Err(invalid_time(trimmed, "Minutes must be 0-59"));
    }

    // Validate seconds.centiseconds
    let sec_parts: Vec<&str> = parts[2].split('.').collect();
    if sec_parts.len() != 2 {
        return Err(invalid_time(
            trimmed,
            "Seconds must include centiseconds (SS.CS)",
        ));
    }

    let seconds = sec_parts[0]
        .parse::<u32>()
        .map_err(|_| invalid_time(trimmed, "Invalid seconds component"))?;
    if seconds >= 60 {
        return Err(invalid_time(trimmed, "Seconds must be 0-59"));
    }

    let centiseconds = sec_parts[1]
        .parse::<u32>()
        .map_err(|_| invalid_time(trimmed, "Invalid centiseconds component"))?;
    if centiseconds >= 100 {
        return Err(invalid_time(trimmed, "Centiseconds must be 0-99"));
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

    #[test]
    fn validate_numeric_in_range() {
        assert_eq!(validate_numeric_range("50", 0, 100).unwrap(), 50);
        assert_eq!(validate_numeric_range("0", 0, 100).unwrap(), 0);
        assert_eq!(validate_numeric_range("100", 0, 100).unwrap(), 100);
    }

    #[test]
    fn numeric_out_of_range() {
        assert!(validate_numeric_range("150", 0, 100).is_err());
        assert!(validate_numeric_range("-10", 0, 100).is_err());
    }

    #[test]
    fn numeric_parse_error() {
        assert!(validate_numeric_range::<i32>("abc", 0, 100).is_err());
        assert!(validate_numeric_range::<f32>("", 0.0, 100.0).is_err());
    }

    #[test]
    fn validate_ass_time() {
        assert!(validate_time_format("0:00:00.00").is_ok());
        assert!(validate_time_format("1:23:45.67").is_ok());
        assert!(validate_time_format("12:59:59.99").is_ok());
    }

    #[test]
    fn invalid_time_format() {
        assert!(validate_time_format("").is_err());
        assert!(validate_time_format("1:23").is_err()); // Missing seconds
        assert!(validate_time_format("1:23:45").is_err()); // Missing centiseconds
        assert!(validate_time_format("1:60:45.00").is_err()); // Invalid minutes
        assert!(validate_time_format("1:23:60.00").is_err()); // Invalid seconds
        assert!(validate_time_format("1:23:45.100").is_err()); // Invalid centiseconds
    }
}
