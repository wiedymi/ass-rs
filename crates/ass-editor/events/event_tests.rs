//! Unit tests for `DocumentEvent` queries and `EventFilter` behavior.

use super::*;
use crate::core::Position;
#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec};

#[test]
fn document_event_creation() {
    let event = DocumentEvent::text_inserted(Position::new(0), "Hello".to_string());

    match event {
        DocumentEvent::TextInserted {
            position,
            text,
            length,
        } => {
            assert_eq!(position.offset, 0);
            assert_eq!(text, "Hello");
            assert_eq!(length, 5);
        }
        _ => panic!("Expected TextInserted event"),
    }
}

#[test]
fn document_event_description() {
    let event = DocumentEvent::text_inserted(Position::new(0), "Hello".to_string());
    assert_eq!(event.description(), "Inserted 5 bytes of text");

    let event = DocumentEvent::cursor_moved(Position::new(0), Position::new(5));
    assert_eq!(event.description(), "Cursor moved");
}

#[test]
fn document_event_modification_check() {
    let insert_event = DocumentEvent::text_inserted(Position::new(0), "Hello".to_string());
    assert!(insert_event.is_modification());

    let cursor_event = DocumentEvent::cursor_moved(Position::new(0), Position::new(5));
    assert!(!cursor_event.is_modification());
}

#[test]
fn document_event_affects_text() {
    let insert_event = DocumentEvent::text_inserted(Position::new(0), "Hello".to_string());
    assert!(insert_event.affects_text());

    let config_event = DocumentEvent::ConfigChanged {
        key: "font_size".to_string(),
        old_value: Some("12".to_string()),
        new_value: "14".to_string(),
    };
    assert!(!config_event.affects_text());
}

#[test]
fn document_event_affected_range() {
    let insert_event = DocumentEvent::text_inserted(Position::new(10), "Hello".to_string());
    let range = insert_event.affected_range().unwrap();
    assert_eq!(range.start.offset, 10);
    assert_eq!(range.end.offset, 15);

    let cursor_event = DocumentEvent::cursor_moved(Position::new(0), Position::new(5));
    assert!(cursor_event.affected_range().is_none());
}

#[test]
fn event_filter_creation() {
    let filter = EventFilter::new()
        .include_modifications(true)
        .exclude_types(vec!["CursorMoved".to_string()]);

    let insert_event = DocumentEvent::text_inserted(Position::new(0), "Hello".to_string());
    assert!(filter.matches(&insert_event));

    let cursor_event = DocumentEvent::cursor_moved(Position::new(0), Position::new(5));
    assert!(!filter.matches(&cursor_event));
}

#[test]
fn event_filter_include_types() {
    let filter = EventFilter::new()
        .include_types(vec!["TextInserted".to_string(), "TextDeleted".to_string()]);

    let insert_event = DocumentEvent::text_inserted(Position::new(0), "Hello".to_string());
    assert!(filter.matches(&insert_event));

    let cursor_event = DocumentEvent::cursor_moved(Position::new(0), Position::new(5));
    assert!(!filter.matches(&cursor_event));
}
