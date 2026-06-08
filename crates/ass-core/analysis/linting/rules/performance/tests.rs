//! Unit tests for the performance linting rule.

use super::*;
use crate::analysis::{
    linting::{IssueCategory, IssueSeverity, LintRule},
    ScriptAnalysis,
};
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::string::String;

#[test]
fn rule_metadata_correct() {
    let rule = PerformanceRule;
    assert_eq!(rule.id(), "performance");
    assert_eq!(rule.name(), "Performance");
    assert_eq!(
        rule.description(),
        "Detects potential performance issues in the script"
    );
    assert_eq!(rule.default_severity(), IssueSeverity::Hint);
    assert_eq!(rule.category(), IssueCategory::Performance);
}

#[test]
fn empty_script_no_issues() {
    let script_text = "[Script Info]\nTitle: Test";
    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();

    let rule = PerformanceRule;
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn small_script_no_issues() {
    let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Short text
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Another short text";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let rule = PerformanceRule;
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn no_events_section_no_issues() {
    let script_text = r"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF&,&H000000FF&,&H00000000&,&H00000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let rule = PerformanceRule;
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn long_text_event_detected() {
    let long_text = "a".repeat(600);
    let script_text = format!(
        r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{long_text}"
    );

    let script = crate::parser::Script::parse(&script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let rule = PerformanceRule;
    let issues = rule.check_script(&analysis);

    assert!(!issues.is_empty());
    assert!(issues
        .iter()
        .any(|issue| issue.message().contains("long text")));
}

#[test]
fn many_override_tags_detected() {
    let mut text_with_tags = String::new();
    for i in 0..25 {
        use core::fmt::Write;
        write!(text_with_tags, "{{\\i{}}}text", i % 2).unwrap();
    }

    let script_text = format!(
        r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{text_with_tags}"
    );

    let script = crate::parser::Script::parse(&script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let rule = PerformanceRule;
    let issues = rule.check_script(&analysis);

    assert!(!issues.is_empty());
    assert!(issues
        .iter()
        .any(|issue| issue.message().contains("override tags")));
}
