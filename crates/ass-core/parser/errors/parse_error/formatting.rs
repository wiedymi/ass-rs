//! Display formatting for `ParseError` variants
//!
//! Groups the human-readable messages for each error category and wires them
//! together through the `core::fmt::Display` implementation for `ParseError`.

use core::fmt;

use super::ParseError;

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
