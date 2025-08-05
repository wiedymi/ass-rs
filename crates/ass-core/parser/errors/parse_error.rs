//! Primary parse error type for ASS script parsing
//!
//! Contains the main `ParseError` enum representing unrecoverable parsing errors
//! that prevent script construction. These errors indicate fundamental issues
//! with the script structure or content that cannot be recovered from.

use alloc::string::String;
use core::fmt;

#[cfg(feature = "std")]
use thiserror::Error;

use crate::parser::SectionType;

/// Primary parse error type for ASS scripts
///
/// Represents unrecoverable parsing errors that prevent script construction.
/// Use `ParseIssue` for recoverable warnings and errors that allow continued parsing.
///
/// # Error Categories
///
/// - **Structure errors**: Missing headers, invalid sections
/// - **Format errors**: Invalid field formats, mismatched counts
/// - **Content errors**: Invalid time/color formats, bad numeric values
/// - **System errors**: Memory limits, UTF-8 issues, I/O problems
#[cfg_attr(feature = "std", derive(Error))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Expected section header but found something else
    ExpectedSectionHeader { line: usize },

    /// Section header not properly closed
    UnclosedSectionHeader { line: usize },

    /// Unknown section type encountered
    UnknownSection { section: String, line: usize },

    /// Invalid field format in section
    InvalidFieldFormat { line: usize },

    /// Missing required field in section
    MissingRequiredField { field: String, section: String },

    /// Invalid format line for styles or events
    InvalidFormatLine { line: usize, reason: String },

    /// Mismatched field count in data line
    FieldCountMismatch {
        line: usize,
        expected: usize,
        found: usize,
    },

    /// Invalid time format
    InvalidTimeFormat {
        time: String,
        line: usize,
        reason: String,
    },

    /// Invalid color format
    InvalidColorFormat {
        color: String,
        line: usize,
        reason: String,
    },

    /// Invalid numeric value
    InvalidNumericValue {
        value: String,
        line: usize,
        reason: String,
    },

    /// Invalid style override syntax
    InvalidStyleOverride { line: usize, reason: String },

    /// Invalid drawing command syntax
    InvalidDrawingCommand { line: usize, reason: String },

    /// UU-encoding decode error
    UuDecodeError { line: usize, reason: String },

    /// UTF-8 encoding error
    Utf8Error { position: usize, reason: String },

    /// Script version not supported
    UnsupportedVersion { version: String },

    /// Circular style reference detected
    CircularStyleReference { chain: String },

    /// Maximum nesting depth exceeded
    MaxNestingDepth { line: usize, limit: usize },

    /// Input too large for processing
    InputTooLarge { size: usize, limit: usize },

    /// Generic I/O error during streaming parse
    IoError { message: String },

    /// Memory allocation failure
    OutOfMemory { message: String },

    /// Internal parser state corruption
    InternalError { line: usize, message: String },

    /// Invalid event type
    InvalidEventType { line: usize },

    /// Insufficient fields in line
    InsufficientFields {
        expected: usize,
        found: usize,
        line: usize,
    },

    /// Missing format specification
    MissingFormat,

    /// Section not found
    SectionNotFound,

    /// Index out of bounds
    IndexOutOfBounds,

    /// Unsupported section type for operation
    UnsupportedSection(SectionType),
}

impl ParseError {
    /// Format structural errors
    fn fmt_structural(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectedSectionHeader { line } => {
                write!(
                    f,
                    "Expected section header like [Script Info] at line {line}"
                )
            }
            Self::UnclosedSectionHeader { line } => {
                write!(f, "Unclosed section header at line {line}: missing ']'")
            }
            Self::UnknownSection { section, line } => {
                write!(f, "Unknown section '{section}' at line {line}")
            }
            Self::InvalidFieldFormat { line } => {
                write!(
                    f,
                    "Invalid field format at line {line}: expected 'key: value'"
                )
            }
            Self::MissingRequiredField { field, section } => {
                write!(f, "Missing required field '{field}' in {section} section")
            }
            Self::InvalidFormatLine { line, reason } => {
                write!(f, "Invalid format line at line {line}: {reason}")
            }
            Self::FieldCountMismatch {
                line,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Field count mismatch at line {line}: expected {expected}, found {found}"
                )
            }
            _ => Err(fmt::Error),
        }
    }

    /// Format content errors
    fn fmt_content(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTimeFormat { time, line, reason } => {
                write!(f, "Invalid time format '{time}' at line {line}: {reason}")
            }
            Self::InvalidColorFormat {
                color,
                line,
                reason,
            } => {
                write!(f, "Invalid color format '{color}' at line {line}: {reason}")
            }
            Self::InvalidNumericValue {
                value,
                line,
                reason,
            } => {
                write!(
                    f,
                    "Invalid numeric value '{value}' at line {line}: {reason}"
                )
            }
            Self::InvalidStyleOverride { line, reason } => {
                write!(f, "Invalid style override at line {line}: {reason}")
            }
            Self::InvalidDrawingCommand { line, reason } => {
                write!(f, "Invalid drawing command at line {line}: {reason}")
            }
            Self::UuDecodeError { line, reason } => {
                write!(f, "UU-decode error at line {line}: {reason}")
            }
            _ => Err(fmt::Error),
        }
    }

    /// Format system errors
    fn fmt_system(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Utf8Error { position, reason } => {
                write!(f, "UTF-8 encoding error at byte {position}: {reason}")
            }
            Self::UnsupportedVersion { version } => {
                write!(
                    f,
                    "Unsupported script version '{version}': expected v4.00+ or compatible"
                )
            }
            Self::CircularStyleReference { chain } => {
                write!(f, "Circular style reference detected: {chain}")
            }
            Self::MaxNestingDepth { line, limit } => {
                write!(
                    f,
                    "Maximum nesting depth exceeded at line {line}: limit is {limit}"
                )
            }
            Self::InputTooLarge { size, limit } => {
                write!(f, "Input size {size} bytes exceeds limit {limit} bytes")
            }
            Self::IoError { message } => {
                write!(f, "I/O error during parsing: {message}")
            }
            Self::OutOfMemory { message } => {
                write!(f, "Memory allocation failed: {message}")
            }
            Self::InternalError { line, message } => {
                write!(f, "Internal parser error at line {line}: {message}")
            }
            Self::InvalidEventType { line } => {
                write!(f, "Invalid event type at line {line}: expected Dialogue, Comment, Picture, Sound, Movie, or Command")
            }
            Self::InsufficientFields {
                expected,
                found,
                line,
            } => {
                write!(
                    f,
                    "Insufficient fields at line {line}: expected {expected}, found {found}"
                )
            }
            Self::MissingFormat => {
                write!(f, "Missing format specification for section")
            }
            Self::SectionNotFound => {
                write!(f, "Section not found")
            }
            Self::IndexOutOfBounds => {
                write!(f, "Index out of bounds")
            }
            Self::UnsupportedSection(section_type) => {
                write!(
                    f,
                    "Unsupported section type for operation: {section_type:?}"
                )
            }
            _ => Err(fmt::Error),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_structural(f)
            .or_else(|_| self.fmt_content(f))
            .or_else(|_| self.fmt_system(f))
    }
}
