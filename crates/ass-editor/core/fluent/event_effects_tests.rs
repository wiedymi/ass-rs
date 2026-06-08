//! Tests for event type toggling and effect modification fluent operations.

use crate::core::EditorDocument;

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

#[test]
fn test_fluent_event_toggle() {
    const TEST_CONTENT: &str = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Comment: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Test toggling first event type
    doc.events().toggle_type().event(0).apply().unwrap();

    let text = doc.text();
    let lines: Vec<&str> = text.lines().collect();
    let event_lines: Vec<&str> = lines
        .iter()
        .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
        .copied()
        .collect();

    // First event should now be Comment (was Dialogue)
    assert_eq!(event_lines.len(), 2);
    assert!(event_lines[0].starts_with("Comment:"));
    assert!(event_lines[1].starts_with("Comment:")); // Second unchanged
}

#[test]
fn test_fluent_event_effects() {
    const TEST_CONTENT: &str = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Test setting effects
    doc.events()
        .effects()
        .events(vec![0, 1])
        .set("Fade(255,0)")
        .unwrap();

    // Both events should have the effect
    let text = doc.text();
    let event_lines: Vec<&str> = text
        .lines()
        .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
        .collect();

    assert!(event_lines[0].contains("Fade(255,0)"));
    assert!(event_lines[1].contains("Fade(255,0)"));
}

#[test]
fn test_fluent_event_effects_chaining() {
    const TEST_CONTENT: &str = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Test effect chaining: set, then append
    doc.events().effects().event(0).set("Fade(255,0)").unwrap();

    doc.events()
        .effects()
        .event(0)
        .append("Move(100,200)")
        .unwrap();

    // Should have both effects
    assert!(doc.text().contains("Fade(255,0) Move(100,200)"));

    // Test clearing
    doc.events().effects().event(0).clear().unwrap();

    // Effect field should be empty
    let text = doc.text();
    let event_line = text
        .lines()
        .find(|line| line.starts_with("Dialogue:"))
        .unwrap();
    let parts: Vec<&str> = event_line.split(',').collect();
    assert_eq!(parts[8].trim(), ""); // Effect field should be empty
}

#[test]
fn test_fluent_event_complex_workflow() {
    const TEST_CONTENT: &str = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,Long event that needs splitting
Dialogue: 0,0:00:05.00,0:00:07.00,Default,Speaker,0,0,0,,Short event
Comment: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Comment to toggle
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Complex workflow: split, adjust timing, toggle type, add effects

    // 1. Split the first event
    doc.events().split(0, "0:00:03.00").unwrap();

    // Now we have 4 events: split first, original second, original comment

    // 2. Shift all events forward by 1 second
    doc.events()
        .timing()
        .shift(100) // 1 second
        .unwrap();

    // 3. Toggle the comment (now at index 3) to dialogue
    doc.events().toggle_type().event(3).apply().unwrap();

    // 4. Add fade effect to all events
    doc.events().effects().set("Fade(255,0)").unwrap();

    let content = doc.text();

    // Verify results
    let event_lines: Vec<&str> = content
        .lines()
        .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
        .collect();

    // Should have 4 events, all Dialogue (comment was toggled)
    assert_eq!(event_lines.len(), 4);
    assert!(event_lines.iter().all(|line| line.starts_with("Dialogue:")));

    // All should have timing shifted by 1 second
    assert!(content.contains("0:00:02.00,0:00:04.00")); // First part of split
    assert!(content.contains("0:00:04.00,0:00:06.00")); // Second part of split
    assert!(content.contains("0:00:06.00,0:00:08.00")); // Original second event
    assert!(content.contains("0:00:11.00,0:00:16.00")); // Original comment (now dialogue)

    // All should have fade effect
    assert!(event_lines.iter().all(|line| line.contains("Fade(255,0)")));
}
