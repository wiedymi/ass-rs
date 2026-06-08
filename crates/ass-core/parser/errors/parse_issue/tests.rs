//! Unit tests for parse issue types.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};

#[test]
fn issue_severity_display() {
    assert_eq!(format!("{}", IssueSeverity::Info), "info");
    assert_eq!(format!("{}", IssueSeverity::Warning), "warning");
    assert_eq!(format!("{}", IssueSeverity::Error), "error");
    assert_eq!(format!("{}", IssueSeverity::Critical), "critical");
}

#[test]
fn issue_category_display() {
    assert_eq!(format!("{}", IssueCategory::Structure), "structure");
    assert_eq!(format!("{}", IssueCategory::Style), "style");
    assert_eq!(format!("{}", IssueCategory::Event), "event");
    assert_eq!(format!("{}", IssueCategory::Timing), "timing");
    assert_eq!(format!("{}", IssueCategory::Color), "color");
    assert_eq!(format!("{}", IssueCategory::Font), "font");
    assert_eq!(format!("{}", IssueCategory::Drawing), "drawing");
    assert_eq!(format!("{}", IssueCategory::Performance), "performance");
    assert_eq!(format!("{}", IssueCategory::Compatibility), "compatibility");
    assert_eq!(format!("{}", IssueCategory::Security), "security");
    assert_eq!(format!("{}", IssueCategory::Format), "format");
}

#[test]
fn parse_issue_creation() {
    let issue = ParseIssue::new(
        IssueSeverity::Warning,
        IssueCategory::Style,
        "Negative font size".to_string(),
        10,
    );

    assert_eq!(issue.severity, IssueSeverity::Warning);
    assert_eq!(issue.category, IssueCategory::Style);
    assert_eq!(issue.message, "Negative font size");
    assert_eq!(issue.line, 10);
    assert_eq!(issue.column, None);
    assert_eq!(issue.span, None);
    assert_eq!(issue.suggestion, None);
    assert!(!issue.is_blocking());
}

#[test]
fn parse_issue_with_location() {
    let issue = ParseIssue::with_location(
        IssueSeverity::Error,
        IssueCategory::Color,
        "Invalid color format".to_string(),
        15,
        25,
        (100, 110),
    );

    assert_eq!(issue.severity, IssueSeverity::Error);
    assert_eq!(issue.category, IssueCategory::Color);
    assert_eq!(issue.line, 15);
    assert_eq!(issue.column, Some(25));
    assert_eq!(issue.span, Some((100, 110)));
    assert!(!issue.is_blocking());
}

#[test]
fn parse_issue_with_suggestion() {
    let issue = ParseIssue::error(
        IssueCategory::Format,
        "Missing colon in field".to_string(),
        8,
    )
    .with_suggestion("Add ':' after field name".to_string());

    assert_eq!(issue.severity, IssueSeverity::Error);
    assert!(issue.suggestion.is_some());
    assert_eq!(issue.suggestion.unwrap(), "Add ':' after field name");
}

#[test]
fn parse_issue_convenience_constructors() {
    let info_issue = ParseIssue::info(IssueCategory::Performance, "Info message".to_string(), 1);
    assert_eq!(info_issue.severity, IssueSeverity::Info);
    assert!(!info_issue.is_blocking());

    let warning_issue = ParseIssue::warning(IssueCategory::Style, "Warning message".to_string(), 2);
    assert_eq!(warning_issue.severity, IssueSeverity::Warning);
    assert!(!warning_issue.is_blocking());

    let error_issue = ParseIssue::error(IssueCategory::Color, "Error message".to_string(), 3);
    assert_eq!(error_issue.severity, IssueSeverity::Error);
    assert!(!error_issue.is_blocking());

    let critical_issue =
        ParseIssue::critical(IssueCategory::Structure, "Critical message".to_string(), 4);
    assert_eq!(critical_issue.severity, IssueSeverity::Critical);
    assert!(critical_issue.is_blocking());
}

#[test]
fn parse_issue_formatting_simple() {
    let issue = ParseIssue::warning(
        IssueCategory::Performance,
        "Many overlapping events".to_string(),
        20,
    );

    let formatted = issue.format_for_display();
    assert!(formatted.contains("20"));
    assert!(formatted.contains("performance"));
    assert!(formatted.contains("warning"));
    assert!(formatted.contains("Many overlapping events"));
    assert!(!formatted.contains("Suggestion:"));
}

#[test]
fn parse_issue_formatting_with_location() {
    let issue = ParseIssue::with_location(
        IssueSeverity::Error,
        IssueCategory::Timing,
        "Overlapping dialogue".to_string(),
        30,
        15,
        (200, 250),
    );

    let formatted = issue.format_for_display();
    assert!(formatted.contains("30:15"));
    assert!(formatted.contains("timing"));
    assert!(formatted.contains("error"));
    assert!(formatted.contains("Overlapping dialogue"));
}

#[test]
fn parse_issue_formatting_with_suggestion() {
    let issue = ParseIssue::with_location(
        IssueSeverity::Warning,
        IssueCategory::Performance,
        "Many override tags".to_string(),
        40,
        5,
        (300, 350),
    )
    .with_suggestion("Consider using styles instead".to_string());

    let formatted = issue.format_for_display();
    assert!(formatted.contains("40:5"));
    assert!(formatted.contains("performance"));
    assert!(formatted.contains("warning"));
    assert!(formatted.contains("Many override tags"));
    assert!(formatted.contains("Suggestion:"));
    assert!(formatted.contains("Consider using styles instead"));
}

#[test]
fn parse_issue_blocking_detection() {
    let non_blocking_info = ParseIssue::info(IssueCategory::Format, "Info".to_string(), 1);
    let non_blocking_warning = ParseIssue::warning(IssueCategory::Style, "Warning".to_string(), 2);
    let non_blocking_error = ParseIssue::error(IssueCategory::Color, "Error".to_string(), 3);
    let blocking_critical =
        ParseIssue::critical(IssueCategory::Structure, "Critical".to_string(), 4);

    assert!(!non_blocking_info.is_blocking());
    assert!(!non_blocking_warning.is_blocking());
    assert!(!non_blocking_error.is_blocking());
    assert!(blocking_critical.is_blocking());
}

#[test]
fn parse_issue_clone_and_equality() {
    let issue1 = ParseIssue::warning(IssueCategory::Font, "Missing font".to_string(), 50);
    let issue2 = issue1.clone();
    assert_eq!(issue1, issue2);

    let issue3 = ParseIssue::warning(IssueCategory::Font, "Different message".to_string(), 50);
    assert_ne!(issue1, issue3);
}

#[test]
fn parse_issue_debug() {
    let issue = ParseIssue::error(IssueCategory::Drawing, "Invalid command".to_string(), 60);
    let debug_str = format!("{issue:?}");
    assert!(debug_str.contains("ParseIssue"));
    assert!(debug_str.contains("Error"));
    assert!(debug_str.contains("Drawing"));
}
