//! Position and range edge cases for ass-editor.
//!
//! Boundary position conversions and range validation/normalization.

use ass_editor::{EditorDocument, Position, Range};

#[test]
fn test_position_edge_cases() {
    let doc = EditorDocument::from_content("Hello\nWorld\n").unwrap();

    // Test position conversions at boundaries
    let test_positions = vec![
        (0, 1, 1),  // Start of document
        (5, 1, 6),  // End of first line (before \n)
        (6, 2, 1),  // Start of second line
        (11, 2, 6), // End of second line
        (12, 3, 1), // Start of empty third line
    ];

    for (offset, expected_line, expected_col) in test_positions {
        let pos = Position::new(offset);
        let lc = doc.position_to_line_column(pos).unwrap();
        assert_eq!(lc.line, expected_line, "offset {offset} line mismatch");
        assert_eq!(lc.column, expected_col, "offset {offset} column mismatch");
    }

    // Test invalid positions
    assert!(doc.position_to_line_column(Position::new(1000)).is_err());
}

#[test]
fn test_range_validation() {
    let doc = EditorDocument::from_content("Hello World").unwrap();

    // Valid ranges
    assert!(doc
        .text_range(Range::new(Position::new(0), Position::new(5)))
        .is_ok());
    assert!(doc
        .text_range(Range::new(Position::new(6), Position::new(11)))
        .is_ok());

    // Invalid ranges (end before start) - Range normalizes automatically,
    // so this creates a valid range from 5 to 10
    let normalized_range = Range::new(Position::new(10), Position::new(5));
    assert!(doc.text_range(normalized_range).is_ok());
    assert_eq!(normalized_range.start.offset, 5);
    assert_eq!(normalized_range.end.offset, 10);

    // Out of bounds ranges
    assert!(doc
        .text_range(Range::new(Position::new(0), Position::new(100)))
        .is_err());
    assert!(doc
        .text_range(Range::new(Position::new(100), Position::new(200)))
        .is_err());
}
