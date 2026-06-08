//! Error constructors for text encoding failures
//!
//! Provides helpers that build structured `CoreError` values for UTF-8 and
//! generic text validation failures, attaching position and context details.

use super::super::CoreError;
use alloc::{format, string::String};
use core::fmt;

/// Create UTF-8 encoding error with position information
///
/// Generates a `CoreError::Utf8Error` with detailed position and context
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
#[must_use]
pub const fn utf8_error(position: usize, message: String) -> CoreError {
    CoreError::Utf8Error { position, message }
}

/// Create validation error for text content
///
/// Generates a `CoreError::Validation` for content that fails ASS-specific
/// text validation rules (e.g., contains invalid control characters).
///
/// # Arguments
///
/// * `message` - Description of the validation failure
pub fn validation_error<T: fmt::Display>(message: T) -> CoreError {
    CoreError::Validation(format!("{message}"))
}
