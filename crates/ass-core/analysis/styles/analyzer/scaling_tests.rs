//! Tests for layout-to-play resolution scaling of resolved styles.

use super::*;

#[test]
fn analyzer_layout_resolution_scaling() {
    let script_text = r"
[Script Info]
Title: Resolution Scaling Test
LayoutResX: 640
LayoutResY: 480
PlayResX: 1280
PlayResY: 960

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,2,0,1,4,2,2,10,10,20,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    let default_style = analyzer.resolve_style("Default").unwrap();
    // Resolution is scaled 2x (1280/640 = 2, 960/480 = 2)
    assert!((default_style.font_size() - 40.0).abs() < f32::EPSILON); // 20 * 2
    assert!((default_style.spacing() - 4.0).abs() < f32::EPSILON); // 2 * 2
    assert!((default_style.outline() - 8.0).abs() < f32::EPSILON); // 4 * 2
    assert!((default_style.shadow() - 4.0).abs() < f32::EPSILON); // 2 * 2
    assert_eq!(default_style.margin_l(), 20); // 10 * 2
    assert_eq!(default_style.margin_r(), 20); // 10 * 2
    assert_eq!(default_style.margin_t(), 40); // 20 * 2
    assert_eq!(default_style.margin_b(), 40); // 20 * 2
}

#[test]
fn analyzer_layout_resolution_scaling_asymmetric() {
    let script_text = r"
[Script Info]
Title: Asymmetric Resolution Scaling Test
LayoutResX: 640
LayoutResY: 480
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,2,0,1,4,2,2,10,10,20,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    let default_style = analyzer.resolve_style("Default").unwrap();
    // X scale: 1920/640 = 3, Y scale: 1080/480 = 2.25, average = 2.625
    let avg_scale = 2.625;
    assert!((20.0f32.mul_add(-avg_scale, default_style.font_size())).abs() < 0.01);
    assert!((default_style.spacing() - 6.0).abs() < f32::EPSILON); // 2 * 3
    assert!((4.0f32.mul_add(-avg_scale, default_style.outline())).abs() < 0.01);
    assert!((2.0f32.mul_add(-avg_scale, default_style.shadow())).abs() < 0.01);
    assert_eq!(default_style.margin_l(), 30); // 10 * 3
    assert_eq!(default_style.margin_r(), 30); // 10 * 3
    assert_eq!(default_style.margin_t(), 45); // 20 * 2.25
    assert_eq!(default_style.margin_b(), 45); // 20 * 2.25
}

#[test]
fn analyzer_no_resolution_scaling_when_same() {
    let script_text = r"
[Script Info]
Title: No Scaling Test
LayoutResX: 1920
LayoutResY: 1080
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,2,0,1,4,2,2,10,10,20,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    let default_style = analyzer.resolve_style("Default").unwrap();
    // No scaling should be applied
    assert!((default_style.font_size() - 20.0).abs() < f32::EPSILON);
    assert!((default_style.spacing() - 2.0).abs() < f32::EPSILON);
    assert!((default_style.outline() - 4.0).abs() < f32::EPSILON);
    assert!((default_style.shadow() - 2.0).abs() < f32::EPSILON);
    assert_eq!(default_style.margin_l(), 10);
    assert_eq!(default_style.margin_r(), 10);
    assert_eq!(default_style.margin_t(), 20);
    assert_eq!(default_style.margin_b(), 20);
}
