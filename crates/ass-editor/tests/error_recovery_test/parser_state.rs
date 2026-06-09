//! Parser recovery and state consistency tests.
//!
//! Tests that the parser recovers gracefully from corrupted input and that
//! document state stays consistent across mixed valid/invalid operations.

use ass_editor::{EditorDocument, Position, Range};

#[test]
fn test_parser_recovery_from_corruption() {
    // Create long string outside vec to avoid temporary value issue
    let nested_tags = format!("[Events]\nDialogue: {}", "{".repeat(1000));

    let corrupted_cases = vec![
        // Binary data mixed with text
        "[\0Script Info]\nTitle: Test",
        // Truncated file
        "[Script Info]\nTitle: Test\n[Eve",
        // Mixed encodings (simulated)
        // "[Script Info]\nTitle: Test\xFF\xFE", // Invalid hex escapes
        // Extremely nested tags
        nested_tags.as_str(),
        // Unclosed sections
        "[Script Info\nTitle: Test\n[Events",
    ];

    for content in corrupted_cases {
        let result = EditorDocument::from_content(content);

        // Parser should either recover or fail gracefully
        match result {
            Ok(doc) => {
                // If it parsed, should be able to get text back
                let _ = doc.text();
                // Should be able to perform operations
                let mut doc_mut = doc;
                let _ = doc_mut.insert(Position::new(0), "X");
            }
            Err(_) => {
                // Failed gracefully - this is also acceptable
            }
        }
    }
}

// ===== State Consistency =====

#[test]
fn test_document_state_consistency_after_errors() {
    let mut doc = EditorDocument::from_content("Initial").unwrap();

    // Track state
    let initial_len = doc.len_bytes();
    let initial_lines = doc.len_lines();

    // Perform mix of valid and invalid operations
    doc.insert(Position::new(7), " state").unwrap();
    assert!(doc.insert(Position::new(1000), "fail").is_err());
    doc.delete(Range::new(Position::new(0), Position::new(7)))
        .unwrap();
    assert!(doc
        .delete(Range::new(Position::new(0), Position::new(1000)))
        .is_err());

    // State should be consistent
    assert_eq!(doc.text(), " state");
    assert_eq!(doc.len_bytes(), 6);
    assert_eq!(doc.len_lines(), 1);

    // Undo all successful operations
    doc.undo().unwrap();
    doc.undo().unwrap();

    // Should be back to initial state
    assert_eq!(doc.text(), "Initial");
    assert_eq!(doc.len_bytes(), initial_len);
    assert_eq!(doc.len_lines(), initial_lines);
}
