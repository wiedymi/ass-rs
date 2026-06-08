//! Tests for override-block parsing across varied tag scenarios.

use super::*;

#[test]
fn text_analysis_empty_override_blocks() {
    let text = "Text {} more text";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "Text  more text");
    // Should have diagnostic for empty override
    assert!(!analysis.diagnostics().is_empty());
}

#[test]
fn text_analysis_unmatched_braces() {
    let text = "Text {\\b1 unmatched";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "Text ");
    // Should handle unmatched braces gracefully
}

#[test]
fn text_analysis_multiple_override_blocks() {
    let text = "{\\b1}Bold{\\b0} and {\\i1}italic{\\i0} text";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "Bold and italic text");
    assert_eq!(analysis.override_tags().len(), 4);
}

#[test]
fn text_analysis_complex_tags() {
    let text =
        "{\\move(0,0,100,100)}{\\t(0,1000,\\fscx120)}{\\fade(255,0,0,0,800,900,1000)}Animated text";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "Animated text");
    assert!(!analysis.override_tags().is_empty());
}

#[test]
fn text_analysis_drawing_commands() {
    let text = "{\\p1}m 0 0 l 100 0 100 100 0 100{\\p0}Square";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "Square");
    assert!(!analysis.override_tags().is_empty());
}

#[test]
fn text_analysis_color_tags() {
    let text = "{\\c&H0000FF&}Red text{\\c} and {\\1c&H00FF00&}green text";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "Red text and green text");
    assert!(!analysis.override_tags().is_empty());
}

#[test]
fn text_analysis_mixed_content() {
    let text = "Start {\\b1}bold\\N{\\i1}italic{\\i0}{\\b0}\\hnormal end";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(
        analysis.plain_text(),
        "Start bold\nitalic\u{00A0}normal end"
    );
    assert_eq!(analysis.line_count(), 2);
    assert!(!analysis.override_tags().is_empty());
}

#[test]
fn text_analysis_whitespace_only() {
    let text = "   \t\n  ";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "   \t\n  ");
    assert_eq!(analysis.char_count(), 7);
    assert_eq!(analysis.line_count(), 2);
}

#[test]
fn text_analysis_empty_text() {
    let text = "";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "");
    assert_eq!(analysis.char_count(), 0);
    assert_eq!(analysis.line_count(), 1); // Minimum 1 line
    assert!(analysis.override_tags().is_empty());
}

#[test]
fn text_analysis_only_override_tags() {
    let text = "{\\b1}{\\i1}{\\u1}";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "");
    assert_eq!(analysis.char_count(), 0);
    assert!(!analysis.override_tags().is_empty());
}

#[test]
fn text_analysis_escape_sequences() {
    let text = "Test`[Events]`backslash and \\{brace and \\}close";
    let analysis = TextAnalysis::analyze(text).unwrap();

    // These should be treated as literal characters, not escape sequences
    assert_eq!(
        analysis.plain_text(),
        "Test`[Events]`backslash and \\{brace and \\}close"
    );
}

#[test]
fn text_analysis_karaoke_tags() {
    let text = "{\\k50}Ka{\\k30}ra{\\k70}o{\\k40}ke";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "Karaoke");
    assert!(!analysis.override_tags().is_empty());
}

#[test]
fn text_analysis_position_and_rotation() {
    let text = "{\\pos(320,240)}{\\frz45}Rotated positioned text";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "Rotated positioned text");
    assert!(!analysis.override_tags().is_empty());
}
