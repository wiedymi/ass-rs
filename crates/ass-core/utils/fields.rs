//! Field value parsing, validation, and normalization helpers.
//!
//! Provides numeric parsing with ASS-friendly error messages along with
//! whitespace normalization and name validation for ASS field values.

#[cfg(not(feature = "std"))]
use alloc::format;
use core::fmt;
#[cfg(feature = "std")]
use std::format;

use super::CoreError;

/// Parse numeric value from ASS field with validation
///
/// Handles integer and floating-point parsing with ASS-specific validation.
/// Provides better error messages than standard parsing.
///
/// # Errors
///
/// Returns an error if the string cannot be parsed as the target numeric type.
pub fn parse_numeric<T>(value_str: &str) -> Result<T, CoreError>
where
    T: core::str::FromStr,
    T::Err: fmt::Display,
{
    value_str
        .trim()
        .parse()
        .map_err(|e| CoreError::InvalidNumeric(format!("Failed to parse '{value_str}': {e}")))
}

/// Trim and normalize whitespace in ASS field values
///
/// ASS fields may have inconsistent whitespace that should be normalized
/// while preserving intentional spacing in text content.
#[must_use]
pub fn normalize_field_value(value: &str) -> &str {
    value.trim()
}

/// Check if string contains only valid ASS characters
///
/// ASS has restrictions on certain characters in names and style definitions.
#[must_use]
pub fn validate_ass_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains(',') // Comma is field separator
        && !name.contains(':') // Colon is key-value separator
        && !name.contains('{') // Override block start
        && !name.contains('}') // Override block end
        && name.chars().all(|c| !c.is_control() || c == '\t')
}
