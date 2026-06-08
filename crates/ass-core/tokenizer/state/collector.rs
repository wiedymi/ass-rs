//! Accumulation of tokenizer issues during lexical analysis.
//!
//! Defines [`IssueCollector`], a convenience container for gathering
//! [`TokenIssue`] values produced while tokenizing an ASS script.

use super::issue::TokenIssue;
use alloc::{string::String, vec::Vec};

/// Issue collector for accumulating tokenization problems
///
/// Provides convenient methods for collecting and managing tokenization
/// issues during lexical analysis.
#[derive(Debug, Clone, Default)]
pub struct IssueCollector<'a> {
    /// Collection of tokenization issues found during parsing
    issues: Vec<TokenIssue<'a>>,
}

impl<'a> IssueCollector<'a> {
    /// Create new empty issue collector
    #[must_use]
    pub const fn new() -> Self {
        Self { issues: Vec::new() }
    }

    /// Add issue to collection
    pub fn add_issue(&mut self, issue: TokenIssue<'a>) {
        self.issues.push(issue);
    }

    /// Add warning issue
    pub fn add_warning(&mut self, message: String, span: &'a str, line: usize, column: usize) {
        self.add_issue(TokenIssue::warning(message, span, line, column));
    }

    /// Add error issue
    pub fn add_error(&mut self, message: String, span: &'a str, line: usize, column: usize) {
        self.add_issue(TokenIssue::error(message, span, line, column));
    }

    /// Add critical issue
    pub fn add_critical(&mut self, message: String, span: &'a str, line: usize, column: usize) {
        self.add_issue(TokenIssue::critical(message, span, line, column));
    }

    /// Get all collected issues
    #[must_use]
    pub fn issues(&self) -> &[TokenIssue<'a>] {
        &self.issues
    }

    /// Check if any issues were collected
    #[must_use]
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    /// Check if any error-level issues were collected
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(TokenIssue::is_error)
    }

    /// Get count of issues
    #[must_use]
    pub fn issue_count(&self) -> usize {
        self.issues.len()
    }

    /// Clear all issues
    pub fn clear(&mut self) {
        self.issues.clear();
    }

    /// Take all issues, leaving collector empty
    pub fn take_issues(&mut self) -> Vec<TokenIssue<'a>> {
        core::mem::take(&mut self.issues)
    }
}
