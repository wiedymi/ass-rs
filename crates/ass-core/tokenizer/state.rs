//! Tokenizer state management and issue reporting
//!
//! Provides context tracking and error reporting for the ASS tokenizer.
//! Maintains parsing state and accumulates issues during lexical analysis.

use alloc::{string::String, vec::Vec};

/// Tokenization context for state-aware parsing
///
/// Tracks current parsing context to enable context-sensitive tokenization
/// of ASS script elements that have different lexical rules in different
/// contexts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenContext {
    /// Top-level document parsing
    ///
    /// Default state for processing section headers, comments, and
    /// top-level document structure.
    Document,

    /// Inside section header like [Events]
    ///
    /// Special tokenization rules for section names within square brackets.
    SectionHeader,

    /// Inside field definition line
    ///
    /// Field values have different whitespace and delimiter handling than
    /// other contexts.
    FieldValue,

    /// Inside style override block like {\b1}
    ///
    /// Override tags use backslash prefixes and have special syntax rules.
    StyleOverride,

    /// Inside drawing commands (\p1)
    ///
    /// Drawing commands use vector graphics syntax with different
    /// coordinate and command parsing rules.
    DrawingCommands,

    /// Inside UU-encoded data (fonts/graphics)
    ///
    /// Binary data sections use different character validation and
    /// line parsing rules.
    UuEncodedData,
}

impl TokenContext {
    /// Check if context allows whitespace skipping
    #[must_use] pub const fn allows_whitespace_skipping(self) -> bool {
        !matches!(self, Self::FieldValue | Self::UuEncodedData)
    }

    /// Check if context is inside a delimited block
    #[must_use] pub const fn is_delimited_block(self) -> bool {
        matches!(self, Self::SectionHeader | Self::StyleOverride)
    }

    /// Get expected closing delimiter for context
    #[must_use] pub const fn closing_delimiter(self) -> Option<char> {
        match self {
            Self::SectionHeader => Some(']'),
            Self::StyleOverride => Some('}'),
            _ => None,
        }
    }

    /// Transition to field value context after colon
    #[must_use]
    pub const fn enter_field_value(self) -> Self {
        match self {
            Self::Document => Self::FieldValue,
            other => other,
        }
    }

    /// Reset to document context (typically after newline)
    #[must_use]
    pub const fn reset_to_document(self) -> Self {
        Self::Document
    }
}

impl Default for TokenContext {
    fn default() -> Self {
        Self::Document
    }
}

/// Token issue severity levels
///
/// Categorizes tokenization issues by severity to enable appropriate
/// error handling and recovery strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueLevel {
    /// Warning that doesn't prevent tokenization
    ///
    /// Indicates potential problems that don't break parsing but may
    /// indicate authoring errors or compatibility issues.
    Warning,

    /// Error that may affect parsing
    ///
    /// Indicates problems that could cause incorrect parsing but allow
    /// tokenization to continue with error recovery.
    Error,

    /// Critical error requiring recovery
    ///
    /// Indicates severe problems that require special handling to
    /// continue tokenization safely.
    Critical,
}

impl IssueLevel {
    /// Check if issue level indicates an error condition
    #[must_use] pub const fn is_error(self) -> bool {
        matches!(self, Self::Error | Self::Critical)
    }

    /// Check if issue level should stop tokenization
    #[must_use] pub const fn should_abort(self) -> bool {
        matches!(self, Self::Critical)
    }

    /// Get string representation for display
    #[must_use] pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }
}

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
    #[must_use] pub const fn new(
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
    #[must_use] pub const fn warning(message: String, span: &'a str, line: usize, column: usize) -> Self {
        Self::new(IssueLevel::Warning, message, span, line, column)
    }

    /// Create error issue
    #[must_use] pub const fn error(message: String, span: &'a str, line: usize, column: usize) -> Self {
        Self::new(IssueLevel::Error, message, span, line, column)
    }

    /// Create critical issue
    #[must_use] pub const fn critical(message: String, span: &'a str, line: usize, column: usize) -> Self {
        Self::new(IssueLevel::Critical, message, span, line, column)
    }

    /// Check if this is an error-level issue
    #[must_use] pub const fn is_error(&self) -> bool {
        self.level.is_error()
    }

    /// Get formatted location string
    #[must_use] pub fn location_string(&self) -> String {
        format!("{}:{}", self.line, self.column)
    }

    /// Get formatted issue string for display
    #[must_use] pub fn format_issue(&self) -> String {
        format!(
            "{}: {} at {}:{}",
            self.level.as_str(),
            self.message,
            self.line,
            self.column
        )
    }
}

/// Issue collector for accumulating tokenization problems
///
/// Provides convenient methods for collecting and managing tokenization
/// issues during lexical analysis.
#[derive(Debug, Clone, Default)]
pub struct IssueCollector<'a> {
    issues: Vec<TokenIssue<'a>>,
}

impl<'a> IssueCollector<'a> {
    /// Create new empty issue collector
    #[must_use] pub const fn new() -> Self {
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
    #[must_use] pub fn issues(&self) -> &[TokenIssue<'a>] {
        &self.issues
    }

    /// Check if any issues were collected
    #[must_use] pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    /// Check if any error-level issues were collected
    pub fn has_errors(&self) -> bool {
        self.issues.iter().any(TokenIssue::is_error)
    }

    /// Get count of issues
    #[must_use] pub fn issue_count(&self) -> usize {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_context_transitions() {
        let mut context = TokenContext::Document;
        assert!(context.allows_whitespace_skipping());
        assert!(!context.is_delimited_block());

        context = context.enter_field_value();
        assert_eq!(context, TokenContext::FieldValue);
        assert!(!context.allows_whitespace_skipping());

        context = context.reset_to_document();
        assert_eq!(context, TokenContext::Document);
    }

    #[test]
    fn token_context_delimiters() {
        assert_eq!(TokenContext::SectionHeader.closing_delimiter(), Some(']'));
        assert_eq!(TokenContext::StyleOverride.closing_delimiter(), Some('}'));
        assert_eq!(TokenContext::Document.closing_delimiter(), None);
    }

    #[test]
    fn issue_level_properties() {
        assert!(!IssueLevel::Warning.is_error());
        assert!(IssueLevel::Error.is_error());
        assert!(IssueLevel::Critical.is_error());

        assert!(!IssueLevel::Warning.should_abort());
        assert!(!IssueLevel::Error.should_abort());
        assert!(IssueLevel::Critical.should_abort());
    }

    #[test]
    fn token_issue_creation() {
        let span = "test span";
        let issue = TokenIssue::warning("Test warning".to_string(), span, 5, 10);

        assert_eq!(issue.level, IssueLevel::Warning);
        assert_eq!(issue.message, "Test warning");
        assert_eq!(issue.span, span);
        assert_eq!(issue.line, 5);
        assert_eq!(issue.column, 10);
        assert!(!issue.is_error());
    }

    #[test]
    fn issue_collector_operations() {
        let mut collector = IssueCollector::new();
        assert!(!collector.has_issues());
        assert!(!collector.has_errors());

        collector.add_warning("Warning".to_string(), "span", 1, 1);
        assert!(collector.has_issues());
        assert!(!collector.has_errors());

        collector.add_error("Error".to_string(), "span", 2, 2);
        assert!(collector.has_errors());
        assert_eq!(collector.issue_count(), 2);

        let issues = collector.take_issues();
        assert_eq!(issues.len(), 2);
        assert!(!collector.has_issues());
    }
}
