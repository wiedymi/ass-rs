//! Tests for split, merge, timing, and toggle event commands.

use super::*;
use crate::commands::EditorCommand;
use crate::core::EditorDocument;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};
const TEST_CONTENT: &str = r#"[Script Info]
Title: Event Commands Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
Comment: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Third event
"#;

#[test]
fn test_split_event_command() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = SplitEventCommand::new(0, "0:00:03.00".to_string());
    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);

    // Should now have 4 events total (1 split into 2)
    let events_count = doc
        .text()
        .lines()
        .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
        .count();
    assert_eq!(events_count, 4);

    // Check split times
    assert!(doc.text().contains("0:00:01.00,0:00:03.00"));
    assert!(doc.text().contains("0:00:03.00,0:00:05.00"));
}

#[test]
fn test_merge_events_command() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = MergeEventsCommand::new(0, 1).with_separator(" | ".to_string());
    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);

    // Should now have 2 events total (2 merged into 1)
    let events_count = doc
        .text()
        .lines()
        .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
        .count();
    assert_eq!(events_count, 2);

    // Check merged text and timing
    assert!(doc.text().contains("First event | Second event"));
    assert!(doc.text().contains("0:00:01.00,0:00:10.00")); // Start of first, end of second
}

#[test]
fn test_timing_adjust_command() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Shift all events forward by 2 seconds (200 centiseconds)
    let command = TimingAdjustCommand::all_events(200, 200);
    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);

    // Check that times were adjusted
    assert!(doc.text().contains("0:00:03.00,0:00:07.00")); // First event shifted
    assert!(doc.text().contains("0:00:07.00,0:00:12.00")); // Second event shifted
    assert!(doc.text().contains("0:00:12.00,0:00:17.00")); // Third event shifted
}

#[test]
fn test_toggle_event_type_command() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = ToggleEventTypeCommand::single(0);
    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);

    // First event should now be Comment, others unchanged
    let text = doc.text();
    let lines: Vec<&str> = text.lines().collect();
    let event_lines: Vec<&str> = lines
        .iter()
        .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
        .copied()
        .collect();

    assert_eq!(event_lines.len(), 3);
    assert!(event_lines[0].starts_with("Comment:")); // Was Dialogue, now Comment
    assert!(event_lines[1].starts_with("Dialogue:")); // Unchanged
    assert!(event_lines[2].starts_with("Comment:")); // Unchanged
}

#[test]
fn test_split_event_invalid_time() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Try to split outside event bounds
    let command = SplitEventCommand::new(0, "0:00:00.50".to_string()); // Before event start
    let result = command.execute(&mut doc);

    assert!(result.is_err());
}

#[test]
fn test_merge_events_invalid_indices() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Try to merge with invalid order
    let command = MergeEventsCommand::new(1, 0); // Second before first
    let result = command.execute(&mut doc);

    assert!(result.is_err());
}

#[test]
fn test_timing_adjust_with_specific_events() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Adjust only first event
    let command = TimingAdjustCommand::new(vec![0], 100, 100); // +1 second
    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);

    // Only first event should be changed
    assert!(doc.text().contains("0:00:02.00,0:00:06.00")); // First event adjusted
    assert!(doc.text().contains("0:00:05.00,0:00:10.00")); // Second event unchanged
}
