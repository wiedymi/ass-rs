//! Parse result types for error handling and issue collection
//!
//! Provides result types that can carry both successful parsing results
//! and accumulated issues/warnings. Enables partial recovery parsing
//! where some errors can be worked around while still collecting problems.

use alloc::vec::Vec;

use super::{
    parse_error::ParseError,
    parse_issue::{IssueSeverity, ParseIssue},
};

/// Result type for operations that can produce parse issues
///
/// Standard Result type using `ParseError` for the error case.
/// Use `ParseResultWithIssues` for operations that need to collect
/// warnings and recoverable errors alongside the main result.
pub type ParseResult<T> = Result<T, ParseError>;

/// Parse result with accumulated issues for partial recovery
///
/// Allows parsing to continue even when encountering recoverable errors,
/// collecting issues for later review while still producing a usable result.
/// Essential for editor integration and robust script processing.
#[derive(Debug, Clone)]
pub struct ParseResultWithIssues<T> {
    /// The parsed result (if successful)
    pub result: ParseResult<T>,

    /// Accumulated parse issues from warnings to recoverable errors
    pub issues: Vec<ParseIssue>,
}

impl<T> ParseResultWithIssues<T> {
    /// Create successful result with no issues
    ///
    /// # Arguments
    ///
    /// * `value` - The successfully parsed value
    ///
    /// # Returns
    ///
    /// Result containing the value with empty issues list
    pub const fn ok(value: T) -> Self {
        Self {
            result: Ok(value),
            issues: Vec::new(),
        }
    }

    /// Create error result with no issues
    ///
    /// # Arguments
    ///
    /// * `error` - The parse error that occurred
    ///
    /// # Returns
    ///
    /// Result containing the error with empty issues list
    #[must_use]
    pub const fn err(error: ParseError) -> Self {
        Self {
            result: Err(error),
            issues: Vec::new(),
        }
    }

    /// Create result with pre-collected issues
    ///
    /// # Arguments
    ///
    /// * `result` - The parse result (success or failure)
    /// * `issues` - List of issues encountered during parsing
    ///
    /// # Returns
    ///
    /// Result with the provided issues attached
    pub const fn with_issues(result: ParseResult<T>, issues: Vec<ParseIssue>) -> Self {
        Self { result, issues }
    }

    /// Add issue to existing result
    ///
    /// # Arguments
    ///
    /// * `issue` - Parse issue to add to the collection
    ///
    /// # Returns
    ///
    /// Self with the issue added to the issues list
    #[must_use]
    pub fn add_issue(mut self, issue: ParseIssue) -> Self {
        self.issues.push(issue);
        self
    }

    /// Get only critical issues from the collection
    ///
    /// Critical issues indicate serious problems that will likely
    /// affect rendering or script functionality.
    ///
    /// # Returns
    ///
    /// Vector of references to critical issues only
    pub fn critical_issues(&self) -> Vec<&ParseIssue> {
        self.issues
            .iter()
            .filter(|issue| matches!(issue.severity, IssueSeverity::Critical))
            .collect()
    }

    /// Check if result has any blocking issues
    ///
    /// Blocking issues are those that should prevent further processing
    /// or indicate fundamental problems with the script.
    ///
    /// # Returns
    ///
    /// True if any issue is marked as blocking
    pub fn has_blocking_issues(&self) -> bool {
        self.issues.iter().any(ParseIssue::is_blocking)
    }

    /// Get issue count by severity level
    ///
    /// # Arguments
    ///
    /// * `severity` - The severity level to count
    ///
    /// # Returns
    ///
    /// Number of issues with the specified severity
    pub fn count_by_severity(&self, severity: IssueSeverity) -> usize {
        self.issues
            .iter()
            .filter(|issue| issue.severity == severity)
            .count()
    }

    /// Check if parsing was successful (ignoring issues)
    ///
    /// # Returns
    ///
    /// True if the main result is Ok, regardless of issues
    pub const fn is_ok(&self) -> bool {
        self.result.is_ok()
    }

    /// Check if parsing failed with an error
    ///
    /// # Returns
    ///
    /// True if the main result is Err
    pub const fn is_err(&self) -> bool {
        self.result.is_err()
    }
}

impl<T> From<ParseResult<T>> for ParseResultWithIssues<T> {
    /// Convert a simple `ParseResult` into a `ParseResultWithIssues`
    ///
    /// Creates a result with no accumulated issues.
    fn from(result: ParseResult<T>) -> Self {
        Self {
            result,
            issues: Vec::new(),
        }
    }
}

