//! Unit tests for severity, category, location, and issue types.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn issue_severity_display() {
    assert_eq!(IssueSeverity::Info.to_string(), "info");
    assert_eq!(IssueSeverity::Hint.to_string(), "hint");
    assert_eq!(IssueSeverity::Warning.to_string(), "warning");
    assert_eq!(IssueSeverity::Error.to_string(), "error");
    assert_eq!(IssueSeverity::Critical.to_string(), "critical");
}

#[test]
fn issue_severity_ordering() {
    assert!(IssueSeverity::Info < IssueSeverity::Hint);
    assert!(IssueSeverity::Hint < IssueSeverity::Warning);
    assert!(IssueSeverity::Warning < IssueSeverity::Error);
    assert!(IssueSeverity::Error < IssueSeverity::Critical);
}

#[test]
fn issue_category_display() {
    assert_eq!(IssueCategory::Timing.to_string(), "timing");
    assert_eq!(IssueCategory::Styling.to_string(), "styling");
    assert_eq!(IssueCategory::Content.to_string(), "content");
    assert_eq!(IssueCategory::Performance.to_string(), "performance");
    assert_eq!(IssueCategory::Compliance.to_string(), "compliance");
    assert_eq!(IssueCategory::Accessibility.to_string(), "accessibility");
    assert_eq!(IssueCategory::Encoding.to_string(), "encoding");
}

#[test]
fn issue_location_creation() {
    let location = IssueLocation {
        line: 42,
        column: 10,
        offset: 1000,
        length: 5,
        span: "error".to_string(),
    };

    assert_eq!(location.line, 42);
    assert_eq!(location.column, 10);
    assert_eq!(location.offset, 1000);
    assert_eq!(location.length, 5);
    assert_eq!(location.span, "error");
}

#[test]
fn lint_issue_creation() {
    let issue = LintIssue::new(
        IssueSeverity::Warning,
        IssueCategory::Timing,
        "test_rule",
        "Test message".to_string(),
    );

    assert_eq!(issue.severity(), IssueSeverity::Warning);
    assert_eq!(issue.category(), IssueCategory::Timing);
    assert_eq!(issue.message(), "Test message");
    assert_eq!(issue.rule_id(), "test_rule");
    assert!(issue.description().is_none());
    assert!(issue.location().is_none());
    assert!(issue.suggested_fix().is_none());
}

#[test]
fn lint_issue_with_description() {
    let issue = LintIssue::new(
        IssueSeverity::Error,
        IssueCategory::Styling,
        "style_rule",
        "Style error".to_string(),
    )
    .with_description("Detailed description".to_string());

    assert_eq!(issue.description(), Some("Detailed description"));
}

#[test]
fn lint_issue_with_location() {
    let location = IssueLocation {
        line: 5,
        column: 2,
        offset: 100,
        length: 3,
        span: "bad".to_string(),
    };

    let issue = LintIssue::new(
        IssueSeverity::Critical,
        IssueCategory::Content,
        "content_rule",
        "Content error".to_string(),
    )
    .with_location(location);

    let loc = issue.location().unwrap();
    assert_eq!(loc.line, 5);
    assert_eq!(loc.column, 2);
    assert_eq!(loc.span, "bad");
}

#[test]
fn lint_issue_with_suggested_fix() {
    let issue = LintIssue::new(
        IssueSeverity::Hint,
        IssueCategory::Performance,
        "perf_rule",
        "Performance hint".to_string(),
    )
    .with_suggested_fix("Use simpler approach".to_string());

    assert_eq!(issue.suggested_fix(), Some("Use simpler approach"));
}
