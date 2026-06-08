//! Tests for resolution scaling combined with inheritance and missing info.

use super::*;

#[test]
fn analyzer_resolution_scaling_with_inheritance() {
    let script_text = r"
[Script Info]
Title: Scaling with Inheritance Test
LayoutResX: 640
LayoutResY: 480
PlayResX: 1280
PlayResY: 960

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Base,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,2,0,1,4,2,2,10,10,20,1
Style: *Base,Derived,Verdana,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,15,15,20,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    let base_style = analyzer.resolve_style("Base").unwrap();
    let derived_style = analyzer.resolve_style("Derived").unwrap();

    // Base style should be scaled 2x
    assert!((base_style.font_size() - 40.0).abs() < f32::EPSILON);

    // Derived style overrides font size to 24, which should be scaled to 48
    assert!((derived_style.font_size() - 48.0).abs() < f32::EPSILON);
    // Margins are overridden and should be scaled
    assert_eq!(derived_style.margin_l(), 30); // 15 * 2
    assert_eq!(derived_style.margin_r(), 30); // 15 * 2
                                              // Since margin_v is "20", it should be scaled to 40
    assert_eq!(derived_style.margin_t(), 40); // 20 * 2
    assert_eq!(derived_style.margin_b(), 40); // 20 * 2
}

#[test]
fn analyzer_no_resolution_info_no_scaling() {
    let script_text = r"
[Script Info]
Title: No Resolution Info Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,2,0,1,4,2,2,10,10,20,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    let default_style = analyzer.resolve_style("Default").unwrap();
    // No scaling should be applied when resolution info is missing
    assert!((default_style.font_size() - 20.0).abs() < f32::EPSILON);
    assert!((default_style.spacing() - 2.0).abs() < f32::EPSILON);
    assert!((default_style.outline() - 4.0).abs() < f32::EPSILON);
    assert!((default_style.shadow() - 2.0).abs() < f32::EPSILON);
    assert_eq!(default_style.margin_l(), 10);
    assert_eq!(default_style.margin_r(), 10);
    assert_eq!(default_style.margin_t(), 20);
    assert_eq!(default_style.margin_b(), 20);
}

#[test]
fn analyzer_partial_resolution_info_no_scaling() {
    let script_text = r"
[Script Info]
Title: Partial Resolution Info Test
LayoutResX: 640
PlayResY: 960

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,2,0,1,4,2,2,10,10,20,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    let default_style = analyzer.resolve_style("Default").unwrap();
    // No scaling should be applied when resolution info is incomplete
    assert!((default_style.font_size() - 20.0).abs() < f32::EPSILON);
    assert!((default_style.spacing() - 2.0).abs() < f32::EPSILON);
    assert!((default_style.outline() - 4.0).abs() < f32::EPSILON);
    assert!((default_style.shadow() - 2.0).abs() < f32::EPSILON);
    assert_eq!(default_style.margin_l(), 10);
    assert_eq!(default_style.margin_r(), 10);
    assert_eq!(default_style.margin_t(), 20);
    assert_eq!(default_style.margin_b(), 20);
}
