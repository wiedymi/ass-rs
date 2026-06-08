//! `ParseResultWithIssues` type for partial-recovery parsing
//!
//! Carries a parse result alongside accumulated issues so parsing can
//! continue past recoverable errors while still reporting them. Provides
//! constructors, issue queries, and `From` conversions from plain results
//! and bare values.

use alloc::vec::Vec;

use crate::parser::errors::parse_error::ParseError;
use crate::parser::errors::parse_issue::{IssueSeverity, ParseIssue};

use super::ParseResult;

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
