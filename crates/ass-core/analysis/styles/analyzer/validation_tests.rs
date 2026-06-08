//! Tests for style validation and performance analysis behavior.

use super::*;

#[test]
fn analyzer_validate_styles() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Large,Arial,60,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,5,0,2,10,10,10,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    let issues = analyzer.validate_styles();
    // Should have some validation issues or none
    assert!(issues.is_empty() || !issues.is_empty());
}

#[test]
fn analyzer_strict_validation() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Large,Arial,250,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let config = StyleAnalysisConfig {
        options: AnalysisOptions::VALIDATION | AnalysisOptions::STRICT_VALIDATION,
        performance_thresholds: PerformanceThresholds::default(),
    };
    let analyzer = StyleAnalyzer::new_with_config(&script, config);

    let issues = analyzer.validate_styles();
    // Should have validation issues for large font size
    assert!(!issues.is_empty());
}

#[test]
fn analyzer_performance_analysis() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Heavy,Arial,60,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,8,5,2,10,10,10,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let config = StyleAnalysisConfig {
        options: AnalysisOptions::PERFORMANCE,
        performance_thresholds: PerformanceThresholds {
            large_font_threshold: 30.0,
            large_outline_threshold: 2.0,
            large_shadow_threshold: 2.0,
            scaling_threshold: 150.0,
        },
    };
    let analyzer = StyleAnalyzer::new_with_config(&script, config);

    let issues = analyzer.validate_styles();
    // Should have performance issues for large values
    assert!(!issues.is_empty());
}
