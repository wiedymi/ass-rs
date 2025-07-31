//! Integration tests for the fluent API
//!
//! Tests the ergonomic doc.at(pos).insert_text() pattern
//! and position conversion utilities.

#![cfg(all(test, feature = "rope"))]

use ass_editor::core::{EditorDocument, Position, PositionBuilder, Range};

#[test]
fn test_fluent_insert_operations() {
    let mut doc = EditorDocument::new();
    
    // Test chaining inserts
    doc.at_start().insert_text("Hello").unwrap();
    doc.at_end().insert_text(", World!").unwrap();
    
    assert_eq!(doc.text(), "Hello, World!");
    
    // Test insert at specific position
    doc.at_pos(Position::new(5)).insert_text(" there").unwrap();
    assert_eq!(doc.text(), "Hello there, World!");
}

#[test]
fn test_fluent_line_operations() {
    let mut doc = EditorDocument::from_content("Line 1\nLine 2\nLine 3").unwrap();
    
    // Insert at start of line 2
    doc.at_line(2).unwrap().insert_text("Start: ").unwrap();
    assert_eq!(doc.text(), "Line 1\nStart: Line 2\nLine 3");
    
    // Replace to end of line
    let pos = doc.line_column_to_position(2, 8).unwrap(); // After "Start: "
    doc.at_pos(pos).replace_to_line_end("Modified Line 2").unwrap();
    assert_eq!(doc.text(), "Line 1\nStart: Modified Line 2\nLine 3");
    
    // Insert line break
    doc.at_line(3).unwrap().insert_line().unwrap();
    assert!(doc.text().contains("Line 1\nStart: Modified Line 2\n\nLine 3"));
}

#[test]
fn test_fluent_delete_operations() {
    let mut doc = EditorDocument::from_content("Hello World!").unwrap();
    
    // Delete forward
    doc.at_pos(Position::new(5)).delete(1).unwrap(); // Delete space
    assert_eq!(doc.text(), "HelloWorld!");
    
    // Backspace
    doc.at_pos(Position::new(5)).backspace(2).unwrap(); // Delete "lo"
    assert_eq!(doc.text(), "HelWorld!");
}

#[test]
fn test_fluent_selection_operations() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();
    
    // Replace selection
    let range = Range::new(Position::new(6), Position::new(11));
    doc.select(range).replace_with("Rust").unwrap();
    assert_eq!(doc.text(), "Hello Rust");
    
    // Wrap with tags
    let range = Range::new(Position::new(6), Position::new(10));
    doc.select(range).wrap_with_tag("{\\b1}", "{\\b0}").unwrap();
    assert_eq!(doc.text(), "Hello {\\b1}Rust{\\b0}");
    
    // Get selected text - the wrapped text is now longer
    let range = Range::new(Position::new(6), Position::new(20)); // Adjusted for tag length
    let selected = doc.select(range).text();
    assert_eq!(selected, "{\\b1}Rust{\\b0}");
}

#[test]
fn test_fluent_indent_operations() {
    let mut doc = EditorDocument::from_content("Line 1\nLine 2\nLine 3").unwrap();
    
    // Indent all lines
    let range = Range::new(Position::start(), Position::new(doc.len()));
    doc.select(range).indent(4).unwrap();
    assert_eq!(doc.text(), "    Line 1\n    Line 2\n    Line 3");
    
    // Unindent line 2 only
    let line2_start = doc.line_column_to_position(2, 1).unwrap();
    let line2_end = doc.line_column_to_position(2, 11).unwrap(); // End of line 2 text
    let range = Range::new(line2_start, line2_end);
    doc.select(range).unindent(2).unwrap();
    assert_eq!(doc.text(), "    Line 1\n  Line 2\n    Line 3");
}

#[test]
fn test_position_conversions() {
    let doc = EditorDocument::from_content("Line 1\nLine 2\nLine 3").unwrap();
    
    // Test position to line/column
    let pos = Position::new(7); // Start of "Line 2"
    let (line, col) = doc.position_to_line_col(pos).unwrap();
    assert_eq!((line, col), (2, 1));
    
    // Test position at end of line
    let pos = Position::new(13); // After "Line 2"
    let (line, col) = doc.position_to_line_col(pos).unwrap();
    assert_eq!((line, col), (2, 7));
    
    // Test line/column to position
    let pos = doc.line_column_to_position(3, 1).unwrap();
    assert_eq!(pos.offset, 14); // Start of "Line 3"
    
    // Test middle of line
    let pos = doc.line_column_to_position(2, 6).unwrap();
    assert_eq!(pos.offset, 12); // "Line |2"
}

#[test]
fn test_position_builder_advanced() {
    let rope = ropey::Rope::from_str("First line\nSecond line\nThird line");
    
    // Build at line start
    let pos = PositionBuilder::new()
        .at_line_start(2)
        .build(&rope)
        .unwrap();
    assert_eq!(pos.offset, 11); // After "First line\n"
    
    // Build at line end
    let pos = PositionBuilder::new()
        .at_line_end(1)
        .build(&rope)
        .unwrap();
    assert_eq!(pos.offset, 10); // Before newline
    
    // Build at document start
    let pos = PositionBuilder::at_start()
        .build(&rope)
        .unwrap();
    assert_eq!(pos.offset, 0);
}

#[test]
fn test_fluent_api_error_handling() {
    let mut doc = EditorDocument::from_content("Short").unwrap();
    
    // Test out of bounds position
    let result = doc.at_pos(Position::new(100)).insert_text("text");
    assert!(result.is_err());
    
    // Test invalid line
    let result = doc.at_line(10);
    assert!(result.is_err());
    
    // Test invalid range
    let range = Range::new(Position::new(10), Position::new(20));
    let result = doc.select(range).delete();
    assert!(result.is_err());
}

#[test]
fn test_fluent_api_with_ass_content() {
    let mut doc = EditorDocument::from_content(
        "[Script Info]\nTitle: Test\n\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello"
    ).unwrap();
    
    // Insert style section
    let events_pos = doc.text().find("[Events]").unwrap();
    doc.at_pos(Position::new(events_pos))
        .insert_text("[V4+ Styles]\nFormat: Name, Fontname\nStyle: Default,Arial,20\n\n")
        .unwrap();
    
    assert!(doc.text().contains("[V4+ Styles]"));
    
    // Wrap dialogue text with effect
    let hello_pos = doc.text().find("Hello").unwrap();
    let range = Range::new(Position::new(hello_pos), Position::new(hello_pos + 5));
    doc.select(range).wrap_with_tag("{\\fad(200,200)}", "").unwrap();
    
    assert!(doc.text().contains("{\\fad(200,200)}Hello"));
}