//! Tests for the static [`EventsParser::parse_event_line`] entry point.

use super::*;
use crate::parser::ast::EventType;
use crate::parser::errors::ParseError;
#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn test_public_parse_event_line() {
    let format = vec![
        "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV", "Effect", "Text",
    ];
    let line = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test text";

    let result = EventsParser::parse_event_line(line, &format, 1);
    assert!(result.is_ok());

    let event = result.unwrap();
    assert!(matches!(event.event_type, EventType::Dialogue));
    assert_eq!(event.start, "0:00:00.00");
    assert_eq!(event.end, "0:00:05.00");
    assert_eq!(event.style, "Default");
    assert_eq!(event.text, "Test text");
}

#[test]
fn test_parse_event_line_invalid_type() {
    let format = vec!["Layer", "Start", "End", "Style", "Text"];
    let line = "Invalid: 0,0:00:00.00,0:00:05.00,Default,Test";

    let result = EventsParser::parse_event_line(line, &format, 1);
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(matches!(e, ParseError::InvalidEventType { .. }));
    }
}

#[test]
fn test_parse_event_line_insufficient_fields() {
    let format = vec![
        "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV", "Effect", "Text",
    ];
    let line = "Dialogue: 0,0:00:00.00,0:00:05.00"; // Missing fields

    let result = EventsParser::parse_event_line(line, &format, 1);
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(matches!(e, ParseError::InsufficientFields { .. }));
    }
}

#[test]
fn test_parse_event_line_with_commas_in_text() {
    let format = vec![
        "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR", "MarginV", "Effect", "Text",
    ];
    let line = "Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello, world, with commas!";

    let result = EventsParser::parse_event_line(line, &format, 1);
    assert!(result.is_ok());

    let event = result.unwrap();
    assert_eq!(event.text, "Hello, world, with commas!");
}
