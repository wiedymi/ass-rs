//! Tests for event effect modification commands.

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
fn test_event_effect_command() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    let command = EventEffectCommand::set_effect(vec![0, 1], "Fade(255,0)".to_string());
    let result = command.execute(&mut doc).unwrap();

    assert!(result.success);
    assert!(result.content_changed);

    // Check that effects were set for first two events
    let content = doc.text();
    let lines: Vec<&str> = content.lines().collect();
    let event_lines: Vec<&str> = lines
        .iter()
        .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
        .copied()
        .collect();

    assert!(event_lines[0].contains("Fade(255,0)"));
    assert!(event_lines[1].contains("Fade(255,0)"));
    assert!(!event_lines[2].contains("Fade(255,0)")); // Third event unchanged
}

#[test]
fn test_effect_operations() {
    let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

    // First set an effect
    let set_cmd = EventEffectCommand::set_effect(vec![0], "Fade(255,0)".to_string());
    set_cmd.execute(&mut doc).unwrap();

    // Then append to it
    let append_cmd = EventEffectCommand::append_effect(vec![0], "Move(100,200)".to_string());
    append_cmd.execute(&mut doc).unwrap();

    // Check that both effects are present
    // println!("Document after append: {}", doc.text());
    assert!(doc.text().contains("Fade(255,0) Move(100,200)"));

    // Clear the effect
    let clear_cmd = EventEffectCommand::clear_effect(vec![0]);
    clear_cmd.execute(&mut doc).unwrap();

    // Check that effect field is empty (has the right number of commas)
    let text = doc.text();
    let lines: Vec<&str> = text.lines().collect();
    let first_event = lines
        .iter()
        .find(|line| line.starts_with("Dialogue:"))
        .unwrap();
    let parts: Vec<&str> = first_event.split(',').collect();
    assert_eq!(parts[8].trim(), ""); // Effect field should be empty
}
