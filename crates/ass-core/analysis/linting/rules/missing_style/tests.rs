//! Tests for the missing style reference detection rule.

use super::*;
use crate::analysis::{
    linting::{IssueCategory, IssueSeverity, LintRule},
    ScriptAnalysis,
};

#[test]
fn rule_metadata_correct() {
    let rule = MissingStyleRule;
    assert_eq!(rule.id(), "missing-style");
    assert_eq!(rule.name(), "Missing Style");
    assert_eq!(
        rule.description(),
        "Detects events referencing non-existent styles"
    );
    assert_eq!(rule.default_severity(), IssueSeverity::Error);
    assert_eq!(rule.category(), IssueCategory::Styling);
}

#[test]
fn empty_script_no_issues() {
    let script_text = "[Script Info]\nTitle: Test";
    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();

    let rule = MissingStyleRule;
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn valid_style_reference_no_issues() {
    let script_text = r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF&,&H000000FF&,&H00000000&,&H00000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Valid event";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let rule = MissingStyleRule;
    let issues = rule.check_script(&analysis);

    assert!(issues.is_empty());
}

#[test]
fn missing_style_reference_detected() {
    let script_text = r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF&,&H000000FF&,&H00000000&,&H00000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Undefined,,0,0,0,,Invalid event";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let rule = MissingStyleRule;
    let issues = rule.check_script(&analysis);

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].severity(), IssueSeverity::Error);
    assert_eq!(issues[0].category(), IssueCategory::Styling);
    assert!(issues[0].message().contains("Undefined"));
}

#[test]
fn multiple_missing_styles() {
    let script_text = r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF&,&H000000FF&,&H00000000&,&H00000000&,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Missing1,,0,0,0,,First invalid
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Valid event
Dialogue: 0,0:00:10.00,0:00:15.00,Missing2,,0,0,0,,Second invalid";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let rule = MissingStyleRule;
    let issues = rule.check_script(&analysis);

    assert_eq!(issues.len(), 2);
}

#[test]
fn no_styles_section_all_invalid() {
    let script_text = r"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Should be invalid";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();
    let rule = MissingStyleRule;
    let issues = rule.check_script(&analysis);

    assert_eq!(issues.len(), 1);
}
