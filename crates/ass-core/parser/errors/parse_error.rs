//! Primary parse error type for ASS script parsing
//!
//! Contains the main ParseError enum representing unrecoverable parsing errors
//! that prevent script construction. These errors indicate fundamental issues
//! with the script structure or content that cannot be recovered from.

use alloc::string::String;
use core::fmt;

#[cfg(feature = "std")]
use thiserror::Error;

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
#[derive(Debug, Clone, PartialEq)]
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
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::ExpectedSectionHeader { line } => {
                write!(
                    f,
                    "Expected section header like [Script Info] at line {}",
                    line
                )
            }
            ParseError::UnclosedSectionHeader { line } => {
                write!(f, "Unclosed section header at line {}: missing ']'", line)
            }
            ParseError::UnknownSection { section, line } => {
                write!(f, "Unknown section '{}' at line {}", section, line)
            }
            ParseError::InvalidFieldFormat { line } => {
                write!(
                    f,
                    "Invalid field format at line {}: expected 'key: value'",
                    line
                )
            }
            ParseError::MissingRequiredField { field, section } => {
                write!(
                    f,
                    "Missing required field '{}' in {} section",
                    field, section
                )
            }
            ParseError::InvalidFormatLine { line, reason } => {
                write!(f, "Invalid format line at line {}: {}", line, reason)
            }
            ParseError::FieldCountMismatch {
                line,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Field count mismatch at line {}: expected {}, found {}",
                    line, expected, found
                )
            }
            ParseError::InvalidTimeFormat { time, line, reason } => {
                write!(
                    f,
                    "Invalid time format '{}' at line {}: {}",
                    time, line, reason
                )
            }
            ParseError::InvalidColorFormat {
                color,
                line,
                reason,
            } => {
                write!(
                    f,
                    "Invalid color format '{}' at line {}: {}",
                    color, line, reason
                )
            }
            ParseError::InvalidNumericValue {
                value,
                line,
                reason,
            } => {
                write!(
                    f,
                    "Invalid numeric value '{}' at line {}: {}",
                    value, line, reason
                )
            }
            ParseError::InvalidStyleOverride { line, reason } => {
                write!(f, "Invalid style override at line {}: {}", line, reason)
            }
            ParseError::InvalidDrawingCommand { line, reason } => {
                write!(f, "Invalid drawing command at line {}: {}", line, reason)
            }
            ParseError::UuDecodeError { line, reason } => {
                write!(f, "UU-decode error at line {}: {}", line, reason)
            }
            ParseError::Utf8Error { position, reason } => {
                write!(f, "UTF-8 encoding error at byte {}: {}", position, reason)
            }
            ParseError::UnsupportedVersion { version } => {
                write!(
                    f,
                    "Unsupported script version '{}': expected v4.00+ or compatible",
                    version
                )
            }
            ParseError::CircularStyleReference { chain } => {
                write!(f, "Circular style reference detected: {}", chain)
            }
            ParseError::MaxNestingDepth { line, limit } => {
                write!(
                    f,
                    "Maximum nesting depth exceeded at line {}: limit is {}",
                    line, limit
                )
            }
            ParseError::InputTooLarge { size, limit } => {
                write!(f, "Input size {} bytes exceeds limit {} bytes", size, limit)
            }
            ParseError::IoError { message } => {
                write!(f, "I/O error during parsing: {}", message)
            }
            ParseError::OutOfMemory { message } => {
                write!(f, "Memory allocation failed: {}", message)
            }
            ParseError::InternalError { line, message } => {
                write!(f, "Internal parser error at line {}: {}", line, message)
            }
        }
    }
}

