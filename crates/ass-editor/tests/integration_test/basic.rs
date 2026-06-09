//! Basic document, editing, and extension-manager integration tests.

use ass_editor::{EditorDocument, ExtensionManager, Position, Range};

#[test]
fn test_basic_document_operations() {
    // These should always work
    let mut doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();

    // Basic operations
    doc.insert(Position::new(doc.len_bytes()), "\nAuthor: Test")
        .unwrap();
    assert!(doc.text().contains("Author: Test"));

    // Undo/redo
    doc.undo().unwrap();
    assert!(!doc.text().contains("Author: Test"));
    doc.redo().unwrap();
    assert!(doc.text().contains("Author: Test"));
}

#[test]
fn test_basic_editing() {
    let mut doc = EditorDocument::new();

    // Insert
    doc.insert(Position::new(0), "Hello World").unwrap();
    assert_eq!(doc.text(), "Hello World");

    // Delete
    doc.delete(Range::new(Position::new(0), Position::new(5)))
        .unwrap();
    assert_eq!(doc.text(), " World");

    // Replace
    doc.replace(Range::new(Position::new(0), Position::new(6)), "Goodbye")
        .unwrap();
    assert_eq!(doc.text(), "Goodbye");
}

#[test]
fn test_extension_manager_basic() {
    let manager = ExtensionManager::new();
    let extensions = manager.list_extensions();
    assert_eq!(extensions.len(), 0);
}
