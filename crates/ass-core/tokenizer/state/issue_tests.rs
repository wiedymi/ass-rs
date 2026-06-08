//! Unit tests for [`TokenIssue`] and [`IssueCollector`] state types.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

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
