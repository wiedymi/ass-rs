//! Unit tests for the top-level script analysis facade.

use super::*;

#[test]
fn analysis_config_default() {
    let config = AnalysisConfig::default();
    assert!(config
        .options
        .contains(ScriptAnalysisOptions::UNICODE_LINEBREAKS));
    assert!(config
        .options
        .contains(ScriptAnalysisOptions::PERFORMANCE_HINTS));
    assert!(!config
        .options
        .contains(ScriptAnalysisOptions::STRICT_COMPLIANCE));
    assert_eq!(config.max_events_threshold, 1000);
}

#[test]
fn script_analysis_basic() {
    let script_text = r"
[Script Info]
Title: Test Script

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events\]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Second line
";

    let script = crate::parser::Script::parse(script_text).unwrap();
    let analysis = ScriptAnalysis::analyze(&script).unwrap();

    assert_eq!(analysis.lint_issues().len(), 0);
    assert!(!analysis.has_critical_issues());

    let perf = analysis.performance_summary();
    assert!(perf.performance_score > 0);
}

#[test]
fn performance_summary_recommendations() {
    let summary = PerformanceSummary {
        total_events: 100,
        overlapping_events: 15,
        complex_animations: 5,
        large_fonts: 2,
        performance_score: 75,
    };

    assert!(summary.has_performance_issues());
    assert!(summary.recommendation().is_some());
    assert!(summary
        .recommendation()
        .unwrap()
        .contains("overlapping events"));
}
