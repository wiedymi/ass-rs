//! Tests for line-count edge cases, drawing mode, and brace-depth limits.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

#[test]
fn text_analysis_very_long_text() {
    let text = "A".repeat(1000);
    let analysis = TextAnalysis::analyze(&text).unwrap();

    assert_eq!(analysis.char_count(), 1000);
    assert_eq!(analysis.plain_text().len(), 1000);
}

#[test]
fn text_analysis_line_count_edge_cases() {
    // Text ending with newline
    let text1 = "Line 1\\nLine 2\\n";
    let analysis1 = TextAnalysis::analyze(text1).unwrap();
    assert_eq!(analysis1.line_count(), 2);

    // Multiple consecutive newlines
    let text2 = "Line 1\\n\\n\\nLine 2";
    let analysis2 = TextAnalysis::analyze(text2).unwrap();
    assert_eq!(analysis2.line_count(), 4);

    // Only newlines
    let text3 = "\\n\\N\\n";
    let analysis3 = TextAnalysis::analyze(text3).unwrap();
    assert_eq!(analysis3.line_count(), 4);
}

#[test]
fn text_analysis_excessive_brace_nesting() {
    // Create deeply nested braces to trigger depth limit error
    let mut text = String::new();
    for _ in 0..110 {
        text.push('{');
    }
    text.push_str("\\b1");
    for _ in 0..110 {
        text.push('}');
    }

    let result = TextAnalysis::analyze(&text);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Maximum brace nesting depth exceeded"));
}

#[test]
fn text_analysis_drawing_mode_escape_sequences() {
    // Test escape sequences in drawing mode - they should not be processed
    let text = "{\\p1}Line1\\nLine2\\hSpace\\NNewline{\\p0}Normal\\ntext";
    let analysis = TextAnalysis::analyze(text).unwrap();

    // In drawing mode, text is ignored entirely from plain_text
    // After {p0}, normal processing resumes
    assert_eq!(analysis.plain_text(), "Normal\ntext");
    assert!(!analysis.override_tags().is_empty());
}

#[test]
fn text_analysis_drawing_mode_p_value_parsing() {
    // Test various p values to trigger drawing mode logic
    let text1 = "{\\p0}Not drawing mode";
    let analysis1 = TextAnalysis::analyze(text1).unwrap();
    assert_eq!(analysis1.plain_text(), "Not drawing mode");

    let text2 = "{\\p1}Drawing mode";
    let analysis2 = TextAnalysis::analyze(text2).unwrap();
    assert_eq!(analysis2.plain_text(), ""); // Drawing mode excludes text

    let text3 = "{\\p5}Also drawing mode";
    let analysis3 = TextAnalysis::analyze(text3).unwrap();
    assert_eq!(analysis3.plain_text(), ""); // Drawing mode excludes text
}

#[test]
fn text_analysis_line_count_only_newlines() {
    // Test line counting when text is only newlines (line 252)
    let text = "\n\n\n";
    let analysis = TextAnalysis::analyze(text).unwrap();
    assert_eq!(analysis.line_count(), 4); // 3 newlines = 4 lines
}

#[test]
fn text_analysis_drawing_mode_mixed_escapes() {
    // Test all escape sequence types in drawing mode
    let text = "{\\p1}Start\\nNew\\NLine\\hHard{\\p0}End\\nNormal";
    let analysis = TextAnalysis::analyze(text).unwrap();

    // Drawing mode excludes all text, normal mode processes escape sequences
    assert_eq!(analysis.plain_text(), "End\nNormal");
    assert!(!analysis.override_tags().is_empty());
}
