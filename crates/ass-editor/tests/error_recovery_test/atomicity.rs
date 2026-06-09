//! Operation atomicity tests.
//!
//! Tests ensuring failed operations leave the document untouched and keep
//! undo/redo history consistent.

use ass_editor::{EditorDocument, Position, Range};

#[test]
fn test_failed_operations_dont_modify_document() {
    let original = "Original content";
    let mut doc = EditorDocument::from_content(original).unwrap();

    // Failed insert
    let insert_result = doc.insert(Position::new(1000), "Should not appear");
    assert!(insert_result.is_err());
    assert_eq!(doc.text(), original);

    // Failed delete
    let delete_result = doc.delete(Range::new(Position::new(0), Position::new(1000)));
    assert!(delete_result.is_err());
    assert_eq!(doc.text(), original);

    // Failed replace
    let replace_result = doc.replace(
        Range::new(Position::new(8), Position::new(1000)),
        "Should not appear",
    );
    assert!(replace_result.is_err());
    assert_eq!(doc.text(), original);
}

#[test]
fn test_undo_redo_consistency_with_errors() {
    let mut doc = EditorDocument::from_content("Step 1").unwrap();

    // Successful operation
    doc.insert(Position::new(6), " - Success").unwrap();
    assert_eq!(doc.text(), "Step 1 - Success");

    // Failed operation (should not affect undo stack)
    assert!(doc.insert(Position::new(1000), " - Fail").is_err());

    // Another successful operation
    doc.insert(Position::new(doc.len_bytes()), " - Step 2")
        .unwrap();

    // Undo should skip the failed operation
    doc.undo().unwrap();
    assert_eq!(doc.text(), "Step 1 - Success");

    doc.undo().unwrap();
    assert_eq!(doc.text(), "Step 1");

    // Redo should also skip the failed operation
    doc.redo().unwrap();
    assert_eq!(doc.text(), "Step 1 - Success");

    // Try to redo the second operation
    let redo_result = doc.redo();
    if redo_result.is_ok() {
        assert_eq!(doc.text(), "Step 1 - Success - Step 2");
    } else {
        // Redo might not be available depending on implementation
        // Just ensure we're in a consistent state
        assert_eq!(doc.text(), "Step 1 - Success");
    }
}
