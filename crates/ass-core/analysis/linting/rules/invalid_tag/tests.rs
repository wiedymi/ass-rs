//! Tests for the invalid tag detection lint rule.

use super::*;
use crate::analysis::{
    linting::{IssueCategory, IssueSeverity, LintRule},
    ScriptAnalysis,
};

#[test]
fn rule_metadata_correct() {
    let rule = InvalidTagRule;
    assert_eq!(rule.id(), "invalid-tag");
    assert_eq!(rule.name(), "Invalid Tag");
    assert_eq!(
        rule.description(),
        "Detects invalid or malformed override tags in event text"
    );
    assert_eq!(rule.default_severity(), IssueSeverity::Warning);
    assert_eq!(rule.category(), IssueCategory::Content);
}

#[test]
fn empty_script_no_issues() {
    let script_text = "[Script Info]\nTitle: Test";
    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();

    let rule = InvalidTagRule;
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn valid_tags_no_issues() {
    let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text with {\i1}valid{\i0} tags";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let rule = InvalidTagRule;
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
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
    let rule = InvalidTagRule;
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn plain_text_no_issues() {
    let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Plain text without any tags";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let rule = InvalidTagRule;
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn empty_tag_after_valid_tag_detected() {
    let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text with {\b1\} override";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let rule = InvalidTagRule;
    let issues = rule.check_script(&analysis);

    assert!(!issues.is_empty());
    assert!(issues
        .iter()
        .any(|issue| issue.message().contains("Empty override tag")));
}
