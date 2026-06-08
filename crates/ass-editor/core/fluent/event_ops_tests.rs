//! Tests for event split/merge/timing and event deletion fluent operations.

use crate::core::EditorDocument;
use ass_core::parser::ast::EventType;

#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn test_fluent_event_operations() {
    const TEST_CONTENT: &str = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
Comment: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Third event
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Test split event
    doc.events().split(0, "0:00:03.00").unwrap();

    // Should now have 4 events (first split into 2)
    let events_count = doc
        .text()
        .lines()
        .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
        .count();
    assert_eq!(events_count, 4);
    assert!(doc.text().contains("0:00:01.00,0:00:03.00"));
    assert!(doc.text().contains("0:00:03.00,0:00:05.00"));
}

#[test]
fn test_fluent_event_merge() {
    const TEST_CONTENT: &str = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
Comment: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Third event
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Test merge events with custom separator
    doc.events()
        .merge(0, 1)
        .with_separator(" | ")
        .apply()
        .unwrap();

    // Should now have 2 events (first two merged)
    let events_count = doc
        .text()
        .lines()
        .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
        .count();
    assert_eq!(events_count, 2);
    assert!(doc.text().contains("First event | Second event"));
    assert!(doc.text().contains("0:00:01.00,0:00:10.00")); // Start of first, end of second
}

#[test]
fn test_fluent_event_timing() {
    const TEST_CONTENT: &str = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Test shifting all events by 2 seconds (200 centiseconds)
    doc.events().timing().shift(200).unwrap();

    assert!(doc.text().contains("0:00:03.00,0:00:07.00")); // First event shifted
    assert!(doc.text().contains("0:00:07.00,0:00:12.00")); // Second event shifted
}

#[test]
fn test_fluent_event_timing_specific() {
    const TEST_CONTENT: &str = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Test adjusting only first event
    doc.events()
        .timing()
        .event(0)
        .shift_start(100) // +1 second to start only
        .unwrap();

    // Only first event should change
    assert!(doc.text().contains("0:00:02.00,0:00:05.00")); // First event start shifted
    assert!(doc.text().contains("0:00:05.00,0:00:10.00")); // Second event unchanged
}

#[test]
fn test_event_delete_single() {
    const TEST_CONTENT: &str = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
Dialogue: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Third event
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Delete the second event (index 1)
    doc.events().delete(1).unwrap();

    let events = doc.events().all().unwrap();
    assert_eq!(events.len(), 2);
    assert!(doc.text().contains("First event"));
    assert!(!doc.text().contains("Second event"));
    assert!(doc.text().contains("Third event"));
}

#[test]
fn test_event_delete_multiple() {
    const TEST_CONTENT: &str = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,Event 1
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Event 2
Dialogue: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Event 3
Dialogue: 0,0:00:15.00,0:00:20.00,Default,Speaker,0,0,0,,Event 4
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Delete events at indices 0 and 2
    doc.events().delete_multiple(vec![0, 2]).unwrap();

    let events = doc.events().all().unwrap();
    assert_eq!(events.len(), 2);
    assert!(!doc.text().contains("Event 1"));
    assert!(doc.text().contains("Event 2"));
    assert!(!doc.text().contains("Event 3"));
    assert!(doc.text().contains("Event 4"));
}

#[test]
fn test_event_delete_query() {
    const TEST_CONTENT: &str = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,Keep this
Comment: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Delete this comment
Dialogue: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Keep this too
Comment: 0,0:00:15.00,0:00:20.00,Default,Speaker,0,0,0,,Delete this comment too
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Delete all comment events
    doc.events().delete_query().comments().unwrap();

    let events = doc.events().all().unwrap();
    assert_eq!(events.len(), 2);
    assert!(events
        .iter()
        .all(|e| e.event.event_type == EventType::Dialogue));
    assert!(!doc.text().contains("Delete this comment"));
}
