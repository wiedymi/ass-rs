//! Tests for the streaming [`EventsParser`] parse loop and event handling.

use super::*;
use crate::parser::ast::{EventType, Section};
#[cfg(not(feature = "std"))]
use alloc::format;

#[test]
fn parse_empty_section() {
    let parser = EventsParser::new("", 0, 1);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, ..) = result.unwrap();
    if let Section::Events(events) = section {
        assert!(events.is_empty());
    } else {
        panic!("Expected Events section");
    }
}

#[test]
fn parse_with_format_and_dialogue() {
    let content = "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World!\n";
    let parser = EventsParser::new(content, 0, 1);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, ..) = result.unwrap();
    if let Section::Events(events) = section {
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert!(matches!(event.event_type, EventType::Dialogue));
        assert_eq!(event.start, "0:00:00.00");
        assert_eq!(event.end, "0:00:05.00");
        assert_eq!(event.style, "Default");
        assert_eq!(event.text, "Hello World!");
        // Check span
        assert!(event.span.start > 0);
        assert!(event.span.end > event.span.start);
    } else {
        panic!("Expected Events section");
    }
}

#[test]
fn parse_different_event_types() {
    let content = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Dialogue\nComment: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Comment\n";
    let parser = EventsParser::new(content, 0, 1);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, ..) = result.unwrap();
    if let Section::Events(events) = section {
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0].event_type, EventType::Dialogue));
        assert!(matches!(events[1].event_type, EventType::Comment));
        assert_eq!(events[0].text, "Dialogue");
        assert_eq!(events[1].text, "Comment");
    } else {
        panic!("Expected Events section");
    }
}

#[test]
fn handle_text_with_commas() {
    let content = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello, world, with commas!\n";
    let parser = EventsParser::new(content, 0, 1);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, ..) = result.unwrap();
    if let Section::Events(events) = section {
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].text, "Hello, world, with commas!");
    } else {
        panic!("Expected Events section");
    }
}

#[test]
fn skip_comments_and_whitespace() {
    let content =
        "; Comment\n# Another comment\n\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test\n";
    let parser = EventsParser::new(content, 0, 1);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, ..) = result.unwrap();
    if let Section::Events(events) = section {
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].text, "Test");
    } else {
        panic!("Expected Events section");
    }
}

#[test]
fn parse_with_position_tracking() {
    // Create a larger content that simulates a full file
    let prefix = "a".repeat(200); // 200 bytes of padding
    let section_content = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test Event\n";
    let full_content = format!("{prefix}{section_content}");

    // Parser starts at position 200
    let parser = EventsParser::new(&full_content, 200, 15);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, _, _, final_pos, final_line) = result.unwrap();
    if let Section::Events(events) = section {
        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.span.start, 200);
        assert_eq!(event.span.line, 15);
        assert_eq!(event.text, "Test Event");
    } else {
        panic!("Expected Events section");
    }

    assert_eq!(final_pos, 200 + section_content.len());
    assert_eq!(final_line, 16);
}

#[test]
fn parse_without_format_line() {
    let content = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,No format line\n";
    let parser = EventsParser::new(content, 0, 1);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, format, ..) = result.unwrap();
    if let Section::Events(events) = section {
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].text, "No format line");
    } else {
        panic!("Expected Events section");
    }
    assert!(format.is_none());
}
