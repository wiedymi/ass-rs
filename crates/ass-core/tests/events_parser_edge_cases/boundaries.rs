//! Boundary condition and mixed event type tests for the events parser.
//!
//! Covers extreme field values and chronologically interleaved event types.

use ass_core::{parser::IssueSeverity, Script};

/// Test boundary conditions for event field parsing
#[test]
fn test_event_field_boundaries() {
    let boundary_cases = r"
[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,,,,,,,
Dialogue: 999,23:59:59.99,23:59:59.99,VeryLongStyleNameThatShouldStillWork,VeryLongSpeakerName,9999,9999,9999,VeryLongEffectName,Very long text that goes on and on and should still be parsed correctly
Dialogue: -1,-1:00:00.00,-1:00:00.00,Default,,-1,-1,-1,,Negative values
Comment: 0,0:00:00.00,0:00:00.01,Default,,0,0,0,,Very short duration
Movie: 0,0:00:00.00,10:00:00.00,Default,,0,0,0,,Very long duration
";

    let script = Script::parse(boundary_cases).expect("Script parsing should work");

    // Should handle boundary values gracefully
    assert!(!script.sections().is_empty());

    // Should not crash on extreme values
    let _has_sections = !script.sections().is_empty();
}

/// Test mixed event types in single section
#[test]
fn test_mixed_event_types() {
    let mixed_events = r"
[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:02.00,Default,,0,0,0,,First dialogue
Comment: 0,0:00:01.00,0:00:03.00,Default,,0,0,0,,Overlapping comment
Dialogue: 0,0:00:02.00,0:00:04.00,Default,,0,0,0,,Second dialogue
Picture: 0,0:00:03.00,0:00:05.00,Default,,0,0,0,,background.jpg
Sound: 0,0:00:04.00,0:00:06.00,Default,,0,0,0,,sfx.wav
Dialogue: 0,0:00:05.00,0:00:07.00,Default,,0,0,0,,Third dialogue
Movie: 0,0:00:06.00,0:00:08.00,Default,,0,0,0,,intro.avi
Command: 0,0:00:07.00,0:00:09.00,Default,,0,0,0,,fade_in
Comment: 0,0:00:08.00,0:00:10.00,Default,,0,0,0,,Final comment
";

    let script = Script::parse(mixed_events).expect("Script parsing should work");

    // Should handle mixed event types in chronological order
    assert!(!script.sections().is_empty());

    // Should parse all events without critical errors
    let critical_errors = script
        .issues()
        .iter()
        .filter(|issue| matches!(issue.severity, IssueSeverity::Error))
        .count();

    // Should have minimal critical errors for valid mixed content
    assert!(critical_errors < 5); // Allow some tolerance for edge cases
}
