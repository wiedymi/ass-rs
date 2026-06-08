//! Behavioural tests for the [`AccessibilityRule`] lint rule.

use super::*;
use crate::analysis::{
    linting::{IssueCategory, IssueSeverity, LintRule},
    ScriptAnalysis,
};
use alloc::format;

#[test]
fn rule_metadata_correct() {
    let rule = AccessibilityRule;
    assert_eq!(rule.id(), "accessibility");
    assert_eq!(rule.name(), "Accessibility");
    assert_eq!(rule.description(), "Detects potential accessibility issues");
    assert_eq!(rule.default_severity(), IssueSeverity::Hint);
    assert_eq!(rule.category(), IssueCategory::Accessibility);
}

#[test]
fn empty_script_no_issues() {
    let script_text = "[Script Info]\nTitle: Test";
    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();

    let rule = AccessibilityRule;
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn normal_duration_no_issues() {
    let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Normal duration text";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let rule = AccessibilityRule;
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn short_duration_detected() {
    let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:00.30,Default,,0,0,0,,Too fast!";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let rule = AccessibilityRule;
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let issues = rule.check_script(&analysis);

    assert!(!issues.is_empty());
    assert!(issues
        .iter()
        .any(|issue| issue.message().contains("short event duration")));
}

#[test]
fn fast_reading_speed_detected() {
    let long_text = "This is a very long text that would require fast reading speed to comprehend in the given short duration which may be difficult for some users";
    let script_text = format!(
        r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:02.00,Default,,0,0,0,,{long_text}"
    );

    let script = crate::parser::Script::parse(&script_text).unwrap();
    let rule = AccessibilityRule;
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let issues = rule.check_script(&analysis);

    assert!(issues
        .iter()
        .any(|issue| issue.message().contains("reading speed")));
}

#[test]
fn text_analysis_excludes_tags() {
    use crate::analysis::events::text_analysis::TextAnalysis;

    let analysis1 = TextAnalysis::analyze("Hello world").unwrap();
    assert_eq!(analysis1.char_count(), 11);

    // "Hello {\i1}world{\i0}" after removing tags becomes "Hello world" (11 chars)
    let analysis2 = TextAnalysis::analyze("Hello {\\i1}world{\\i0}").unwrap();
    assert_eq!(analysis2.char_count(), 11);

    // "{\b1}Bold{\b0} text" after removing tags becomes "Bold text" (9 chars)
    let analysis3 = TextAnalysis::analyze("{\\b1}Bold{\\b0} text").unwrap();
    assert_eq!(analysis3.char_count(), 9);

    let analysis4 = TextAnalysis::analyze("").unwrap();
    assert_eq!(analysis4.char_count(), 0);
}

#[test]
fn long_text_detected() {
    let long_text = "a".repeat(250);
    let script_text = format!(
        r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:10.00,Default,,0,0,0,,{long_text}"
    );

    let script = crate::parser::Script::parse(&script_text).unwrap();
    let rule = AccessibilityRule;
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let issues = rule.check_script(&analysis);

    assert!(issues
        .iter()
        .any(|issue| issue.message().contains("Very long text")));
}

#[test]
fn no_events_section_no_issues() {
    let script_text = r"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF&,&H000000FF&,&H00000000&,&H00000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let rule = AccessibilityRule;
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}
