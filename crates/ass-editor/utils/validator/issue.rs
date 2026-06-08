//! Validation severity levels and individual issue representation.
//!
//! Defines `ValidationSeverity` and `ValidationIssue`, the building blocks
//! used to describe problems found while validating editor documents.

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ValidationSeverity {
    /// Informational message
    #[default]
    Info,
    /// Warning that doesn't prevent script execution
    Warning,
    /// Error that may cause rendering issues
    Error,
    /// Critical error that prevents script execution
    Critical,
}

/// A validation issue found in the document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    /// Severity of the issue
    pub severity: ValidationSeverity,

    /// Line number where the issue occurs (1-indexed)
    pub line: Option<usize>,

    /// Column number where the issue occurs (1-indexed)
    pub column: Option<usize>,

    /// Human-readable description of the issue
    pub message: String,

    /// Rule or check that generated this issue
    pub rule: String,

    /// Suggested fix for the issue (if available)
    pub suggestion: Option<String>,
}

impl ValidationIssue {
    /// Create a new validation issue
    ///
    /// # Examples
    ///
    /// ```
    /// use ass_editor::utils::validator::{ValidationIssue, ValidationSeverity};
    ///
    /// let issue = ValidationIssue::new(
    ///     ValidationSeverity::Warning,
    ///     "Missing subtitle end time".to_string(),
    ///     "timing_check".to_string()
    /// )
    /// .at_location(10, 25)
    /// .with_suggestion("Add explicit end time".to_string());
    ///
    /// assert_eq!(issue.line, Some(10));
    /// assert_eq!(issue.column, Some(25));
    /// assert!(!issue.is_error());
    /// ```
    pub fn new(severity: ValidationSeverity, message: String, rule: String) -> Self {
        Self {
            severity,
            line: None,
            column: None,
            message,
            rule,
            suggestion: None,
        }
    }

    /// Set the location of this issue
    #[must_use]
    pub fn at_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Add a suggestion for fixing this issue
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    /// Check if this is an error or critical issue
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(
            self.severity,
            ValidationSeverity::Error | ValidationSeverity::Critical
        )
    }

    /// Check if this is a warning or higher
    #[must_use]
    pub const fn is_warning_or_higher(&self) -> bool {
        matches!(
            self.severity,
            ValidationSeverity::Warning | ValidationSeverity::Error | ValidationSeverity::Critical
        )
    }
}
