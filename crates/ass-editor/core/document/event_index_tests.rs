//! Tests for index-based event editing via `edit_event_by_index`

use super::*;
use crate::core::errors::EditorError;

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec};

#[test]
fn test_edit_event_by_index() {
    let content = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Second event
Dialogue: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,Third event"#;

    let mut doc = EditorDocument::from_content(content).unwrap();

    // Edit the second event (index 1)
    let result = doc.edit_event_by_index(1, |_event| {
        vec![
            ("text", "Modified second event".to_string()),
            ("style", "NewStyle".to_string()),
            ("start", "0:00:06.00".to_string()),
        ]
    });

    assert!(result.is_ok());
    let new_line = result.unwrap();
    assert!(new_line.contains("Modified second event"));
    assert!(new_line.contains("NewStyle"));
    assert!(new_line.contains("0:00:06.00"));

    // Verify the document was updated
    let updated_content = doc.text();
    assert!(updated_content.contains("Modified second event"));
    assert!(updated_content.contains("NewStyle"));
    assert!(updated_content.contains("0:00:06.00"));
    assert!(!updated_content.contains("Second event")); // Old text should be gone
}

#[test]
fn test_edit_event_by_index_out_of_bounds() {
    let content = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,First event"#;

    let mut doc = EditorDocument::from_content(content).unwrap();

    // Try to edit non-existent event
    let result =
        doc.edit_event_by_index(5, |_event| vec![("text", "This should fail".to_string())]);

    assert!(result.is_err());
    match result.err().unwrap() {
        EditorError::InvalidRange { start, end, length } => {
            assert_eq!(start, 5);
            assert_eq!(end, 6);
            assert_eq!(length, 1); // Only 1 event exists
        }
        _ => panic!("Expected InvalidRange error"),
    }
}

#[test]
fn test_edit_event_by_index_all_fields() {
    let content = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,Speaker,10,20,30,fade,Original text"#;

    let mut doc = EditorDocument::from_content(content).unwrap();

    // Edit all possible fields
    let result = doc.edit_event_by_index(0, |_event| {
        vec![
            ("layer", "1".to_string()),
            ("start", "0:00:01.00".to_string()),
            ("end", "0:00:06.00".to_string()),
            ("style", "Custom".to_string()),
            ("name", "NewSpeaker".to_string()),
            ("margin_l", "15".to_string()),
            ("margin_r", "25".to_string()),
            ("margin_v", "35".to_string()),
            ("effect", "scroll".to_string()),
            ("text", "Completely new text".to_string()),
        ]
    });

    assert!(result.is_ok());
    let new_line = result.unwrap();

    // Verify all fields were updated
    assert!(new_line.contains("Dialogue: 1,"));
    assert!(new_line.contains("0:00:01.00"));
    assert!(new_line.contains("0:00:06.00"));
    assert!(new_line.contains("Custom"));
    assert!(new_line.contains("NewSpeaker"));
    assert!(new_line.contains("15"));
    assert!(new_line.contains("25"));
    assert!(new_line.contains("35"));
    assert!(new_line.contains("scroll"));
    assert!(new_line.contains("Completely new text"));
}
