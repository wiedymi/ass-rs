//! Crate-level integration tests verifying core parsing, version detection,
//! analysis, and error handling work together end-to-end.

use super::*;
#[cfg(feature = "analysis")]
use crate::analysis::{
    linting::{lint_script, LintConfig},
    ScriptAnalysis,
};

/// Comprehensive integration test verifying core functionality works correctly
#[test]
fn test_core_functionality_integration() {
    let script_text = r"
[Script Info]
Title: Test Script
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1
Style: Large,Arial,80,&H00FF0000,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!
Dialogue: 0,0:00:02.00,0:00:07.00,Default,,0,0,0,,Overlapping dialogue
Dialogue: 0,0:00:10.00,0:00:15.00,Large,,0,0,0,,{\t(0,1000,\fscx200\fscy200)}Large animated text
Comment: 0,0:00:30.00,0:00:35.00,Default,,0,0,0,,This is a comment
";

    let script = Script::parse(script_text).expect("Should parse script successfully");
    assert!(
        script.sections().len() >= 2,
        "Should have parsed multiple sections"
    );

    let version = ScriptVersion::from_header("v4.00+").expect("Should detect script version");
    assert_eq!(version, ScriptVersion::AssV4);
    assert!(!version.supports_extensions());

    let version_plus =
        ScriptVersion::from_header("v4.00++").expect("Should detect extended version");
    assert_eq!(version_plus, ScriptVersion::AssV4Plus);
    assert!(version_plus.supports_extensions());

    #[cfg(feature = "analysis")]
    {
        let analysis =
            ScriptAnalysis::analyze(&script).expect("Should analyze script successfully");

        assert!(
            !analysis.resolved_styles().is_empty(),
            "Should resolve styles"
        );

        assert!(
            !analysis.dialogue_info().is_empty(),
            "Should analyze dialogue events"
        );

        let perf = analysis.performance_summary();
        assert!(
            perf.performance_score <= 100,
            "Performance score should be valid"
        );

        let default_style = analysis.resolve_style("Default");
        assert!(default_style.is_some(), "Should find Default style");

        let lint_config = LintConfig::default();
        let issues = lint_script(&script, &lint_config).expect("Should run linting successfully");

        assert!(!issues.is_empty(), "Should detect some lint issues");
    }
}

#[test]
fn test_script_version_functionality() {
    assert_eq!(
        ScriptVersion::from_header("v4.00"),
        Some(ScriptVersion::SsaV4)
    );
    assert_eq!(
        ScriptVersion::from_header("v4.00+"),
        Some(ScriptVersion::AssV4)
    );
    assert_eq!(
        ScriptVersion::from_header("v4.00++"),
        Some(ScriptVersion::AssV4Plus)
    );
    assert_eq!(
        ScriptVersion::from_header("v4.00+ extended"),
        Some(ScriptVersion::AssV4Plus)
    );
    assert_eq!(ScriptVersion::from_header("invalid"), None);

    assert!(!ScriptVersion::SsaV4.supports_extensions());
    assert!(!ScriptVersion::AssV4.supports_extensions());
    assert!(ScriptVersion::AssV4Plus.supports_extensions());
}

#[test]
fn test_error_handling() {
    let invalid_script = "This is not a valid ASS script";
    let result = Script::parse(invalid_script);

    if let Ok(script) = result {
        assert!(
            !script.issues().is_empty(),
            "Invalid script should have parse issues"
        );
    }
}

#[test]
fn test_empty_script_handling() {
    let empty_script = "";
    let result = Script::parse(empty_script);

    assert!(result.is_ok(), "Should handle empty script gracefully");

    let script = result.unwrap();
    assert_eq!(
        script.sections().len(),
        0,
        "Empty script should have no sections"
    );
}
