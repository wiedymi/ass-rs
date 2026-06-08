//! Tests for custom configuration, style extraction, and inheritance metadata.

use super::*;

#[test]
fn analyzer_with_custom_config() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let config = StyleAnalysisConfig {
        options: AnalysisOptions::VALIDATION | AnalysisOptions::STRICT_VALIDATION,
        performance_thresholds: PerformanceThresholds {
            large_font_threshold: 30.0,
            large_outline_threshold: 2.0,
            large_shadow_threshold: 2.0,
            scaling_threshold: 150.0,
        },
    };
    let analyzer = StyleAnalyzer::new_with_config(&script, config);

    assert_eq!(analyzer.resolved_styles().len(), 1);
    assert!(analyzer.resolve_style("Default").is_some());
}

#[test]
fn analyzer_extract_styles() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    let styles = analyzer.extract_styles();
    assert!(styles.is_some());
    assert_eq!(styles.unwrap().len(), 1);
}

#[test]
fn analyzer_extract_styles_no_section() {
    let script_text = r"
[Script Info]
Title: Test Script
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    let styles = analyzer.extract_styles();
    assert!(styles.is_none());
}

#[test]
fn analyzer_inheritance_info() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Title,Arial,32,&H00FFFF00,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,3,0,2,20,20,20,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);

    let inheritance_info = analyzer.inheritance_info();
    assert_eq!(inheritance_info.len(), 2);
    assert!(inheritance_info.contains_key("Default"));
    assert!(inheritance_info.contains_key("Title"));
}

#[test]
fn analyzer_minimal_options() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let config = StyleAnalysisConfig {
        options: AnalysisOptions::empty(),
        performance_thresholds: PerformanceThresholds::default(),
    };
    let analyzer = StyleAnalyzer::new_with_config(&script, config);

    assert_eq!(analyzer.resolved_styles().len(), 1);
    assert!(analyzer.inheritance_info().is_empty());
    assert!(analyzer.conflicts().is_empty());
}
