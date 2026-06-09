//! Error recovery and resilience against empty and malformed scripts.
//!
//! Ensures the parser and analysis engine degrade gracefully instead of
//! panicking when fed incomplete sections, bad timing, or invalid overrides.

use ass_core::{
    analysis::ScriptAnalysis,
    parser::ast::{Section, SectionType},
    Script,
};

#[test]
fn test_empty_script_handling() {
    let empty_script = "";
    let script = Script::parse(empty_script).expect("Should handle empty script");

    assert!(script.sections().is_empty());
}

#[test]
fn test_malformed_script_resilience() {
    let malformed_script = r"[V4+ Styles]
Format: Name, Fontname
Style: Incomplete

[Events]
Format: Layer, Start, End, Text
Dialogue: 0,invalid_time,another_invalid,Malformed event
";

    // Should not panic on malformed input
    let result = Script::parse(malformed_script);
    // May succeed with defaults or fail gracefully
    if let Ok(script) = result {
        // If parsing succeeds, analysis should handle gracefully
        let _analysis_result = ScriptAnalysis::analyze(&script);
    } else {
        // Graceful failure is acceptable for malformed input
    }
}

/// Test comprehensive error recovery and malformed input handling
#[test]
fn test_error_recovery_comprehensive() {
    let malformed_script = r"[Script Info]
Title: Error Recovery Test
ScriptType: v4.00+

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1
Style: Incomplete,Arial,20  // Missing fields should be handled
Style: ,,,,,,,,,,,,,,,,,,,,,,   // Empty fields

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text

; Valid events
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Normal dialogue

; Malformed timing
Dialogue: 0,invalid_time,0:00:10.00,Default,,0,0,0,,Bad start time
Dialogue: 0,0:00:15.00,invalid_time,Default,,0,0,0,,Bad end time
Dialogue: 0,0:00:25.00,0:00:20.00,Default,,0,0,0,,End before start

; Missing required fields
Dialogue: 0,0:00:30.00,0:00:35.00  // Incomplete line
Dialogue: 0,0:00:40.00,0:00:45.00,NonexistentStyle,,0,0,0,,Missing style reference

; Malformed style overrides
Dialogue: 0,0:00:50.00,0:00:55.00,Default,,0,0,0,,{Unclosed override
Dialogue: 0,0:01:00.00,0:01:05.00,Default,,0,0,0,,{Invalid}override}content
Dialogue: 0,0:01:10.00,0:01:15.00,Default,,0,0,0,,{\invalid_tag}Unknown tag
Dialogue: 0,0:01:20.00,0:01:25.00,Default,,0,0,0,,{\\}Empty tag

; Binary/control characters
Dialogue: 0,0:01:30.00,0:01:35.00,Default,,0,0,0,,Content with binary data

; Invalid section
[Unknown Section]
SomeKey: SomeValue
AnotherKey: AnotherValue

; Partial sections
[Incomplete Section

[Another Section]
";

    // Should parse successfully despite errors
    let script = Script::parse(malformed_script).expect("Should parse with error recovery");

    // Should have accumulated issues but still produce a usable script
    assert!(!script.issues().is_empty(), "Should detect parsing issues");

    // Should still have valid sections
    assert!(script.find_section(SectionType::ScriptInfo).is_some());
    assert!(script.find_section(SectionType::Styles).is_some());
    assert!(script.find_section(SectionType::Events).is_some());

    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        // Should have parsed at least the valid events
        let valid_events = events
            .iter()
            .filter(|e| !e.start.is_empty() && !e.end.is_empty())
            .count();
        assert!(valid_events > 0, "Should parse some valid events");
    }

    #[cfg(feature = "analysis")]
    {
        // Analysis should handle malformed script gracefully
        let analysis_result = ScriptAnalysis::analyze(&script);
        assert!(
            analysis_result.is_ok(),
            "Analysis should handle errors gracefully"
        );

        if let Ok(analysis) = analysis_result {
            // Should detect issues through linting
            assert!(
                !analysis.lint_issues().is_empty(),
                "Should detect lint issues"
            );
        }
    }
}
