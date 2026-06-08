//! Lint issue and source-location types.
//!
//! Defines [`IssueLocation`], describing where a problem occurs in source,
//! and [`LintIssue`], the rich diagnostic produced by linting rules along
//! with its builder-style construction API.

use super::{IssueCategory, IssueSeverity};
use alloc::string::String;

/// Location information for a lint issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssueLocation {
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Byte offset in source
    pub offset: usize,
    /// Length of the problematic span
    pub length: usize,
    /// The problematic text span
    pub span: String,
}

/// A single lint issue found in the script.
#[derive(Debug, Clone)]
pub struct LintIssue {
    /// Severity level
    severity: IssueSeverity,
    /// Category of issue
    category: IssueCategory,
    /// Human-readable message
    message: String,
    /// Optional detailed description
    description: Option<String>,
    /// Location in source (if available)
    location: Option<IssueLocation>,
    /// Rule ID that generated this issue
    rule_id: &'static str,
    /// Suggested fix (if available)
    suggested_fix: Option<String>,
}

impl LintIssue {
    /// Create a new lint issue.
    #[must_use]
    pub const fn new(
        severity: IssueSeverity,
        category: IssueCategory,
        rule_id: &'static str,
        message: String,
    ) -> Self {
        Self {
            severity,
            category,
            message,
            description: None,
            location: None,
            rule_id,
            suggested_fix: None,
        }
    }

    /// Add detailed description.
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Add location information.
    #[must_use]
    pub fn with_location(mut self, location: IssueLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add suggested fix.
    #[must_use]
    pub fn with_suggested_fix(mut self, fix: String) -> Self {
        self.suggested_fix = Some(fix);
        self
    }

    /// Get severity level.
    #[must_use]
    pub const fn severity(&self) -> IssueSeverity {
        self.severity
    }

    /// Get issue category.
    #[must_use]
    pub const fn category(&self) -> IssueCategory {
        self.category
    }

    /// Get issue message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get detailed description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get location information.
    #[must_use]
    pub const fn location(&self) -> Option<&IssueLocation> {
        self.location.as_ref()
    }

    /// Get rule ID.
    #[must_use]
    pub const fn rule_id(&self) -> &'static str {
        self.rule_id
    }

    /// Get suggested fix.
    #[must_use]
    pub fn suggested_fix(&self) -> Option<&str> {
        self.suggested_fix.as_deref()
    }
}
