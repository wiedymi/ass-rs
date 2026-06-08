//! Unit tests for the core editor command implementations.

use super::*;
use crate::core::{EditorDocument, Position, Range};
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[cfg(not(feature = "std"))]
#[test]
fn insert_command_execution() {
    let mut doc = EditorDocument::new();
    let command = InsertTextCommand::new(Position::new(0), "Hello".to_string());

    let result = command.execute(&mut doc).unwrap();
    assert!(result.success);
    assert!(result.content_changed);
    assert_eq!(doc.text(), "Hello");
}

#[test]
fn delete_command_execution() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();
    let range = Range::new(Position::new(6), Position::new(11));
    let command = DeleteTextCommand::new(range);

    let result = command.execute(&mut doc).unwrap();
    assert!(result.success);
    assert!(result.content_changed);
    assert_eq!(doc.text(), "Hello ");
}

#[test]
fn replace_command_execution() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();
    let range = Range::new(Position::new(6), Position::new(11));
    let command = ReplaceTextCommand::new(range, "Rust".to_string());

    let result = command.execute(&mut doc).unwrap();
    assert!(result.success);
    assert!(result.content_changed);
    assert_eq!(doc.text(), "Hello Rust");
}

#[test]
fn batch_command_execution() {
    let mut doc = EditorDocument::from_content("Hello").unwrap();

    let batch = BatchCommand::new("Insert and replace".to_string())
        .add_command(Box::new(InsertTextCommand::new(
            Position::new(5),
            " World".to_string(),
        )))
        .add_command(Box::new(ReplaceTextCommand::new(
            Range::new(Position::new(0), Position::new(5)),
            "Hi".to_string(),
        )));

    let result = batch.execute(&mut doc).unwrap();
    assert!(result.success);
    assert!(result.content_changed);
    assert_eq!(doc.text(), "Hi World");
}

#[test]
fn fluent_api_usage() {
    let mut doc = EditorDocument::new();

    // Test fluent insertion
    let result = doc.command().at(Position::new(0)).insert("Hello").unwrap();

    assert!(result.success);
    assert_eq!(doc.text(), "Hello");

    // Test fluent replacement
    let range = Range::new(Position::new(0), Position::new(5));
    let result = doc.command().range(range).replace("Hi").unwrap();

    assert!(result.success);
    assert_eq!(doc.text(), "Hi");
}

#[test]
fn document_extension_methods() {
    let mut doc = EditorDocument::new();

    // Test insert_at
    doc.insert_at(Position::new(0), "Hello").unwrap();
    assert_eq!(doc.text(), "Hello");

    // Test replace_range
    let range = Range::new(Position::new(0), Position::new(5));
    doc.replace_range(range, "Hi").unwrap();
    assert_eq!(doc.text(), "Hi");

    // Test delete_range
    let range = Range::new(Position::new(0), Position::new(2));
    doc.delete_range(range).unwrap();
    assert_eq!(doc.text(), "");
}

#[test]
fn command_memory_usage() {
    let insert_cmd = InsertTextCommand::new(Position::new(0), "Hello".to_string());
    let usage = insert_cmd.memory_usage();

    // Should account for the struct size plus string length
    assert!(usage >= core::mem::size_of::<InsertTextCommand>() + 5);
}
