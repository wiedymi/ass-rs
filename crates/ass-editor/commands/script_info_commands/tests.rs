//! Tests for the `[Script Info]` management commands.

use super::*;
use crate::commands::EditorCommand;
use crate::core::EditorDocument;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

const TEST_CONTENT: &str = r#"[Script Info]
Title: Test Subtitle
ScriptType: v4.00+
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#;

#[test]
fn test_set_existing_property() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = SetScriptInfoCommand::new("Title".to_string(), "New Title".to_string());
    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);
    assert!(doc.text().contains("Title: New Title"));
    assert!(!doc.text().contains("Title: Test Subtitle"));
}

#[test]
fn test_set_new_property() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = SetScriptInfoCommand::new("Author".to_string(), "John Doe".to_string());
    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);
    assert!(doc.text().contains("Author: John Doe"));
}

#[test]
fn test_get_existing_property() {
    let doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = GetScriptInfoCommand::new("Title".to_string());
    let value = command.get_value(&doc).unwrap();

    assert_eq!(value, Some("Test Subtitle".to_string()));
}

#[test]
fn test_get_nonexistent_property() {
    let doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = GetScriptInfoCommand::new("Author".to_string());
    let value = command.get_value(&doc).unwrap();

    assert_eq!(value, None);
}

#[test]
fn test_delete_property() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = DeleteScriptInfoCommand::new("PlayResX".to_string());
    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);
    assert!(!doc.text().contains("PlayResX: 1920"));
    assert!(doc.text().contains("PlayResY: 1080")); // Other properties remain
}

#[test]
fn test_get_all_properties() {
    let doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = GetAllScriptInfoCommand::new();
    let properties = command.get_all(&doc).unwrap();

    assert_eq!(properties.len(), 4);
    assert!(properties.contains(&("Title".to_string(), "Test Subtitle".to_string())));
    assert!(properties.contains(&("ScriptType".to_string(), "v4.00+".to_string())));
    assert!(properties.contains(&("PlayResX".to_string(), "1920".to_string())));
    assert!(properties.contains(&("PlayResY".to_string(), "1080".to_string())));
}
