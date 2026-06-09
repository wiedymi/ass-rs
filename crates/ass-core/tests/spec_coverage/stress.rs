//! Stress scenarios: large event counts and deeply nested override sequences.
//!
//! Generates synthetic scripts to confirm parsing and analysis scale to many
//! events and dense, overlapping animation tags.

use std::fmt::Write;

use ass_core::{
    analysis::ScriptAnalysis,
    parser::ast::{Section, SectionType},
    utils::format_ass_time,
    Script,
};

#[test]
fn test_large_script_handling() {
    // Generate a large script with many events
    let mut large_script = String::from(
        r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
",
    );

    // Add 1000 dialogue events
    for i in 0..1000 {
        let start_cs = i * 500; // 5 second intervals (500 centiseconds)
        let end_cs = start_cs + 400; // 4 second duration (400 centiseconds)

        writeln!(
            large_script,
            "Dialogue: 0,{},{},Default,,0,0,0,,Event {} with some text content.",
            format_ass_time(start_cs),
            format_ass_time(end_cs),
            i
        )
        .unwrap();
    }

    let script = Script::parse(&large_script).expect("Failed to parse large script");

    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        assert_eq!(events.len(), 1000);
    } else {
        panic!("Events section should be present");
    }

    // Analysis should handle large scripts efficiently
    let analysis = ScriptAnalysis::analyze(&script).expect("Failed to analyze large script");
    assert_eq!(analysis.dialogue_info().len(), 1000);
}

/// Test performance edge cases and stress scenarios
#[test]
fn test_performance_edge_cases() {
    // Test very dense overlapping events
    let mut dense_script = String::from(
        r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
",
    );

    // Create many overlapping events in a short time span
    for i in 0..100 {
        let start_cs = i * 10; // Every 0.1 seconds
        let end_cs = start_cs + 500; // 5 second duration each
        writeln!(
            dense_script,
            "Dialogue: {},{},{},Default,,0,0,0,,Overlapping event {} content.",
            i % 10, // Different layers
            format_ass_time(start_cs),
            format_ass_time(end_cs),
            i
        )
        .unwrap();
    }

    let script = Script::parse(&dense_script).expect("Failed to parse dense script");

    #[cfg(feature = "analysis")]
    {
        let analysis = ScriptAnalysis::analyze(&script).expect("Failed to analyze dense script");

        // Should detect many overlaps
        let performance = analysis.performance_summary();
        assert!(
            performance.overlapping_events > 50,
            "Should detect many overlapping events"
        );
        assert!(
            performance.performance_score < 90,
            "Performance score should reflect complexity"
        );
    }

    // Test deeply nested style overrides
    let nested_overrides = format!(
        "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{}Text{}",
        "{\\b1\\i1\\u1\\s1\\fscx150\\fscy75\\frz45\\c&H0000FF&\\alpha&H80&\\pos(100,200)\\move(100,200,300,400)\\t(0,1000,\\fscx200)\\t(1000,2000,\\fscy50)\\blur2\\be1\\bord3\\shad2}".repeat(5),
        "{\\r}".repeat(5)
    );

    let complex_script = format!(
        r"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
{nested_overrides}
"
    );

    let script = Script::parse(&complex_script).expect("Failed to parse complex script");

    #[cfg(feature = "analysis")]
    {
        let analysis = ScriptAnalysis::analyze(&script).expect("Failed to analyze complex script");

        // Should handle complex analysis
        assert!(!analysis.dialogue_info().is_empty());
        let complex_animation_count = analysis
            .dialogue_info()
            .iter()
            .filter(|info| info.animation_score() > 5)
            .count();
        assert!(
            complex_animation_count > 0,
            "Should detect very complex animations"
        );
    }
}
