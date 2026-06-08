//! Tests for analyzer configuration, options flags, debug, and clone behavior.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::format;

#[test]
fn config_defaults() {
    let config = StyleAnalysisConfig::default();
    assert!(config.options.contains(AnalysisOptions::INHERITANCE));
    assert!(config.options.contains(AnalysisOptions::CONFLICTS));
    assert!(config.options.contains(AnalysisOptions::VALIDATION));
    assert!(!config.options.contains(AnalysisOptions::STRICT_VALIDATION));
}

#[test]
fn performance_thresholds() {
    let thresholds = PerformanceThresholds::default();
    assert!((thresholds.large_font_threshold - 50.0).abs() < f32::EPSILON);
    assert!((thresholds.large_outline_threshold - 4.0).abs() < f32::EPSILON);
    assert!((thresholds.large_shadow_threshold - 4.0).abs() < f32::EPSILON);
    assert!((thresholds.scaling_threshold - 200.0).abs() < f32::EPSILON);
}

#[test]
fn analyzer_options_flags() {
    let options = AnalysisOptions::INHERITANCE | AnalysisOptions::CONFLICTS;
    assert!(options.contains(AnalysisOptions::INHERITANCE));
    assert!(options.contains(AnalysisOptions::CONFLICTS));
    assert!(!options.contains(AnalysisOptions::VALIDATION));
    assert!(!options.contains(AnalysisOptions::PERFORMANCE));
    assert!(!options.contains(AnalysisOptions::STRICT_VALIDATION));
}

#[test]
fn analyzer_options_debug() {
    let options = AnalysisOptions::INHERITANCE;
    let debug_str = format!("{options:?}");
    assert!(debug_str.contains("INHERITANCE"));
}

#[test]
fn analyzer_config_debug() {
    let config = StyleAnalysisConfig::default();
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("StyleAnalysisConfig"));
    assert!(debug_str.contains("options"));
    assert!(debug_str.contains("performance_thresholds"));
}

#[test]
fn analyzer_debug() {
    let script_text = r"
[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analyzer = StyleAnalyzer::new(&script);
    let debug_str = format!("{analyzer:?}");
    assert!(debug_str.contains("StyleAnalyzer"));
}

#[test]
fn performance_thresholds_debug() {
    let thresholds = PerformanceThresholds::default();
    let debug_str = format!("{thresholds:?}");
    assert!(debug_str.contains("PerformanceThresholds"));
    assert!(debug_str.contains("large_font_threshold"));
}

#[test]
fn config_clone() {
    let config = StyleAnalysisConfig::default();
    let cloned = config.clone();
    assert_eq!(config.options, cloned.options);
    assert!(
        (config.performance_thresholds.large_font_threshold
            - cloned.performance_thresholds.large_font_threshold)
            .abs()
            < f32::EPSILON
    );
}

#[test]
fn performance_thresholds_clone() {
    let thresholds = PerformanceThresholds::default();
    let cloned = thresholds.clone();
    assert!((thresholds.large_font_threshold - cloned.large_font_threshold).abs() < f32::EPSILON);
    assert!(
        (thresholds.large_outline_threshold - cloned.large_outline_threshold).abs() < f32::EPSILON
    );
    assert!(
        (thresholds.large_shadow_threshold - cloned.large_shadow_threshold).abs() < f32::EPSILON
    );
    assert!((thresholds.scaling_threshold - cloned.scaling_threshold).abs() < f32::EPSILON);
}
