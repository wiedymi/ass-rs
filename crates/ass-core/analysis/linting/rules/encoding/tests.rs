//! Behavioural tests for the [`EncodingRule`] lint rule.

use super::*;
use crate::analysis::{
    linting::{IssueCategory, IssueSeverity, LintRule},
    ScriptAnalysis,
};
use alloc::format;

#[test]
fn rule_metadata_correct() {
    let rule = EncodingRule;
    assert_eq!(rule.id(), "encoding");
    assert_eq!(rule.name(), "Encoding");
    assert_eq!(
        rule.description(),
        "Detects potential encoding or character issues"
    );
    assert_eq!(rule.default_severity(), IssueSeverity::Warning);
    assert_eq!(rule.category(), IssueCategory::Encoding);
}

#[test]
fn empty_script_no_issues() {
    let script_text = "[Script Info]\nTitle: Test";
    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();

    let rule = EncodingRule;
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn valid_text_no_issues() {
    let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Valid text with unicode: ñáéíóú";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let rule = EncodingRule;
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn newlines_allowed() {
    let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text with\Nline break";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let rule = EncodingRule;
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn tabs_allowed() {
    let script_text = "[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text with\ttab";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let rule = EncodingRule;
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn replacement_character_detected() {
    let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text with � replacement";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let rule = EncodingRule;
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let issues = rule.check_script(&analysis);

    assert!(!issues.is_empty());
    assert!(issues
        .iter()
        .any(|issue| issue.message().contains("replacement character")));
}

#[test]
fn control_character_in_script_info() {
    let script_text = "[Script Info]\nTitle: Test\x00\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let rule = EncodingRule;
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let issues = rule.check_script(&analysis);

    assert!(!issues.is_empty());
    assert!(issues
        .iter()
        .any(|issue| issue.message().contains("control characters")));
}

#[test]
fn no_events_section_no_issues() {
    let script_text = r"[Script Info]
Title: Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF&,&H000000FF&,&H00000000&,&H00000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let rule = EncodingRule;
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn multibyte_characters_hint() {
    let heavy_unicode = "🎵🎶🎵🎶".repeat(20);
    let script_text = format!(
        r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{heavy_unicode}"
    );

    let script = crate::parser::Script::parse(&script_text).unwrap();
    let rule = EncodingRule;
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let issues = rule.check_script(&analysis);

    assert!(issues
        .iter()
        .any(|issue| issue.message().contains("multi-byte characters")));
}

#[test]
fn control_character_in_event_detected() {
    // Test control characters in events (not script info) to cover lines 101-104, 107, 110, 112
    let script_text = "[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Text with\x00control char";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let rule = EncodingRule;
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let issues = rule.check_script(&analysis);

    assert!(!issues.is_empty());
    assert!(issues
        .iter()
        .any(|issue| issue.message().contains("non-printable control characters")));

    // Check that the issue has description and suggested fix
    let control_issue = issues
        .iter()
        .find(|issue| issue.message().contains("non-printable control characters"))
        .unwrap();
    assert!(control_issue.description().is_some());
    assert!(control_issue.suggested_fix().is_some());
}
