//! Tests for delta-aware incremental editing commands

use super::*;
use crate::core::Position;
use crate::EditorDocument;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
#[test]
fn test_incremental_insert_command() {
    let mut doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();

    let command = IncrementalInsertCommand::new(
        Position::new(doc.len_bytes()),
        "\nAuthor: Test Author".to_string(),
    );

    let result = command.execute_with_delta(&mut doc).unwrap();
    assert!(result.success);
    assert!(result.content_changed);
    assert!(doc.text().contains("Author: Test Author"));
}

#[test]
fn test_incremental_event_edit_command() {
    let content = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:05.00,0:00:10.00,Default,John,0,0,0,,Hello, world!"#;

    let mut doc = EditorDocument::from_content(content).unwrap();

    let command =
        IncrementalEventEditCommand::new("Hello, world!".to_string(), "Hello, ASS-RS!".to_string());

    let result = command.execute_with_delta(&mut doc).unwrap();
    assert!(result.success);
    assert!(doc.text().contains("Hello, ASS-RS!"));
}

#[test]
fn test_delta_batch_command() {
    let mut doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();

    // Simple single command batch to test the infrastructure without complex position calculations
    let batch = DeltaBatchCommand::new().add_command(IncrementalInsertCommand::new(
        Position::new(doc.len_bytes()),
        "\nAuthor: Test".to_string(),
    ));

    let results = batch.execute_batch(&mut doc).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results.iter().all(|r| r.success));
    assert!(doc.text().contains("Author: Test"));
}
