//! Unit tests for parse result types and issue accumulation.

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
    let result_with_issues: ParseResultWithIssues<i32> = ParseResultWithIssues::from(parse_result);

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
