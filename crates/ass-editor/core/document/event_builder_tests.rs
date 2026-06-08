//! Tests for builder-driven event editing via `edit_event_with_builder`

use super::*;

#[test]
fn test_edit_event_with_builder() {
    let content = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,Speaker,0,0,0,,Original text
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Second event"#;

    let mut doc = EditorDocument::from_content(content).unwrap();

    // Edit the first event using builder
    let result = doc.edit_event_with_builder(0, |builder| {
        builder
            .text("Modified with builder")
            .style("NewStyle")
            .end_time("0:00:08.00")
            .speaker("NewSpeaker")
    });

    assert!(result.is_ok());
    let new_line = result.unwrap();
    assert!(new_line.contains("Modified with builder"));
    assert!(new_line.contains("NewStyle"));
    assert!(new_line.contains("0:00:08.00"));
    assert!(new_line.contains("NewSpeaker"));

    // Verify the document was updated
    let updated_content = doc.text();
    assert!(updated_content.contains("Modified with builder"));
    assert!(updated_content.contains("0:00:08.00"));
    assert!(!updated_content.contains("Original text"));
}

#[test]
fn test_edit_event_with_builder_preserves_format() {
    // Test with V4++ format that includes MarginT and MarginB
    let content = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginT, MarginB, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,10,20,5,15,fade,Original text"#;

    let mut doc = EditorDocument::from_content(content).unwrap();

    // Edit preserving the V4++ format
    let result = doc.edit_event_with_builder(0, |builder| {
        builder.text("New text").margin_top(30).margin_bottom(40)
    });

    assert!(result.is_ok());
    let new_line = result.unwrap();

    // Should use MarginT and MarginB fields based on format line
    assert!(new_line.contains("30")); // margin_top
    assert!(new_line.contains("40")); // margin_bottom
    assert!(new_line.contains("New text"));
    assert!(new_line.contains("10,20,30,40")); // All margins in correct order
}

#[test]
fn test_edit_event_with_builder_comment() {
    let content = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Comment: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,This is a comment"#;

    let mut doc = EditorDocument::from_content(content).unwrap();

    // Edit comment event
    let result = doc.edit_event_with_builder(0, |builder| builder.text("Updated comment"));

    assert!(result.is_ok());
    let new_line = result.unwrap();
    assert!(new_line.starts_with("Comment:"));
    assert!(new_line.contains("Updated comment"));
}
