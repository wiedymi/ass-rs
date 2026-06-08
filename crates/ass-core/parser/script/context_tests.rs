//! Tests for format preservation, context-aware parsing, and line updates.

use super::*;
use crate::parser::ast::{Section, SectionType};

#[test]
fn format_preservation() {
    let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, Bold\nStyle: Default,Arial,20,1\n\n[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,Hello";

    let script = Script::parse(content).unwrap();

    // Check that format fields are preserved
    let styles_format = script.styles_format().unwrap();
    assert_eq!(styles_format.len(), 4);
    assert_eq!(styles_format[0], "Name");
    assert_eq!(styles_format[1], "Fontname");
    assert_eq!(styles_format[2], "Fontsize");
    assert_eq!(styles_format[3], "Bold");

    let events_format = script.events_format().unwrap();
    assert_eq!(events_format.len(), 5);
    assert_eq!(events_format[0], "Layer");
    assert_eq!(events_format[1], "Start");
    assert_eq!(events_format[2], "End");
    assert_eq!(events_format[3], "Style");
    assert_eq!(events_format[4], "Text");
}

#[test]
fn context_aware_style_parsing() {
    let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Bold\nStyle: Default,Arial,1";
    let script = Script::parse(content).unwrap();

    // Test parsing a style line with custom format
    let style_line = "NewStyle,Times,0";
    let result = script.parse_style_line_with_context(style_line, 10);
    assert!(result.is_ok());

    let style = result.unwrap();
    assert_eq!(style.name, "NewStyle");
    assert_eq!(style.fontname, "Times");
    assert_eq!(style.bold, "0");
}

#[test]
fn context_aware_event_parsing() {
    let content = "[Script Info]\nTitle: Test\n\n[Events]\nFormat: Start, End, Text\nDialogue: 0:00:00.00,0:00:05.00,Hello";
    let script = Script::parse(content).unwrap();

    // Test parsing an event line with custom format
    let event_line = "Dialogue: 0:00:05.00,0:00:10.00,World";
    let result = script.parse_event_line_with_context(event_line, 10);
    assert!(result.is_ok());

    let event = result.unwrap();
    assert_eq!(event.start, "0:00:05.00");
    assert_eq!(event.end, "0:00:10.00");
    assert_eq!(event.text, "World");
}

#[test]
fn parse_line_auto_detection() {
    let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname\n\n[Events]\nFormat: Layer, Start, End, Style, Text";
    let script = Script::parse(content).unwrap();

    // Test style line detection
    let style_line = "Style: Default,Arial";
    let result = script.parse_line_auto(style_line, 10);
    assert!(result.is_ok());
    let (section_type, content) = result.unwrap();
    assert_eq!(section_type, SectionType::Styles);
    assert!(matches!(content, LineContent::Style(_)));

    // Test event line detection
    let event_line = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,Test";
    let result = script.parse_line_auto(event_line, 11);
    assert!(result.is_ok());
    let (section_type, content) = result.unwrap();
    assert_eq!(section_type, SectionType::Events);
    assert!(matches!(content, LineContent::Event(_)));

    // Test script info field detection
    let info_line = "PlayResX: 1920";
    let result = script.parse_line_auto(info_line, 12);
    assert!(result.is_ok());
    let (section_type, content) = result.unwrap();
    assert_eq!(section_type, SectionType::ScriptInfo);
    if let LineContent::Field(key, value) = content {
        assert_eq!(key, "PlayResX");
        assert_eq!(value, "1920");
    } else {
        panic!("Expected Field variant");
    }
}

#[test]
fn context_parsing_with_default_format() {
    // Test that context-aware parsing works even without explicit format
    let content = "[Script Info]\nTitle: Test";
    let script = Script::parse(content).unwrap();

    // Should use default format
    let style_line = "Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1";
    let result = script.parse_style_line_with_context(style_line, 10);
    assert!(result.is_ok());

    let event_line = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test";
    let result = script.parse_event_line_with_context(event_line, 11);
    assert!(result.is_ok());
}

#[test]
fn update_style_line() {
    let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize\nStyle: Default,Arial,20\nStyle: Alt,Times,18";
    let mut script = Script::parse(content).unwrap();

    // Find the offset of the Default style
    if let Some(Section::Styles(styles)) = script.find_section(SectionType::Styles) {
        let default_style = &styles[0];
        let offset = default_style.span.start;

        // Update the style
        let new_line = "Style: Default,Helvetica,24";
        let result = script.update_line_at_offset(offset, new_line, 10);
        assert!(result.is_ok());

        // Verify the update
        if let Ok(LineContent::Style(old_style)) = result {
            assert_eq!(old_style.name, "Default");
            assert_eq!(old_style.fontname, "Arial");
            assert_eq!(old_style.fontsize, "20");
        }

        // Check the new value
        if let Some(Section::Styles(updated_styles)) = script.find_section(SectionType::Styles) {
            assert_eq!(updated_styles[0].fontname, "Helvetica");
            assert_eq!(updated_styles[0].fontsize, "24");
        }
    }
}

#[test]
fn update_event_line() {
    let content = "[Script Info]\nTitle: Test\n\n[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,Hello\nDialogue: 0,0:00:05.00,0:00:10.00,Default,World";
    let mut script = Script::parse(content).unwrap();

    // Find the offset of the first event
    if let Some(Section::Events(events)) = script.find_section(SectionType::Events) {
        let first_event = &events[0];
        let offset = first_event.span.start;

        // Update the event
        let new_line = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,Updated Text";
        let result = script.update_line_at_offset(offset, new_line, 10);
        assert!(result.is_ok());

        // Verify the update
        if let Ok(LineContent::Event(old_event)) = result {
            assert_eq!(old_event.text, "Hello");
        }

        // Check the new value
        if let Some(Section::Events(updated_events)) = script.find_section(SectionType::Events) {
            assert_eq!(updated_events[0].text, "Updated Text");
        }
    }
}