impl<T> From<T> for ParseResultWithIssues<T> {
    /// Convert a value into a successful `ParseResultWithIssues`
    ///
    /// Creates an Ok result with no accumulated issues.
    fn from(value: T) -> Self {
        Self::ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::errors::{IssueCategory, IssueSeverity, ParseError, ParseIssue};
    #[cfg(not(feature = "std"))]
    use alloc::{
        string::{String, ToString},
        vec,
    };

    #[test]
    fn parse_result_with_issues_ok() {
        let result = ParseResultWithIssues::ok(42);
        assert!(result.is_ok());
        assert!(!result.is_err());
        assert_eq!(result.issues.len(), 0);
        assert!(!result.has_blocking_issues());
    }

    #[test]
    fn parse_result_with_issues_err() {
        let error = ParseError::ExpectedSectionHeader { line: 5 };
        let result = ParseResultWithIssues::<i32>::err(error);
        assert!(result.is_err());
        assert!(!result.is_ok());
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn parse_result_with_issues_add_issue() {
        let mut result = ParseResultWithIssues::ok(42);
        let issue = ParseIssue::warning(IssueCategory::Format, "Minor issue".to_string(), 1);
        result = result.add_issue(issue);

        assert!(result.is_ok());
        assert_eq!(result.issues.len(), 1);
        assert!(!result.has_blocking_issues());
        assert_eq!(result.count_by_severity(IssueSeverity::Warning), 1);
        assert_eq!(result.count_by_severity(IssueSeverity::Critical), 0);
    }

    #[test]
    fn parse_result_with_issues_blocking() {
        let mut result = ParseResultWithIssues::ok(42);
        let critical_issue =
            ParseIssue::critical(IssueCategory::Structure, "Critical error".to_string(), 1);
        result = result.add_issue(critical_issue);

        assert!(result.has_blocking_issues());
        assert_eq!(result.critical_issues().len(), 1);
        assert_eq!(result.count_by_severity(IssueSeverity::Critical), 1);
    }

    #[test]
    fn parse_result_with_issues_multiple_severities() {
        let mut result = ParseResultWithIssues::ok("test");

        result = result.add_issue(ParseIssue::info(
            IssueCategory::Performance,
            "Info message".to_string(),
            1,
        ));

        result = result.add_issue(ParseIssue::warning(
            IssueCategory::Style,
            "Warning message".to_string(),
            2,
        ));

        result = result.add_issue(ParseIssue::error(
            IssueCategory::Color,
            "Error message".to_string(),
            3,
        ));

        assert_eq!(result.issues.len(), 3);
        assert_eq!(result.count_by_severity(IssueSeverity::Info), 1);
        assert_eq!(result.count_by_severity(IssueSeverity::Warning), 1);
        assert_eq!(result.count_by_severity(IssueSeverity::Error), 1);
        assert_eq!(result.count_by_severity(IssueSeverity::Critical), 0);
        assert!(!result.has_blocking_issues());
    }

    #[test]
    fn parse_result_with_issues_from_parse_result() {
        let parse_result: ParseResult<i32> = Ok(100);
        let result_with_issues: ParseResultWithIssues<i32> =
            ParseResultWithIssues::from(parse_result);

        assert!(result_with_issues.is_ok());
        assert_eq!(result_with_issues.issues.len(), 0);
    }

    #[test]
    fn parse_result_with_issues_from_value() {
        let result_with_issues = ParseResultWithIssues::from("hello");

        assert!(result_with_issues.is_ok());
        assert_eq!(result_with_issues.issues.len(), 0);
        if let Ok(value) = result_with_issues.result {
            assert_eq!(value, "hello");
        }
    }

    #[test]
    fn parse_result_with_issues_from_error() {
        let error = ParseError::InvalidFieldFormat { line: 10 };
        let parse_result: ParseResult<String> = Err(error);
        let result_with_issues: ParseResultWithIssues<String> =
            ParseResultWithIssues::from(parse_result);

        assert!(result_with_issues.is_err());
        assert_eq!(result_with_issues.issues.len(), 0);
    }

    #[test]
    fn parse_result_with_issues_pre_collected() {
        let issues = vec![
            ParseIssue::warning(IssueCategory::Timing, "Late timing".to_string(), 5),
            ParseIssue::error(IssueCategory::Font, "Missing font".to_string(), 10),
        ];

        let result = ParseResultWithIssues::with_issues(Ok("success"), issues);

        assert!(result.is_ok());
        assert_eq!(result.issues.len(), 2);
        assert_eq!(result.count_by_severity(IssueSeverity::Warning), 1);
        assert_eq!(result.count_by_severity(IssueSeverity::Error), 1);
    }
}
