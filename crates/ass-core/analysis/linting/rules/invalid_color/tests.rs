//! Tests for the invalid color format detection rule.

use super::*;
use crate::analysis::{
    linting::{IssueCategory, IssueSeverity, LintRule},
    ScriptAnalysis,
};
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn rule_metadata_correct() {
    let rule = InvalidColorRule;
    assert_eq!(rule.id(), "invalid-color");
    assert_eq!(rule.name(), "Invalid Color");
    assert_eq!(
        rule.description(),
        "Detects invalid color format in styles and override tags"
    );
    assert_eq!(rule.default_severity(), IssueSeverity::Error);
    assert_eq!(rule.category(), IssueCategory::Styling);
}

#[test]
fn empty_script_no_issues() {
    let script_text = "[Script Info]\nTitle: Test";
    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();

    let rule = InvalidColorRule;
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn parse_color_tag_formats() {
    assert_eq!(
        InvalidColorRule::parse_color_tag("c&H00FF00&"),
        Some(("c".to_string(), "00FF00".to_string()))
    );

    assert_eq!(
        InvalidColorRule::parse_color_tag("1c&H00FF00&"),
        Some(("1c".to_string(), "00FF00".to_string()))
    );

    assert!(InvalidColorRule::parse_color_tag("invalid").is_none());
}
