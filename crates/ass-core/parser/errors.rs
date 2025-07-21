//! Parser error types for ASS script parsing
//!
//! Provides comprehensive error handling for ASS subtitle script parsing with
//! detailed error messages and recovery information for interactive editing.
//!
//! # Error Philosophy
//!
//! - Prefer recovery over failure where possible
//! - Provide detailed location information for editor integration
//! - Group related errors for efficient handling
//! - Include suggestions for common mistakes

use alloc::{format, string::String, vec::Vec};
use core::fmt;

#[cfg(feature = "std")]
use thiserror::Error;

/// Primary parse error type for ASS scripts
///
/// Represents unrecoverable parsing errors that prevent script construction.
/// Use `ParseIssue` for recoverable warnings and errors.
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

#[cfg(not(feature = "std"))]
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

/// std compatible Display implementation
#[cfg(feature = "std")]
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

/// Parse issue severity levels for partial recovery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueSeverity {
    /// Information that may be useful but doesn't affect functionality
    Info,

    /// Warning about potential problems or non-standard usage
    Warning,

    /// Error that was recovered from but may affect rendering
    Error,

    /// Critical error that will likely cause rendering problems
    Critical,
}

impl fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueSeverity::Info => write!(f, "info"),
            IssueSeverity::Warning => write!(f, "warning"),
            IssueSeverity::Error => write!(f, "error"),
            IssueSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Parse issue for recoverable problems and warnings
///
/// Used for problems that don't prevent parsing but may affect
/// rendering quality or indicate potential script issues.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseIssue {
    /// Issue severity level
    pub severity: IssueSeverity,

    /// Issue category for filtering/grouping
    pub category: IssueCategory,

    /// Human-readable message
    pub message: String,

    /// Line number where issue occurred (1-based)
    pub line: usize,

    /// Column number where issue occurred (1-based)
    pub column: Option<usize>,

    /// Byte range in source where issue occurred
    pub span: Option<(usize, usize)>,

    /// Suggested fix or explanation
    pub suggestion: Option<String>,
}

/// Issue categories for filtering and editor integration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueCategory {
    /// Script structure issues
    Structure,

    /// Style definition problems
    Style,

    /// Event/dialogue issues
    Event,

    /// Timing-related problems
    Timing,

    /// Color format issues
    Color,

    /// Font/typography issues
    Font,

    /// Drawing command problems
    Drawing,

    /// Performance warnings
    Performance,

    /// Compatibility warnings
    Compatibility,

    /// Security warnings
    Security,

    /// General format issues
    Format,
}

impl fmt::Display for IssueCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueCategory::Structure => write!(f, "structure"),
            IssueCategory::Style => write!(f, "style"),
            IssueCategory::Event => write!(f, "event"),
            IssueCategory::Timing => write!(f, "timing"),
            IssueCategory::Color => write!(f, "color"),
            IssueCategory::Font => write!(f, "font"),
            IssueCategory::Drawing => write!(f, "drawing"),
            IssueCategory::Performance => write!(f, "performance"),
            IssueCategory::Compatibility => write!(f, "compatibility"),
            IssueCategory::Security => write!(f, "security"),
            IssueCategory::Format => write!(f, "format"),
        }
    }
}

impl ParseIssue {
    /// Create new parse issue with minimal information
    pub fn new(
        severity: IssueSeverity,
        category: IssueCategory,
        message: String,
        line: usize,
    ) -> Self {
        Self {
            severity,
            category,
            message,
            line,
            column: None,
            span: None,
            suggestion: None,
        }
    }

    /// Create issue with full location information
    pub fn with_location(
        severity: IssueSeverity,
        category: IssueCategory,
        message: String,
        line: usize,
        column: usize,
        span: (usize, usize),
    ) -> Self {
        Self {
            severity,
            category,
            message,
            line,
            column: Some(column),
            span: Some(span),
            suggestion: None,
        }
    }

    /// Add suggestion to existing issue
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    /// Create info-level issue
    pub fn info(category: IssueCategory, message: String, line: usize) -> Self {
        Self::new(IssueSeverity::Info, category, message, line)
    }

    /// Create warning-level issue
    pub fn warning(category: IssueCategory, message: String, line: usize) -> Self {
        Self::new(IssueSeverity::Warning, category, message, line)
    }

    /// Create error-level issue
    pub fn error(category: IssueCategory, message: String, line: usize) -> Self {
        Self::new(IssueSeverity::Error, category, message, line)
    }

    /// Create critical-level issue
    pub fn critical(category: IssueCategory, message: String, line: usize) -> Self {
        Self::new(IssueSeverity::Critical, category, message, line)
    }

    /// Format issue for display in editor or console
    pub fn format_for_display(&self) -> String {
        let location = if let Some(column) = self.column {
            format!("{}:{}", self.line, column)
        } else {
            format!("{}", self.line)
        };

        let mut result = format!(
            "[{}:{}] {}: {}",
            location, self.category, self.severity, self.message
        );

        if let Some(suggestion) = &self.suggestion {
            result.push_str(&format!("\n  Suggestion: {}", suggestion));
        }

        result
    }