#[cfg(not(feature = "std"))]
impl core::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_display_expected_section_header() {
        let error = ParseError::ExpectedSectionHeader { line: 5 };
        let message = format!("{}", error);
        assert!(message.contains("line 5"));
        assert!(message.contains("Expected section header"));
        assert!(message.contains("[Script Info]"));
    }

    #[test]
    fn parse_error_display_unclosed_section_header() {
        let error = ParseError::UnclosedSectionHeader { line: 10 };
        let message = format!("{}", error);
        assert!(message.contains("line 10"));
        assert!(message.contains("Unclosed section header"));
        assert!(message.contains("missing ']'"));
    }

    #[test]
    fn parse_error_display_unknown_section() {
        let error = ParseError::UnknownSection {
            section: "BadSection".to_string(),
            line: 15,
        };
        let message = format!("{}", error);
        assert!(message.contains("line 15"));
        assert!(message.contains("Unknown section"));
        assert!(message.contains("BadSection"));
    }

    #[test]
    fn parse_error_display_field_count_mismatch() {
        let error = ParseError::FieldCountMismatch {
            line: 20,
            expected: 5,
            found: 3,
        };
        let message = format!("{}", error);
        assert!(message.contains("line 20"));
        assert!(message.contains("expected 5"));
        assert!(message.contains("found 3"));
        assert!(message.contains("Field count mismatch"));
    }

    #[test]
    fn parse_error_display_invalid_time_format() {
        let error = ParseError::InvalidTimeFormat {
            time: "invalid:time".to_string(),
            line: 25,
            reason: "bad format".to_string(),
        };
        let message = format!("{}", error);
        assert!(message.contains("line 25"));
        assert!(message.contains("invalid:time"));
        assert!(message.contains("bad format"));
        assert!(message.contains("Invalid time format"));
    }

    #[test]
    fn parse_error_display_invalid_color_format() {
        let error = ParseError::InvalidColorFormat {
            color: "&HINVALID&".to_string(),
            line: 30,
            reason: "not hex".to_string(),
        };
        let message = format!("{}", error);
        assert!(message.contains("line 30"));
        assert!(message.contains("&HINVALID&"));
        assert!(message.contains("not hex"));
        assert!(message.contains("Invalid color format"));
    }

    #[test]
    fn parse_error_display_utf8_error() {
        let error = ParseError::Utf8Error {
            position: 100,
            reason: "invalid byte sequence".to_string(),
        };
        let message = format!("{}", error);
        assert!(message.contains("byte 100"));
        assert!(message.contains("invalid byte sequence"));
        assert!(message.contains("UTF-8 encoding error"));
    }

    #[test]
    fn parse_error_display_unsupported_version() {
        let error = ParseError::UnsupportedVersion {
            version: "v3.00".to_string(),
        };
        let message = format!("{}", error);
        assert!(message.contains("v3.00"));
        assert!(message.contains("Unsupported script version"));
        assert!(message.contains("v4.00+"));
    }

    #[test]
    fn parse_error_display_circular_style_reference() {
        let error = ParseError::CircularStyleReference {
            chain: "StyleA -> StyleB -> StyleA".to_string(),
        };
        let message = format!("{}", error);
        assert!(message.contains("StyleA -> StyleB -> StyleA"));
        assert!(message.contains("Circular style reference"));
    }

    #[test]
    fn parse_error_display_input_too_large() {
        let error = ParseError::InputTooLarge {
            size: 1_000_000,
            limit: 500_000,
        };
        let message = format!("{}", error);
        assert!(message.contains("1000000"));
        assert!(message.contains("500000"));
        assert!(message.contains("Input size"));
        assert!(message.contains("exceeds limit"));
    }

    #[test]
    fn parse_error_clone_and_equality() {
        let error1 = ParseError::ExpectedSectionHeader { line: 42 };
        let error2 = error1.clone();
        assert_eq!(error1, error2);

        let error3 = ParseError::ExpectedSectionHeader { line: 43 };
        assert_ne!(error1, error3);
    }

    #[test]
    fn parse_error_debug() {
        let error = ParseError::InvalidFieldFormat { line: 99 };
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidFieldFormat"));
        assert!(debug_str.contains("99"));
    }
}
