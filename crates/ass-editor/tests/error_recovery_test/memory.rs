//! Memory exhaustion scenario tests.
//!
//! Tests for large documents and deep undo stacks under memory pressure.

use ass_editor::{EditorDocument, Position};

#[test]
fn test_document_size_limits() {
    // Try to create documents approaching memory limits
    let sizes = vec![
        1_000_000, // 1MB
        10_000_000, // 10MB
                   // Note: Larger sizes might cause actual OOM, so we keep it reasonable
    ];

    for size in sizes {
        let content = "X".repeat(size);
        let result = EditorDocument::from_content(&content);

        if let Ok(mut doc) = result {
            // Document was created, try operations
            assert_eq!(doc.len_bytes(), size);

            // Try to double the size
            let double_result = doc.insert(Position::new(0), &content);
            if double_result.is_ok() {
                assert_eq!(doc.len_bytes(), size * 2);
            }
        }
    }
}

#[test]
fn test_undo_stack_memory_pressure() {
    let mut doc = EditorDocument::new();

    // Create many small operations to fill undo stack
    for i in 0..10000 {
        doc.insert(Position::new(0), &i.to_string()).unwrap();
    }

    // Undo many operations
    let mut undo_count = 0;
    while doc.undo().is_ok() && undo_count < 5000 {
        undo_count += 1;
    }

    // Create a large operation
    let large_text = "X".repeat(1_000_000);
    let result = doc.insert(Position::new(0), &large_text);

    if result.is_ok() {
        // Redo should now fail (history was cleared)
        assert!(doc.redo().is_err());
    }
}
