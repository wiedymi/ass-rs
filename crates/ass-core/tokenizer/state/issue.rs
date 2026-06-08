//! Tokenization issue records for error reporting.
//!
//! Defines [`TokenIssue`], a single diagnostic carrying location information
//! and a severity [`IssueLevel`] for appropriate handling during lexical
//! analysis.

use super::level::IssueLevel;
use alloc::{format, string::String};

/// Tokenization issue for error reporting
///
/// Represents a problem encountered during tokenization with location
/// information and severity level for appropriate handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenIssue<'a> {
    /// Issue severity level
    pub level: IssueLevel,

    /// Human-readable error message
    pub message: String,

    /// Source span where issue occurred
    pub span: &'a str,

    /// Line number where issue occurred (1-based)
    pub line: usize,

    /// Column number where issue occurred (1-based)
    pub column: usize,
}

impl<'a> TokenIssue<'a> {
    /// Create new tokenization issue
    ///
    /// # Arguments
    ///
    /// * `level` - Severity level of the issue
    /// * `message` - Human-readable description
    /// * `span` - Source text span where issue occurred
    /// * `line` - Line number (1-based)
    /// * `column` - Column number (1-based)
    #[must_use]
    pub const fn new(
        level: IssueLevel,
        message: String,
        span: &'a str,
        line: usize,
        column: usize,
    ) -> Self {
        Self {
            level,
            message,
            span,
            line,
            column,
        }
    }

    /// Create warning issue
    #[must_use]
    pub const fn warning(message: String, span: &'a str, line: usize, column: usize) -> Self {
        Self::new(IssueLevel::Warning, message, span, line, column)
    }

    /// Create error issue
    #[must_use]
    pub const fn error(message: String, span: &'a str, line: usize, column: usize) -> Self {
        Self::new(IssueLevel::Error, message, span, line, column)
    }

    /// Create critical issue
    #[must_use]
    pub const fn critical(message: String, span: &'a str, line: usize, column: usize) -> Self {
        Self::new(IssueLevel::Critical, message, span, line, column)
    }

    /// Check if this is an error-level issue
    #[must_use]
    pub const fn is_error(&self) -> bool {
        self.level.is_error()
    }

    /// Get formatted location string
    #[must_use]
    pub fn location_string(&self) -> String {
        format!("{}:{}", self.line, self.column)
    }

    /// Get formatted issue string for display
    #[must_use]
    pub fn format_issue(&self) -> String {
        format!(
            "{}: {} at {}:{}",
            self.level.as_str(),
            self.message,
            self.line,
            self.column
        )
    }
}
