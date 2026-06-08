//! `From` conversions into [`CoreError`] from related error types.
//!
//! Centralizes the trait implementations that let foreign error types
//! propagate into [`CoreError`] via the `?` operator.

use super::CoreError;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::format;

/// Convert from parser errors
impl From<crate::parser::ParseError> for CoreError {
    fn from(err: crate::parser::ParseError) -> Self {
        Self::Parse(err)
    }
}

/// Convert from standard I/O errors (when std is available)
#[cfg(feature = "std")]
impl From<std::io::Error> for CoreError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(format!("{err}"))
    }
}

/// Convert from `core::str::Utf8Error`
impl From<::core::str::Utf8Error> for CoreError {
    fn from(err: ::core::str::Utf8Error) -> Self {
        Self::Utf8Error {
            position: 0, // Position not available from Utf8Error
            message: format!("{err}"),
        }
    }
}

/// Convert from integer parse errors
impl From<::core::num::ParseIntError> for CoreError {
    fn from(err: ::core::num::ParseIntError) -> Self {
        Self::InvalidNumeric(format!("Integer parse error: {err}"))
    }
}

/// Convert from float parse errors
impl From<::core::num::ParseFloatError> for CoreError {
    fn from(err: ::core::num::ParseFloatError) -> Self {
        Self::InvalidNumeric(format!("Float parse error: {err}"))
    }
}
