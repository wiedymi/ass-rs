//! Unit tests for position, range, builder, and selection types.

use super::*;

#[test]
fn position_operations() {
    let pos = Position::new(10);
    assert_eq!(pos.advance(5).offset, 15);
    assert_eq!(pos.retreat(5).offset, 5);
    assert_eq!(pos.retreat(20).offset, 0); // saturating
}

#[test]
fn line_column_validation() {
    assert!(LineColumn::new(0, 1).is_err());
    assert!(LineColumn::new(1, 0).is_err());
    assert!(LineColumn::new(1, 1).is_ok());
}

#[test]
fn range_normalization() {
    let r = Range::new(Position::new(10), Position::new(5));
    assert_eq!(r.start.offset, 5);
    assert_eq!(r.end.offset, 10);
}

#[test]
fn range_operations() {
    let r1 = Range::new(Position::new(5), Position::new(10));
    let r2 = Range::new(Position::new(8), Position::new(15));

    assert!(r1.overlaps(&r2));
    assert_eq!(r1.union(&r2).start.offset, 5);
    assert_eq!(r1.union(&r2).end.offset, 15);

    let intersection = r1.intersection(&r2).unwrap();
    assert_eq!(intersection.start.offset, 8);
    assert_eq!(intersection.end.offset, 10);
}

#[test]
fn selection_direction() {
    let sel = Selection::new(Position::new(10), Position::new(5));
    assert!(sel.is_reversed());
    assert_eq!(sel.range().start.offset, 5);
    assert_eq!(sel.range().end.offset, 10);
}

#[test]
#[cfg(feature = "rope")]
fn position_builder_with_rope() {
    let rope = ropey::Rope::from_str("Line 1\nLine 2\nLine 3");
    let pos = PositionBuilder::new()
        .line(2)
        .column(1)
        .build(&rope)
        .unwrap();
    assert_eq!(pos.offset, 7); // After "Line 1\n"
}

#[test]
#[cfg(not(feature = "rope"))]
fn position_builder_offset() {
    let pos = PositionBuilder::new().offset(42).build().unwrap();
    assert_eq!(pos.offset, 42);
}
