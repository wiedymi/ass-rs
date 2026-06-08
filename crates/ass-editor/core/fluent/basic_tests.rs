//! Tests for position-based, selection-based, and tag fluent operations.

use crate::core::{EditorDocument, Position, Range};

#[test]
#[cfg(feature = "rope")]
fn test_fluent_insert() {
    let mut doc = EditorDocument::new();
    doc.at_start().insert_text("Hello, ").unwrap();
    doc.at_end().insert_text("World!").unwrap();

    assert_eq!(doc.text(), "Hello, World!");
}

#[test]
#[cfg(feature = "rope")]
fn test_fluent_line_operations() {
    let mut doc = EditorDocument::from_content("Line 1\nLine 2\nLine 3").unwrap();

    // Insert at beginning of line 2
    doc.at_line(2).unwrap().insert_text("Start: ").unwrap();
    assert_eq!(doc.text(), "Line 1\nStart: Line 2\nLine 3");

    // Replace to end of line
    doc.at_line(2)
        .unwrap()
        .replace_to_line_end("New Line 2")
        .unwrap();
    assert_eq!(doc.text(), "Line 1\nNew Line 2\nLine 3");
}

#[test]
#[cfg(feature = "rope")]
fn test_fluent_selection() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();

    let range = Range::new(Position::new(6), Position::new(11));
    doc.select(range).replace_with("Rust").unwrap();
    assert_eq!(doc.text(), "Hello Rust");

    // Test wrapping
    let range = Range::new(Position::new(6), Position::new(10));
    doc.select(range).wrap_with_tag("{\\b1}", "{\\b0}").unwrap();
    assert_eq!(doc.text(), "Hello {\\b1}Rust{\\b0}");
}

#[test]
#[cfg(feature = "rope")]
fn test_position_conversion() {
    let doc = EditorDocument::from_content("Line 1\nLine 2\nLine 3").unwrap();

    // Test position to line/column
    let pos = Position::new(7); // Start of "Line 2"
    let (line, col) = doc.position_to_line_col(pos).unwrap();
    assert_eq!((line, col), (2, 1));

    // Test line/column to position
    let pos2 = doc.line_column_to_position(2, 1).unwrap();
    assert_eq!(pos2.offset, 7);
}

#[test]
#[cfg(feature = "rope")]
fn test_indent_unindent() {
    let mut doc = EditorDocument::from_content("Line 1\nLine 2\nLine 3").unwrap();

    // Select all and indent
    let range = Range::new(Position::start(), Position::new(doc.len()));
    doc.select(range).indent(2).unwrap();
    assert_eq!(doc.text(), "  Line 1\n  Line 2\n  Line 3");

    // Unindent
    let range = Range::new(Position::start(), Position::new(doc.len()));
    doc.select(range).unindent(2).unwrap();
    assert_eq!(doc.text(), "Line 1\nLine 2\nLine 3");
}

#[test]
fn tag_operations() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();

    // Test tag insertion
    doc.tags().at(Position::new(5)).insert("\\b1").unwrap();
    assert_eq!(doc.text(), "Hello{\\b1} World");

    // Test raw tag insertion - need to account for the inserted tag
    doc.tags().at(Position::new(12)).insert_raw("\\i1").unwrap();
    assert_eq!(doc.text(), "Hello{\\b1} W\\i1orld");
}

#[test]
fn tag_removal() {
    let mut doc = EditorDocument::from_content("Hello {\\b1\\i1}World{\\c&H00FF00&} test").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    // Remove specific pattern
    doc.tags().in_range(range).remove_pattern("\\b").unwrap();
    assert_eq!(doc.text(), "Hello {\\i1}World{\\c&H00FF00&} test");

    // Remove all tags
    let full_range = Range::new(Position::new(0), Position::new(doc.text().len()));
    doc.tags().in_range(full_range).remove_all().unwrap();
    assert_eq!(doc.text(), "Hello World test");
}

#[test]
fn tag_replacement() {
    let mut doc = EditorDocument::from_content("Hello {\\b1}World{\\b1} test").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    // Replace all bold tags with italic
    doc.tags()
        .in_range(range)
        .replace_all("\\b1", "\\i1")
        .unwrap();
    assert_eq!(doc.text(), "Hello {\\i1}World{\\i1} test");
}

#[test]
fn tag_wrapping() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();
    let range = Range::new(Position::new(6), Position::new(11));

    // Wrap with bold tags
    doc.tags().in_range(range).wrap("\\b1").unwrap();
    assert_eq!(doc.text(), "Hello {\\b1}World{\\b0}");

    // Test explicit closing tag
    let mut doc2 = EditorDocument::from_content("Hello World").unwrap();
    let range2 = Range::new(Position::new(6), Position::new(11));
    doc2.tags()
        .in_range(range2)
        .wrap_with("\\c&HFF0000&", "\\c")
        .unwrap();
    assert_eq!(doc2.text(), "Hello {\\c&HFF0000&}World{\\c}");
}

#[test]
fn tag_parsing() {
    let mut doc =
        EditorDocument::from_content("Hello {\\b1\\c&H00FF00&\\pos(100,200)}World").unwrap();
    let range = Range::new(Position::new(0), Position::new(doc.text().len()));

    let parsed_tags = doc.tags().in_range(range).parse().unwrap();

    assert_eq!(parsed_tags.len(), 3);
    assert_eq!(parsed_tags[0].tag, "\\b1");
    assert_eq!(parsed_tags[1].tag, "\\c&H00FF00&");
    assert_eq!(parsed_tags[2].tag, "\\pos");
    assert_eq!(parsed_tags[2].parameters.len(), 2);
    assert_eq!(parsed_tags[2].parameters[0], "100");
    assert_eq!(parsed_tags[2].parameters[1], "200");
}
