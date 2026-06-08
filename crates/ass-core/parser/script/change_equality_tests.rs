//! Tests for `ChangeTracker` defaults and `Change` value equality.

use super::*;
use crate::parser::ast::Style;
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

#[test]
fn test_change_tracker_default() {
    let tracker = ChangeTracker::<'_>::default();
    assert!(!tracker.is_enabled());
    assert!(tracker.is_empty());
    assert_eq!(tracker.len(), 0);
}

#[test]
fn test_change_equality() {
    use crate::parser::ast::Span;

    let style = Style {
        name: "Test",
        parent: None,
        fontname: "Arial",
        fontsize: "20",
        primary_colour: "&H00FFFFFF",
        secondary_colour: "&H000000FF",
        outline_colour: "&H00000000",
        back_colour: "&H00000000",
        bold: "0",
        italic: "0",
        underline: "0",
        strikeout: "0",
        scale_x: "100",
        scale_y: "100",
        spacing: "0",
        angle: "0",
        border_style: "1",
        outline: "0",
        shadow: "0",
        alignment: "2",
        margin_l: "0",
        margin_r: "0",
        margin_v: "0",
        margin_t: None,
        margin_b: None,
        encoding: "1",
        relative_to: None,
        span: Span::new(0, 0, 0, 0),
    };

    let change1 = Change::Added {
        offset: 100,
        content: LineContent::Style(Box::new(style.clone())),
        line_number: 5,
    };

    let change2 = Change::Added {
        offset: 100,
        content: LineContent::Style(Box::new(style)),
        line_number: 5,
    };

    assert_eq!(change1, change2);
}
