//! Tests for the direct event access, filtering, and sorting query API.

use crate::core::EditorDocument;
use ass_core::parser::ast::EventType;

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec};

#[test]
fn test_new_event_api_direct_access() {
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

    // Test direct event access
    let event_info = doc.events().get(0).unwrap();
    assert!(event_info.is_some());
    let info = event_info.unwrap();
    assert_eq!(info.index, 0);
    assert_eq!(info.event.text, "First event");
    assert_eq!(info.event.event_type, EventType::Dialogue);

    // Test event count
    let count = doc.events().count().unwrap();
    assert_eq!(count, 3);

    // Test fluent accessor
    let text = doc.events().event(1).text().unwrap();
    assert_eq!(text, Some("Second event".to_string()));

    let style = doc.events().event(1).style().unwrap();
    assert_eq!(style, Some("Default".to_string()));

    let exists = doc.events().event(5).exists().unwrap();
    assert!(!exists);
}

#[test]
fn test_new_event_api_filtering() {
    const TEST_CONTENT: &str = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First dialogue
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second dialogue
Comment: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,First comment
Comment: 0,0:00:15.00,0:00:20.00,Default,Speaker,0,0,0,,Second comment
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Test filtering by type
    let dialogues = doc.events().dialogues().execute().unwrap();
    assert_eq!(dialogues.len(), 2);
    assert!(dialogues
        .iter()
        .all(|info| info.event.event_type == EventType::Dialogue));

    let comments = doc.events().comments().execute().unwrap();
    assert_eq!(comments.len(), 2);
    assert!(comments
        .iter()
        .all(|info| info.event.event_type == EventType::Comment));

    // Test text filtering
    let with_first = doc
        .events()
        .query()
        .filter_by_text("First")
        .execute()
        .unwrap();
    assert_eq!(with_first.len(), 2);
    assert!(with_first[0].event.text.contains("First"));
    assert!(with_first[1].event.text.contains("First"));

    // Test case insensitive filtering
    let with_first_insensitive = doc
        .events()
        .query()
        .filter_by_text("first")
        .case_sensitive(false)
        .execute()
        .unwrap();
    assert_eq!(with_first_insensitive.len(), 2);
}

#[test]
fn test_new_event_api_sorting() {
    const TEST_CONTENT: &str = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Third by time
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First by time
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second by time
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Test sorting by time (should reorder events)
    let by_time = doc.events().by_time().execute().unwrap();
    assert_eq!(by_time.len(), 3);
    assert_eq!(by_time[0].event.text, "First by time");
    assert_eq!(by_time[1].event.text, "Second by time");
    assert_eq!(by_time[2].event.text, "Third by time");

    // Test original order
    let in_order = doc.events().in_order().execute().unwrap();
    assert_eq!(in_order.len(), 3);
    assert_eq!(in_order[0].event.text, "Third by time");
    assert_eq!(in_order[1].event.text, "First by time");
    assert_eq!(in_order[2].event.text, "Second by time");

    // Test descending sort
    let by_time_desc = doc
        .events()
        .query()
        .sort_by_time()
        .descending()
        .execute()
        .unwrap();
    assert_eq!(by_time_desc[0].event.text, "Third by time");
    assert_eq!(by_time_desc[1].event.text, "Second by time");
    assert_eq!(by_time_desc[2].event.text, "First by time");
}

#[test]
fn test_new_event_api_combined_operations() {
    const TEST_CONTENT: &str = r#"[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Important dialogue
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,Another dialogue
Comment: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Important comment
Dialogue: 0,0:00:15.00,0:00:20.00,Default,Speaker,0,0,0,,Final dialogue
"#;

    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Test combined filtering and sorting with limit
    let important_dialogues = doc
        .events()
        .query()
        .filter_by_type(EventType::Dialogue)
        .filter_by_text("Important")
        .sort_by_time()
        .limit(1)
        .execute()
        .unwrap();

    assert_eq!(important_dialogues.len(), 1);
    assert_eq!(important_dialogues[0].event.text, "Important dialogue");
    assert_eq!(important_dialogues[0].event.event_type, EventType::Dialogue);

    // Test getting indices only
    let dialogue_indices = doc.events().dialogues().sort_by_time().indices().unwrap();

    assert_eq!(dialogue_indices.len(), 3);
    // Should be indices in time order: 1, 0, 3 (based on start times)
    assert_eq!(dialogue_indices, vec![1, 0, 3]);

    // Test count
    let dialogue_count = doc.events().dialogues().count().unwrap();
    assert_eq!(dialogue_count, 3);

    // Test first
    let first_dialogue = doc.events().dialogues().sort_by_time().first().unwrap();

    assert!(first_dialogue.is_some());
    let first = first_dialogue.unwrap();
    assert_eq!(first.event.text, "Another dialogue");
}
