//! Inherent constructors and introspection methods for `CoreError`.
//!
//! Provides convenience constructors plus helpers to classify errors by
//! recoverability, internal-bug status, and location information.

use super::CoreError;

use alloc::format;
use core::fmt;

#[cfg(not(feature = "std"))]
extern crate alloc;

impl CoreError {
    /// Create parse error from message
    pub fn parse<T: fmt::Display>(message: T) -> Self {
        Self::Parse(crate::parser::ParseError::IoError {
            message: format!("{message}"),
        })
    }

    /// Create internal error (indicates a bug)
    pub fn internal<T: fmt::Display>(message: T) -> Self {
        Self::Internal(format!("{message}"))
    }

    /// Check if error is recoverable
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        match self {
            Self::Parse(parse_err) => !matches!(
                parse_err,
                crate::parser::ParseError::OutOfMemory { .. }
                    | crate::parser::ParseError::InputTooLarge { .. }
                    | crate::parser::ParseError::InternalError { .. }
            ),
            Self::Tokenization(_)
            | Self::InvalidColor(_)
            | Self::InvalidNumeric(_)
            | Self::InvalidTime(_)
            | Self::Validation(_)
            | Self::Analysis(_)
            | Self::Plugin(_)
            | Self::Utf8Error { .. }
            | Self::Io(_)
            | Self::Config(_)
            | Self::FeatureNotSupported { .. }
            | Self::VersionIncompatible { .. } => true,

            Self::OutOfMemory(_)
            | Self::ResourceLimitExceeded { .. }
            | Self::SecurityViolation(_)
            | Self::Internal(_) => false,
        }
    }

    /// Check if error indicates a bug in the library
    #[must_use]
    pub const fn is_internal_bug(&self) -> bool {
        matches!(self, Self::Internal(_))
    }

    /// Get the underlying parse error if this is a parse error
    #[must_use]
    pub const fn as_parse_error(&self) -> Option<&crate::parser::ParseError> {
        match self {
            Self::Parse(parse_err) => Some(parse_err),
            _ => None,
        }
    }

    /// Get line number for errors that have location information
    #[must_use]
    pub const fn line_number(&self) -> Option<usize> {
        match self {
            Self::Parse(
                crate::parser::ParseError::ExpectedSectionHeader { line }
                | crate::parser::ParseError::UnclosedSectionHeader { line }
                | crate::parser::ParseError::UnknownSection { line, .. }
                | crate::parser::ParseError::InvalidFieldFormat { line }
                | crate::parser::ParseError::InvalidFormatLine { line, .. }
                | crate::parser::ParseError::FieldCountMismatch { line, .. }
                | crate::parser::ParseError::InvalidTimeFormat { line, .. }
                | crate::parser::ParseError::InvalidColorFormat { line, .. }
                | crate::parser::ParseError::InvalidNumericValue { line, .. }
                | crate::parser::ParseError::InvalidStyleOverride { line, .. }
                | crate::parser::ParseError::InvalidDrawingCommand { line, .. }
                | crate::parser::ParseError::UuDecodeError { line, .. }
                | crate::parser::ParseError::MaxNestingDepth { line, .. }
                | crate::parser::ParseError::InternalError { line, .. },
            ) => Some(*line),
            Self::Utf8Error { position, .. } => Some(*position),
            _ => None,
        }
    }

    /// Check if this is a specific type of parse error
    #[must_use]
    pub fn is_parse_error_type(&self, error_type: &str) -> bool {
        match self {
            Self::Parse(parse_err) => matches!(
                (error_type, parse_err),
                (
                    "section_header",
                    crate::parser::ParseError::ExpectedSectionHeader { .. }
                ) | (
                    "unclosed_header",
                    crate::parser::ParseError::UnclosedSectionHeader { .. }
                ) | (
                    "unknown_section",
                    crate::parser::ParseError::UnknownSection { .. }
                ) | (
                    "field_format",
                    crate::parser::ParseError::InvalidFieldFormat { .. }
                        | crate::parser::ParseError::FieldCountMismatch { .. }
                ) | (
                    "time_format",
                    crate::parser::ParseError::InvalidTimeFormat { .. }
                ) | (
                    "color_format",
                    crate::parser::ParseError::InvalidColorFormat { .. }
                ) | (
                    "numeric_value",
                    crate::parser::ParseError::InvalidNumericValue { .. }
                ) | ("utf8", crate::parser::ParseError::Utf8Error { .. })
            ),
            _ => false,
        }
    }
}
