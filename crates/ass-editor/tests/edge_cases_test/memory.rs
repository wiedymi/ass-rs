//! Memory and performance edge cases for ass-editor.
//!
//! Large undo histories, pathological replace patterns, and position
//! advancement across multi-byte characters.

use ass_editor::{EditorDocument, Position, Range};

#[test]
fn test_large_undo_history() {
    let mut doc = EditorDocument::new();

    // Create a large undo history
    for i in 0..1000 {
        doc.insert(Position::new(0), &format!("{i}")).unwrap();
    }

    // Try to undo half - but only last 50 operations are kept (default limit)
    let mut actual_undos = 0;
    for _ in 0..500 {
        if doc.undo().is_ok() {
            actual_undos += 1;
        } else {
            break;
        }
    }

    // Should only be able to undo up to the limit (50 by default)
    assert!(actual_undos <= 50);

    // Make a new change (this might clear redo history)
    doc.insert(Position::new(0), "NEW").unwrap();

    // Redo should fail since we made a new change
    assert!(doc.redo().is_err());
}

#[test]
fn test_pathological_replace_patterns() {
    let mut doc = EditorDocument::from_content("aaaaaaaaaaaaaaaaaaaa").unwrap();

    // Replace operations that might cause issues
    doc.replace(
        Range::new(Position::new(0), Position::new(20)),
        "bbbbbbbbbbbbbbbbbbbb",
    )
    .unwrap();
    assert_eq!(doc.text(), "bbbbbbbbbbbbbbbbbbbb");

    // Replace with much larger text
    doc.replace(
        Range::new(Position::new(0), Position::new(20)),
        &"c".repeat(1000),
    )
    .unwrap();
    assert_eq!(doc.text().len(), 1000);

    // Replace with empty
    doc.replace(Range::new(Position::new(0), Position::new(1000)), "")
        .unwrap();
    assert_eq!(doc.text(), "");
}

#[test]
fn test_position_advance_edge_cases() {
    // Test position advancement with various character types
    let test_cases = vec![
        ("a", 1),
        ("é", 2),    // 2-byte UTF-8
        ("中", 3),   // 3-byte UTF-8
        ("𝄞", 4),    // 4-byte UTF-8
        ("\r\n", 2), // CRLF
        ("\n", 1),   // LF
        ("\t", 1),   // Tab
    ];

    for (ch, expected_advance) in test_cases {
        let pos = Position::new(0);
        let new_pos = pos.advance(ch.len());
        assert_eq!(new_pos.offset - pos.offset, expected_advance);
    }
}
