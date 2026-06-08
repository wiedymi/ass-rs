//! `ParseIssue` type for recoverable parsing problems
//!
//! Defines [`ParseIssue`], a recoverable problem record that carries severity,
//! category, location, and optional suggestion information for editor and
//! console reporting.

use super::{IssueCategory, IssueSeverity};
use alloc::{format, string::String};

/// Parse issue for recoverable problems and warnings
///
/// Used for problems that don't prevent parsing but may affect
/// rendering quality or indicate potential script issues.
/// Includes location information for editor integration.
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl ParseIssue {
    /// Create new parse issue with minimal information
    #[must_use]
    pub const fn new(
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
    #[must_use]
    pub const fn with_location(
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
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    /// Create info-level issue
    #[must_use]
    pub const fn info(category: IssueCategory, message: String, line: usize) -> Self {
        Self::new(IssueSeverity::Info, category, message, line)
    }

    /// Create warning-level issue
    #[must_use]
    pub const fn warning(category: IssueCategory, message: String, line: usize) -> Self {
        Self::new(IssueSeverity::Warning, category, message, line)
    }

    /// Create error-level issue
    #[must_use]
    pub const fn error(category: IssueCategory, message: String, line: usize) -> Self {
        Self::new(IssueSeverity::Error, category, message, line)
    }

    /// Create critical-level issue
    #[must_use]
    pub const fn critical(category: IssueCategory, message: String, line: usize) -> Self {
        Self::new(IssueSeverity::Critical, category, message, line)
    }

    /// Format issue for display in editor or console
    #[must_use]
    pub fn format_for_display(&self) -> String {
        let location = self.column.map_or_else(
            || format!("{}", self.line),
            |column| format!("{}:{}", self.line, column),
        );

        let mut result = format!(
            "[{}:{}] {}: {}",
            location, self.category, self.severity, self.message
        );

        if let Some(suggestion) = &self.suggestion {
            result.push_str("\n  Suggestion: ");
            result.push_str(suggestion);
        }

        result
    }

    /// Check if this is a blocking error that should prevent further processing
    #[must_use]
    pub const fn is_blocking(&self) -> bool {
        matches!(self.severity, IssueSeverity::Critical)
    }
}
