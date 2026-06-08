//! Tests for style management commands.

use super::*;
use crate::commands::EditorCommand;
use crate::core::{EditorDocument, StyleBuilder};
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
const TEST_CONTENT: &str = r#"[Script Info]
Title: Test Script

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,Hello world!
"#;

#[test]
fn test_create_style_command() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let style_builder = StyleBuilder::new()
        .font("Comic Sans MS")
        .size(24)
        .bold(true);

    let command = CreateStyleCommand::new("NewStyle".to_string(), style_builder);
    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);
    assert!(doc.text().contains("Style: NewStyle"));
    assert!(doc.text().contains("Comic Sans MS"));
}

#[test]
fn test_edit_style_command() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = EditStyleCommand::new("Default".to_string())
        .set_font("Helvetica")
        .set_size(24)
        .set_bold(true);

    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);
    assert!(doc.text().contains("Helvetica"));
    assert!(doc.text().contains("24"));
    assert!(doc.text().contains("-1")); // Bold = true
}

#[test]
fn test_delete_style_command() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = DeleteStyleCommand::new("Default".to_string());
    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);
    assert!(!doc.text().contains("Style: Default"));
}

#[test]
fn test_clone_style_command() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = CloneStyleCommand::new("Default".to_string(), "DefaultCopy".to_string());
    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);
    assert!(doc.text().contains("Style: Default")); // Original should still exist
    assert!(doc.text().contains("Style: DefaultCopy")); // Clone should exist
}

#[test]
fn test_apply_style_command() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // First create a new style to apply
    let create_cmd = CreateStyleCommand::new(
        "NewStyle".to_string(),
        StyleBuilder::new().font("Verdana").size(18),
    );
    create_cmd.execute(&mut doc).unwrap();

    // Now apply the new style to events
    let command = ApplyStyleCommand::new("Default".to_string(), "NewStyle".to_string());
    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);
    assert!(doc.text().contains("NewStyle")); // Event should now use NewStyle
}

#[test]
fn test_apply_style_with_filter() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // Create a new style
    let create_cmd = CreateStyleCommand::new(
        "FilteredStyle".to_string(),
        StyleBuilder::new().font("Times").size(22),
    );
    create_cmd.execute(&mut doc).unwrap();

    // Apply style only to events containing "Hello"
    let command = ApplyStyleCommand::new("Default".to_string(), "FilteredStyle".to_string())
        .with_filter("Hello".to_string());

    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);
    assert!(doc.text().contains("FilteredStyle"));
}

#[test]
fn test_edit_nonexistent_style() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = EditStyleCommand::new("NonExistent".to_string()).set_font("Arial");

    let result = command.execute(&mut doc);
    assert!(result.is_err());
}

#[test]
fn test_clone_to_existing_style() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = CloneStyleCommand::new("Default".to_string(), "Default".to_string());
    let result = command.execute(&mut doc);

    assert!(result.is_err());
}
