//! Tests for document creation, basic mutation, and position conversion

use super::*;
use crate::core::position::{Position, Range};

#[test]
fn document_creation() {
    let doc = EditorDocument::new();
    assert!(doc.is_empty());
    assert_eq!(doc.len_lines(), 1);
    assert!(!doc.is_modified());
}

#[test]
fn document_from_content() {
    let content = "[Script Info]\nTitle: Test";
    let doc = EditorDocument::from_content(content).unwrap();
    assert_eq!(doc.text(), content);
    assert_eq!(doc.len_bytes(), content.len());
}

#[test]
fn document_modification() {
    let mut doc = EditorDocument::new();
    doc.insert(Position::new(0), "Hello").unwrap();
    assert!(doc.is_modified());
    assert_eq!(doc.text(), "Hello");
}

#[test]
fn position_conversion() {
    let content = "Line 1\nLine 2\nLine 3";
    let doc = EditorDocument::from_content(content).unwrap();

    // Start of second line
    let pos = Position::new(7); // After "Line 1\n"
    let lc = doc.position_to_line_column(pos).unwrap();
    assert_eq!(lc.line, 2);
    assert_eq!(lc.column, 1);
}

#[test]
fn range_operations() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();

    // Delete "World"
    let range = Range::new(Position::new(6), Position::new(11));
    doc.delete(range).unwrap();
    assert_eq!(doc.text(), "Hello ");

    // Replace with "Rust"
    doc.insert(Position::new(6), "Rust").unwrap();
    assert_eq!(doc.text(), "Hello Rust");
}

#[test]
fn parse_script_test() {
    let content = "[Script Info]\nTitle: Test\n[Events]\nDialogue: test";
    let doc = EditorDocument::from_content(content).unwrap();

    // Validate should succeed
    doc.validate().unwrap();

    // Parse and use script
    let sections_count = doc.sections_count().unwrap();
    assert!(sections_count > 0);
}
