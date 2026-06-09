//! Position and range error tests.
//!
//! Tests for out-of-bounds positions, malformed ranges, and line/column
//! conversion failures.

use ass_editor::{EditorDocument, Position, Range};

#[test]
fn test_invalid_position_operations() {
    let mut doc = EditorDocument::from_content("Hello").unwrap();

    // Positions beyond document
    assert!(doc.insert(Position::new(100), "X").is_err());
    assert!(doc.position_to_line_column(Position::new(100)).is_err());

    // Negative-like positions (using large values that might wrap)
    let huge_pos = Position::new(usize::MAX);
    assert!(doc.insert(huge_pos, "X").is_err());

    // Position at exact boundary should work
    assert!(doc.insert(Position::new(5), " World").is_ok());
}

#[test]
fn test_invalid_range_operations() {
    let mut doc = EditorDocument::from_content("Hello World").unwrap();

    // Invalid ranges
    let test_ranges = [
        // End before start
        Range::new(Position::new(5), Position::new(0)),
        // Both positions beyond document
        Range::new(Position::new(100), Position::new(200)),
        // Start valid, end invalid
        Range::new(Position::new(5), Position::new(100)),
        // Extremely large ranges
        Range::new(Position::new(0), Position::new(usize::MAX)),
    ];

    for (idx, range) in test_ranges.iter().enumerate() {
        // First range gets normalized from (5,0) to (0,5) which is valid
        if idx == 0 {
            // This range is valid after normalization
            assert_eq!(range.start.offset, 0);
            assert_eq!(range.end.offset, 5);
            assert!(doc.delete(*range).is_ok());
            // Restore document
            doc.undo().unwrap();
            continue;
        }

        // These should fail due to out-of-bounds positions
        assert!(
            doc.delete(*range).is_err(),
            "delete should fail for range {range:?}"
        );
        assert!(
            doc.text_range(*range).is_err(),
            "text_range should fail for range {range:?}"
        );
        assert!(
            doc.replace(*range, "X").is_err(),
            "replace should fail for range {range:?}"
        );
    }
}

#[test]
fn test_line_column_errors() {
    let doc = EditorDocument::from_content("Line1\nLine2\nLine3").unwrap();

    // Test position_to_line_column with invalid positions instead
    let invalid_positions = vec![
        Position::new(1000), // Beyond document end
        Position::new(usize::MAX),
    ];

    for pos in invalid_positions {
        assert!(doc.position_to_line_column(pos).is_err());
    }

    // Test LineColumn construction errors
    assert!(ass_editor::core::LineColumn::new(0, 1).is_err()); // Line 0 doesn't exist
    assert!(ass_editor::core::LineColumn::new(1, 0).is_err()); // Column 0 doesn't exist
}
