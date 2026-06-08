//! Tests for plain-text extraction, override-tag stripping, and escape handling.

use super::*;

#[test]
fn text_analysis_simple_text() {
    let text = "Hello world!";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "Hello world!");
    assert_eq!(analysis.char_count(), 12);
    assert_eq!(analysis.line_count(), 1);
    assert!(!analysis.has_bidi_text());
    assert!(!analysis.has_complex_unicode());
    assert!(analysis.override_tags().is_empty());
    assert!(analysis.diagnostics().is_empty());
}

#[test]
fn text_analysis_with_override_tags() {
    let text = "Hello {\\b1}bold{\\b0} world!";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "Hello bold world!");
    assert_eq!(analysis.char_count(), 17);
    assert_eq!(analysis.line_count(), 1);
    assert!(!analysis.override_tags().is_empty());
}

#[test]
fn text_analysis_nested_braces() {
    let text = "Text {\\pos(100,{\\some}200)} more text";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "Text  more text");
    assert!(!analysis.override_tags().is_empty());
}

#[test]
fn text_analysis_line_breaks() {
    let text = "First line\\NSecond line\\nThird line";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "First line\nSecond line\nThird line");
    assert_eq!(analysis.line_count(), 3);
}

#[test]
fn text_analysis_hard_spaces() {
    let text = "Text\\hwith\\hhard\\hspaces";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(
        analysis.plain_text(),
        "Text\u{00A0}with\u{00A0}hard\u{00A0}spaces"
    );
}

#[test]
fn text_analysis_mixed_escapes() {
    let text = "Line 1\\NLine 2\\hspace\\nLine 3";
    let analysis = TextAnalysis::analyze(text).unwrap();

    assert_eq!(analysis.plain_text(), "Line 1\nLine 2\u{00A0}space\nLine 3");
    assert_eq!(analysis.line_count(), 3);
}
