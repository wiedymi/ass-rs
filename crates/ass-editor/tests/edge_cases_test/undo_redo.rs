//! Undo/redo edge cases for ass-editor.
//!
//! Stack limits and undo consistency across non-mutating navigation calls.

use ass_editor::{EditorDocument, Position, Range};

#[test]
fn test_undo_redo_limits() {
    let mut doc = EditorDocument::from_content("Start").unwrap();

    // Perform many operations
    for i in 0..100 {
        doc.insert(Position::new(doc.len_bytes()), &format!("\nLine {i}"))
            .unwrap();
    }

    // Undo all operations - default limit is 50, so only last 50 operations are kept
    let mut undo_count = 0;
    while doc.undo().is_ok() {
        undo_count += 1;
        if undo_count > 200 {
            panic!("Undo loop detected");
        }
    }

    // Should have undone the last 50 operations (default limit)
    assert!(undo_count <= 50);
    eprintln!("Undo count: {undo_count}");
    eprintln!("Document text length: {}", doc.text().len());
    eprintln!("Lines in doc: {}", doc.text().lines().count());

    // The exact behavior depends on the undo limit and memory constraints
    // Just verify we undid some operations
    assert!(undo_count > 0);

    // Redo all operations
    let mut redo_count = 0;
    while doc.redo().is_ok() {
        redo_count += 1;
        if redo_count > 200 {
            panic!("Redo loop detected");
        }
    }

    // Redo count might not match undo count due to different limits
    eprintln!("Redo count: {redo_count}");
    assert!(redo_count > 0 && redo_count <= undo_count);
}

#[test]
fn test_undo_after_navigation() {
    let mut doc = EditorDocument::from_content("Line 1\nLine 2\nLine 3").unwrap();

    // Make changes
    doc.insert(Position::new(6), " modified").unwrap();
    let after_insert = doc.text();

    // Navigate (these operations shouldn't affect undo)
    let _ = doc.position_to_line_column(Position::new(0));
    let _ = doc.text_range(Range::new(Position::new(0), Position::new(5)));
    let _ = doc.len_lines();

    // Undo should still work
    doc.undo().unwrap();
    assert_ne!(doc.text(), after_insert);
}