    /// Check if this is a blocking error that should prevent further processing
    pub fn is_blocking(&self) -> bool {
        matches!(self.severity, IssueSeverity::Critical)
    }
}

/// Result type for operations that can produce parse issues
pub type ParseResult<T> = Result<T, ParseError>;

/// Parse result with accumulated issues for partial recovery
#[derive(Debug, Clone)]
pub struct ParseResultWithIssues<T> {
    /// The parsed result (if successful)
    pub result: ParseResult<T>,

    /// Accumulated parse issues
    pub issues: Vec<ParseIssue>,
}

impl<T> ParseResultWithIssues<T> {
    /// Create successful result with no issues
    pub fn ok(value: T) -> Self {
        Self {
            result: Ok(value),
            issues: Vec::new(),
        }
    }

    /// Create error result with no issues
    pub fn err(error: ParseError) -> Self {
        Self {
            result: Err(error),
            issues: Vec::new(),
        }
    }

    /// Create result with issues
    pub fn with_issues(result: ParseResult<T>, issues: Vec<ParseIssue>) -> Self {
        Self { result, issues }
    }

    /// Add issue to existing result
    pub fn add_issue(mut self, issue: ParseIssue) -> Self {
        self.issues.push(issue);
        self
    }

    /// Get only critical issues
    pub fn critical_issues(&self) -> Vec<&ParseIssue> {
        self.issues
            .iter()
            .filter(|issue| matches!(issue.severity, IssueSeverity::Critical))
            .collect()
    }

    /// Check if result has any blocking issues
    pub fn has_blocking_issues(&self) -> bool {
        self.issues.iter().any(|issue| issue.is_blocking())
    }

    /// Convert to standard Result, losing issue information
    pub fn into_result(self) -> ParseResult<T> {
        self.result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_display() {
        let error = ParseError::ExpectedSectionHeader { line: 5 };
        let message = format!("{}", error);
        assert!(message.contains("line 5"));
        assert!(message.contains("Expected section header"));
    }

    #[test]
    fn parse_issue_creation() {
        let issue = ParseIssue::warning(IssueCategory::Style, "Negative font size".to_string(), 10);

        assert_eq!(issue.severity, IssueSeverity::Warning);
        assert_eq!(issue.category, IssueCategory::Style);
        assert_eq!(issue.line, 10);
        assert_eq!(issue.column, None);
    }

    #[test]
    fn parse_issue_with_location() {
        let issue = ParseIssue::with_location(
            IssueSeverity::Error,
            IssueCategory::Color,
            "Invalid color format".to_string(),
            15,
            25,
            (100, 110),
        );

        assert_eq!(issue.line, 15);
        assert_eq!(issue.column, Some(25));
        assert_eq!(issue.span, Some((100, 110)));
    }

    #[test]
    fn parse_issue_with_suggestion() {
        let issue = ParseIssue::error(
            IssueCategory::Format,
            "Missing colon in field".to_string(),
            8,
        )
        .with_suggestion("Add ':' after field name".to_string());

        assert!(issue.suggestion.is_some());
        assert_eq!(issue.suggestion.unwrap(), "Add ':' after field name");
    }

    #[test]
    fn parse_issue_formatting() {
        let issue = ParseIssue::with_location(
            IssueSeverity::Warning,
            IssueCategory::Performance,
            "Many overlapping events".to_string(),
            20,
            5,
            (200, 250),
        )
        .with_suggestion("Consider reducing event density".to_string());

        let formatted = issue.format_for_display();
        assert!(formatted.contains("20:5"));
        assert!(formatted.contains("performance"));
        assert!(formatted.contains("warning"));
        assert!(formatted.contains("Suggestion:"));
    }

    #[test]
    fn issue_severity_display() {
        assert_eq!(format!("{}", IssueSeverity::Info), "info");
        assert_eq!(format!("{}", IssueSeverity::Warning), "warning");
        assert_eq!(format!("{}", IssueSeverity::Error), "error");
        assert_eq!(format!("{}", IssueSeverity::Critical), "critical");
    }

    #[test]
    fn issue_category_display() {
        assert_eq!(format!("{}", IssueCategory::Style), "style");
        assert_eq!(format!("{}", IssueCategory::Timing), "timing");
        assert_eq!(format!("{}", IssueCategory::Performance), "performance");
    }

    #[test]
    fn parse_result_with_issues() {
        let mut result = ParseResultWithIssues::ok(42);
        result = result.add_issue(ParseIssue::warning(
            IssueCategory::Format,
            "Minor issue".to_string(),
            1,
        ));

        assert!(result.result.is_ok());
        assert_eq!(result.issues.len(), 1);
        assert!(!result.has_blocking_issues());
    }

    #[test]
    fn blocking_issues_detection() {
        let mut result = ParseResultWithIssues::ok(42);
        result = result.add_issue(ParseIssue::critical(
            IssueCategory::Structure,
            "Critical error".to_string(),
            1,
        ));

        assert!(result.has_blocking_issues());
        assert_eq!(result.critical_issues().len(), 1);
    }
}
