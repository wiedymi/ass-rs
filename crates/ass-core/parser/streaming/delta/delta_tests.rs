//! Behavioural tests for the [`ParseDelta`] enum.

use super::*;
use crate::parser::ast::{ScriptInfo, Section, Span};
#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString, vec};

#[test]
fn delta_creation() {
    let section = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });
    let delta = ParseDelta::add_section(section);
    assert!(matches!(delta, ParseDelta::AddSection(_)));
    assert!(!delta.is_error());
    assert!(delta.is_structural());
}

#[test]
fn delta_properties() {
    let remove_delta = ParseDelta::remove_section(5);
    assert!(remove_delta.is_structural());
    assert!(!remove_delta.is_error());
    assert_eq!(remove_delta.section(), None);

    let error_delta = ParseDelta::parse_issue("Test error".to_string());
    assert!(!error_delta.is_structural());
    assert!(error_delta.is_error());
}

#[test]
fn delta_update_section() {
    let section = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });
    let delta = ParseDelta::update_section(3, section);

    assert!(matches!(delta, ParseDelta::UpdateSection(3, _)));
    assert!(!delta.is_error());
    assert!(delta.is_structural());
    assert!(delta.section().is_some());
}

#[test]
fn delta_section_getter() {
    let section = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });

    let add_delta = ParseDelta::add_section(section.clone());
    assert!(add_delta.section().is_some());

    let update_delta = ParseDelta::update_section(0, section);
    assert!(update_delta.section().is_some());

    let remove_delta = ParseDelta::remove_section(1);
    assert!(remove_delta.section().is_none());

    let error_delta = ParseDelta::parse_issue("Test".to_string());
    assert!(error_delta.section().is_none());
}

#[test]
fn delta_debug_formatting() {
    let section = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });
    let delta = ParseDelta::add_section(section);
    let debug_str = format!("{delta:?}");
    assert!(debug_str.contains("AddSection"));

    let error_delta = ParseDelta::parse_issue("Error message".to_string());
    let error_debug = format!("{error_delta:?}");
    assert!(error_debug.contains("ParseIssue"));
    assert!(error_debug.contains("Error message"));
}

#[test]
fn delta_clone() {
    let section = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });
    let delta = ParseDelta::add_section(section);
    let cloned = delta.clone();

    assert!(matches!(cloned, ParseDelta::AddSection(_)));
    assert_eq!(delta.is_error(), cloned.is_error());
    assert_eq!(delta.is_structural(), cloned.is_structural());
}

#[test]
fn delta_all_constructors() {
    let section = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });

    let add = ParseDelta::add_section(section.clone());
    assert!(matches!(add, ParseDelta::AddSection(_)));

    let update = ParseDelta::update_section(42, section);
    assert!(matches!(update, ParseDelta::UpdateSection(42, _)));

    let remove = ParseDelta::remove_section(99);
    assert!(matches!(remove, ParseDelta::RemoveSection(99)));

    let issue = ParseDelta::parse_issue("Test issue".to_string());
    assert!(matches!(issue, ParseDelta::ParseIssue(_)));
}

#[test]
fn delta_all_variants_coverage() {
    // Test all ParseDelta variants
    let section = Section::ScriptInfo(ScriptInfo {
        fields: vec![],
        span: Span::new(0, 0, 0, 0),
    });

    // AddSection
    let add = ParseDelta::AddSection(section.clone());
    assert!(add.is_structural());
    assert!(!add.is_error());
    assert!(add.section().is_some());

    // UpdateSection
    let update = ParseDelta::UpdateSection(5, section);
    assert!(update.is_structural());
    assert!(!update.is_error());
    assert!(update.section().is_some());

    // RemoveSection
    let remove = ParseDelta::RemoveSection(10);
    assert!(remove.is_structural());
    assert!(!remove.is_error());
    assert!(remove.section().is_none());

    // ParseIssue
    let issue = ParseDelta::ParseIssue("Critical error".to_string());
    assert!(!issue.is_structural());
    assert!(issue.is_error());
    assert!(issue.section().is_none());
}
