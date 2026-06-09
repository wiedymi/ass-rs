//! Error recovery edge cases for ass-editor.
//!
//! Operation atomicity and undo consistency after failed operations.

use ass_editor::{EditorDocument, Position, Range};

#[test]
fn test_operation_atomicity() {
    let mut doc = EditorDocument::from_content("Original").unwrap();
    let original_text = doc.text();

    // Try to delete beyond bounds - should fail without modifying document
    let result = doc.delete(Range::new(Position::new(5), Position::new(100)));
    assert!(result.is_err());
    assert_eq!(doc.text(), original_text);

    // Try to insert at invalid position - should fail without modifying document
    let result = doc.insert(Position::new(100), "Test");
    assert!(result.is_err());
    assert_eq!(doc.text(), original_text);
}

#[test]
fn test_undo_consistency_after_errors() {
    let mut doc = EditorDocument::from_content("Start").unwrap();

    // Successful operation
    doc.insert(Position::new(5), " Middle").unwrap();

    // Failed operation
    assert!(doc.insert(Position::new(100), " Bad").is_err());

    // Another successful operation
    doc.insert(Position::new(doc.len_bytes()), " End").unwrap();

    // Undo should skip the failed operation
    doc.undo().unwrap();
    assert_eq!(doc.text(), "Start Middle");

    doc.undo().unwrap();
    assert_eq!(doc.text(), "Start");
}
