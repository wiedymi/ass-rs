//! Tests for undo/redo across insert, replace, and delete operations

use super::*;
use crate::core::position::{Position, Range};

#[test]
fn test_undo_redo_basic() {
    let mut doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();
    let initial_len = doc.len_bytes();
    // println!(
    //     "Initial doc length: {}, content: {:?}",
    //     initial_len,
    //     doc.text()
    // );

    // Insert some text
    doc.insert(Position::new(initial_len), "\nAuthor: John")
        .unwrap();
    // println!(
    //     "After insert: length: {}, content: {:?}",
    //     doc.len_bytes(),
    //     doc.text()
    // );
    assert!(doc.text().contains("Author: John"));
    assert!(doc.can_undo());
    assert!(!doc.can_redo());

    // Undo the insert
    let result = doc.undo().unwrap();
    // println!(
    //     "After undo: length: {}, content: {:?}",
    //     doc.len_bytes(),
    //     doc.text()
    // );
    assert!(result.success);
    assert!(!doc.text().contains("Author: John"));
    assert!(!doc.can_undo());
    assert!(doc.can_redo());

    // Redo the insert
    // println!("About to redo...");
    let result = doc.redo().unwrap();
    // println!(
    //     "After redo: length: {}, content: {:?}",
    //     doc.len_bytes(),
    //     doc.text()
    // );
    assert!(result.success);
    assert!(doc.text().contains("Author: John"));
    assert!(doc.can_undo());
    assert!(!doc.can_redo());
}

#[test]
fn test_undo_redo_multiple_operations() {
    let mut doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();

    // Multiple operations
    doc.insert(Position::new(doc.len_bytes()), "\nAuthor: John")
        .unwrap();
    doc.insert(Position::new(doc.len_bytes()), "\nVersion: 1.0")
        .unwrap();
    doc.insert(Position::new(doc.len_bytes()), "\nComment: Test script")
        .unwrap();

    assert!(doc.text().contains("Author: John"));
    assert!(doc.text().contains("Version: 1.0"));
    assert!(doc.text().contains("Comment: Test script"));

    // Undo all operations
    doc.undo().unwrap();
    assert!(!doc.text().contains("Comment: Test script"));

    doc.undo().unwrap();
    assert!(!doc.text().contains("Version: 1.0"));

    doc.undo().unwrap();
    assert!(!doc.text().contains("Author: John"));

    // Redo one operation
    doc.redo().unwrap();
    assert!(doc.text().contains("Author: John"));
    assert!(!doc.text().contains("Version: 1.0"));
}

#[test]
fn test_undo_redo_replace() {
    let mut doc = EditorDocument::from_content("[Script Info]\nTitle: Original").unwrap();

    // Find and replace "Original" with "Modified"
    let start = doc.text().find("Original").unwrap();
    let range = Range::new(Position::new(start), Position::new(start + 8));
    doc.replace(range, "Modified").unwrap();

    assert!(doc.text().contains("Title: Modified"));
    assert!(!doc.text().contains("Original"));

    // Undo the replace
    doc.undo().unwrap();
    assert!(doc.text().contains("Title: Original"));
    assert!(!doc.text().contains("Modified"));

    // Redo the replace
    doc.redo().unwrap();
    assert!(doc.text().contains("Title: Modified"));
    assert!(!doc.text().contains("Original"));
}

#[test]
fn test_undo_redo_delete() {
    let mut doc = EditorDocument::from_content("[Script Info]\nTitle: Test\nAuthor: John").unwrap();

    // Delete the Author line
    let start = doc.text().find("\nAuthor: John").unwrap();
    let range = Range::new(Position::new(start), Position::new(start + 13));
    doc.delete(range).unwrap();

    assert!(!doc.text().contains("Author: John"));

    // Undo the delete
    doc.undo().unwrap();
    assert!(doc.text().contains("Author: John"));

    // Redo the delete
    doc.redo().unwrap();
    assert!(!doc.text().contains("Author: John"));
}
