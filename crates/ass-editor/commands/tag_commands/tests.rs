//! Tests for ASS override tag commands.

use super::*;
use crate::commands::EditorCommand;
use crate::core::{EditorDocument, Position, Range};
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn insert_tag_basic() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();
    let command = InsertTagCommand::new(Position::new(5), "\\b1".to_string());

    let result = command.execute(&mut doc).unwrap();
    assert!(result.success);
    assert_eq!(doc.text(), "Hello{\\b1} World");
}

#[test]
fn insert_tag_no_auto_wrap() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();
    let command = InsertTagCommand::new(Position::new(5), "\\b1".to_string()).no_auto_wrap();

    let result = command.execute(&mut doc).unwrap();
    assert!(result.success);
    assert_eq!(doc.text(), "Hello\\b1 World");
}

#[test]
fn remove_tag_all() {
    let mut doc = EditorDocument::from_content("Hello {\\b1}World{\\i1} Test").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));
    let command = RemoveTagCommand::new(range);

    let result = command.execute(&mut doc).unwrap();
    assert!(result.success);
    assert_eq!(doc.text(), "Hello World Test");
}

#[test]
fn remove_tag_specific_pattern() {
    let mut doc = EditorDocument::from_content("Hello {\\b1\\i1}World").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));
    let command = RemoveTagCommand::new(range).pattern("\\b".to_string());

    let result = command.execute(&mut doc).unwrap();
    assert!(result.success);
    assert_eq!(doc.text(), "Hello {\\i1}World");
}

#[test]
fn replace_tag() {
    let mut doc = EditorDocument::from_content("Hello {\\b1}World{\\b1} Test").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));
    let command = ReplaceTagCommand::new(range, "\\b1".to_string(), "\\b0".to_string()).all();

    let result = command.execute(&mut doc).unwrap();
    assert!(result.success);
    assert_eq!(doc.text(), "Hello {\\b0}World{\\b0} Test");
}

#[test]
fn wrap_tag() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();
    let range = Range::new(Position::new(6), Position::new(11));
    let command = WrapTagCommand::new(range, "\\b1".to_string());

    let result = command.execute(&mut doc).unwrap();
    assert!(result.success);
    assert_eq!(doc.text(), "Hello {\\b1}World{\\b0}");
}

#[test]
fn parse_tags() {
    let mut doc =
        EditorDocument::from_content("Hello {\\b1\\c&H00FF00&\\pos(100,200)}World").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));
    let command = ParseTagCommand::new(range).with_positions();

    let result = command.execute(&mut doc).unwrap();
    assert!(result.success);

    // Test parsing functionality directly
    let text = doc.text();
    let parsed = command.parse_tags_from_text(&text).unwrap();

    assert_eq!(parsed.len(), 3);
    assert_eq!(parsed[0].tag, "\\b1");
    assert_eq!(parsed[1].tag, "\\c&H00FF00&");
    assert_eq!(parsed[2].tag, "\\pos");
    assert_eq!(parsed[2].parameters, vec!["100", "200"]);
}

#[test]
fn tag_validation() {
    let mut doc = EditorDocument::new();

    // Test invalid tag (no backslash)
    let command = InsertTagCommand::new(Position::new(0), "b1".to_string());
    let result = command.execute(&mut doc);
    assert!(result.is_err());
}
