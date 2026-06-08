//! Tests for basic analyzer construction and style enumeration.

use super::*;

#[test]
fn analyzer_creation() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    assert_eq!(analyzer.resolved_styles().len(), 1);
    assert!(analyzer.resolve_style("Default").is_some());
}

#[test]
fn analyzer_multiple_styles() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Title,Arial,32,&H00FFFF00,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,3,0,2,20,20,20,1
Style: Subtitle,Arial,16,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,1,0,0,100,100,0,0,1,1,0,2,5,5,5,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    assert_eq!(analyzer.resolved_styles().len(), 3);
    assert!(analyzer.resolve_style("Default").is_some());
    assert!(analyzer.resolve_style("Title").is_some());
    assert!(analyzer.resolve_style("Subtitle").is_some());
    assert!(analyzer.resolve_style("NonExistent").is_none());
}

#[test]
fn analyzer_duplicate_styles() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Default,Times,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    let conflicts = analyzer.conflicts();
    assert!(!conflicts.is_empty());
}

#[test]
fn analyzer_no_styles_section() {
    let script_text = r"
[Script Info]
Title: Test Script
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    assert_eq!(analyzer.resolved_styles().len(), 0);
    assert!(analyzer.conflicts().is_empty());
}

#[test]
fn analyzer_empty_styles_section() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    assert_eq!(analyzer.resolved_styles().len(), 0);
    assert!(analyzer.conflicts().is_empty());
}
