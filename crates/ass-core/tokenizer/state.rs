//! Tokenizer state management and issue reporting
//!
//! Provides context tracking and error reporting for the ASS tokenizer.
//! Maintains parsing state and accumulates issues during lexical analysis.

use alloc::{format, string::String, vec::Vec};

#[cfg(not(feature = "std"))]
extern crate alloc;
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

    /// Inside section header like `[Events]`
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
    #[must_use]
    pub const fn allows_whitespace_skipping(self) -> bool {
        !matches!(self, Self::FieldValue | Self::UuEncodedData)
    }

    /// Check if context is inside a delimited block
    #[must_use]
    pub const fn is_delimited_block(self) -> bool {
        matches!(self, Self::SectionHeader | Self::StyleOverride)
    }

    /// Get expected closing delimiter for context
    #[must_use]
    pub const fn closing_delimiter(self) -> Option<char> {
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
    #[must_use]
    pub const fn is_error(self) -> bool {
        matches!(self, Self::Error | Self::Critical)
    }

    /// Check if issue level should stop tokenization
    #[must_use]
    pub const fn should_abort(self) -> bool {
        matches!(self, Self::Critical)
    }

    /// Get string representation for display
    #[must_use]
    pub const fn as_str(self) -> &'static str {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

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

    #[test]
    fn token_context_all_variants() {
        // Test all TokenContext variants for is_delimited_block
        assert!(!TokenContext::Document.is_delimited_block());
        assert!(TokenContext::SectionHeader.is_delimited_block());
        assert!(!TokenContext::FieldValue.is_delimited_block());
        assert!(TokenContext::StyleOverride.is_delimited_block());
        assert!(!TokenContext::DrawingCommands.is_delimited_block());
        assert!(!TokenContext::UuEncodedData.is_delimited_block());
    }

    #[test]
    fn token_context_whitespace_skipping_all_variants() {
        // Test all TokenContext variants for allows_whitespace_skipping
        assert!(TokenContext::Document.allows_whitespace_skipping());
        assert!(TokenContext::SectionHeader.allows_whitespace_skipping());
        assert!(!TokenContext::FieldValue.allows_whitespace_skipping());
        assert!(TokenContext::StyleOverride.allows_whitespace_skipping());
        assert!(TokenContext::DrawingCommands.allows_whitespace_skipping());
        assert!(!TokenContext::UuEncodedData.allows_whitespace_skipping());
    }

    #[test]
    fn token_context_closing_delimiters_all_variants() {
        // Test all TokenContext variants for closing_delimiter
        assert_eq!(TokenContext::Document.closing_delimiter(), None);
        assert_eq!(TokenContext::SectionHeader.closing_delimiter(), Some(']'));
        assert_eq!(TokenContext::FieldValue.closing_delimiter(), None);
        assert_eq!(TokenContext::StyleOverride.closing_delimiter(), Some('}'));
        assert_eq!(TokenContext::DrawingCommands.closing_delimiter(), None);
        assert_eq!(TokenContext::UuEncodedData.closing_delimiter(), None);
    }

    #[test]
    fn token_context_enter_field_value_all_variants() {
        // Test enter_field_value from all contexts
        assert_eq!(
            TokenContext::Document.enter_field_value(),
            TokenContext::FieldValue
        );
        assert_eq!(
            TokenContext::SectionHeader.enter_field_value(),
            TokenContext::SectionHeader
        );
        assert_eq!(
            TokenContext::FieldValue.enter_field_value(),
            TokenContext::FieldValue
        );
        assert_eq!(
            TokenContext::StyleOverride.enter_field_value(),
            TokenContext::StyleOverride
        );
        assert_eq!(
            TokenContext::DrawingCommands.enter_field_value(),
            TokenContext::DrawingCommands
        );
        assert_eq!(
            TokenContext::UuEncodedData.enter_field_value(),
            TokenContext::UuEncodedData
        );
    }

    #[test]
    fn token_context_reset_to_document_all_variants() {
        // Test reset_to_document from all contexts
        assert_eq!(
            TokenContext::Document.reset_to_document(),
            TokenContext::Document
        );
        assert_eq!(
            TokenContext::SectionHeader.reset_to_document(),
            TokenContext::Document
        );
        assert_eq!(
            TokenContext::FieldValue.reset_to_document(),
            TokenContext::Document
        );
        assert_eq!(
            TokenContext::StyleOverride.reset_to_document(),
            TokenContext::Document
        );
        assert_eq!(
            TokenContext::DrawingCommands.reset_to_document(),
            TokenContext::Document
        );
        assert_eq!(
            TokenContext::UuEncodedData.reset_to_document(),
            TokenContext::Document
        );
    }

    #[test]
    fn token_context_default() {
        assert_eq!(TokenContext::default(), TokenContext::Document);
    }

    #[test]
    fn issue_level_as_str() {
        assert_eq!(IssueLevel::Warning.as_str(), "warning");
        assert_eq!(IssueLevel::Error.as_str(), "error");
        assert_eq!(IssueLevel::Critical.as_str(), "critical");
    }

    #[test]
    fn token_issue_all_constructors() {
        let span = "test span";

        let warning = TokenIssue::warning("Warning message".to_string(), span, 10, 5);
        assert_eq!(warning.level, IssueLevel::Warning);
        assert_eq!(warning.message, "Warning message");
        assert!(!warning.is_error());

        let error = TokenIssue::error("Error message".to_string(), span, 15, 8);
        assert_eq!(error.level, IssueLevel::Error);
        assert_eq!(error.message, "Error message");
        assert!(error.is_error());

        let critical = TokenIssue::critical("Critical message".to_string(), span, 20, 12);
        assert_eq!(critical.level, IssueLevel::Critical);
        assert_eq!(critical.message, "Critical message");
        assert!(critical.is_error());
    }

    #[test]
    fn token_issue_location_string() {
        let issue = TokenIssue::new(IssueLevel::Warning, "Test".to_string(), "span", 42, 13);
        assert_eq!(issue.location_string(), "42:13");
    }

    #[test]
    fn token_issue_format_issue() {
        let issue = TokenIssue::error("Test error message".to_string(), "span", 5, 10);
        let formatted = issue.format_issue();
        assert!(formatted.contains("error"));
        assert!(formatted.contains("Test error message"));
        assert!(formatted.contains("5:10"));
    }

    #[test]
    fn issue_collector_new_vs_default() {
        let collector1 = IssueCollector::new();
        let collector2 = IssueCollector::default();

        assert_eq!(collector1.issue_count(), collector2.issue_count());
        assert_eq!(collector1.has_issues(), collector2.has_issues());
    }

    #[test]
    fn issue_collector_add_issue_directly() {
        let mut collector = IssueCollector::new();
        let issue = TokenIssue::warning("Direct issue".to_string(), "span", 1, 1);

        collector.add_issue(issue.clone());
        assert_eq!(collector.issue_count(), 1);
        assert_eq!(collector.issues()[0], issue);
    }

    #[test]
    fn issue_collector_add_critical() {
        let mut collector = IssueCollector::new();
        collector.add_critical("Critical issue".to_string(), "span", 3, 7);

        assert!(collector.has_issues());
        assert!(collector.has_errors());
        assert_eq!(collector.issues()[0].level, IssueLevel::Critical);
        assert!(collector.issues()[0].level.should_abort());
    }

    #[test]
    fn issue_collector_clear() {
        let mut collector = IssueCollector::new();
        collector.add_warning("Warning".to_string(), "span", 1, 1);
        collector.add_error("Error".to_string(), "span", 2, 2);

        assert!(collector.has_issues());
        assert_eq!(collector.issue_count(), 2);

        collector.clear();
        assert!(!collector.has_issues());
        assert_eq!(collector.issue_count(), 0);
    }

    #[test]
    fn issue_collector_mixed_issue_types() {
        let mut collector = IssueCollector::new();

        collector.add_warning("First warning".to_string(), "span1", 1, 1);
        collector.add_error("First error".to_string(), "span2", 2, 2);
        collector.add_critical("Critical issue".to_string(), "span3", 3, 3);
        collector.add_warning("Second warning".to_string(), "span4", 4, 4);

        assert_eq!(collector.issue_count(), 4);
        assert!(collector.has_issues());
        assert!(collector.has_errors());

        let issues = collector.issues();
        assert_eq!(issues[0].level, IssueLevel::Warning);
        assert_eq!(issues[1].level, IssueLevel::Error);
        assert_eq!(issues[2].level, IssueLevel::Critical);
        assert_eq!(issues[3].level, IssueLevel::Warning);
    }

    #[test]
    fn token_issue_equality() {
        let issue1 = TokenIssue::warning("Same message".to_string(), "same span", 5, 10);
        let issue2 = TokenIssue::warning("Same message".to_string(), "same span", 5, 10);
        let issue3 = TokenIssue::error("Same message".to_string(), "same span", 5, 10);

        assert_eq!(issue1, issue2);
        assert_ne!(issue1, issue3); // Different levels
    }

    #[test]
    fn issue_level_clone_and_copy() {
        let level1 = IssueLevel::Warning;
        let level2 = level1;
        let level3 = level1;

        assert_eq!(level1, level2);
        assert_eq!(level1, level3);
    }

    #[test]
    fn token_context_clone_and_copy() {
        let context1 = TokenContext::StyleOverride;
        let context2 = context1;
        let context3 = context1;

        assert_eq!(context1, context2);
        assert_eq!(context1, context3);
    }
}
