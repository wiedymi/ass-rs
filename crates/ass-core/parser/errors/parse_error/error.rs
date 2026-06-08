//! `ParseError` enum definition for unrecoverable ASS parsing failures
//!
//! Declares the variants representing errors that prevent script construction.
//! Display formatting for these variants lives in the sibling `formatting` module.

use alloc::string::String;

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
