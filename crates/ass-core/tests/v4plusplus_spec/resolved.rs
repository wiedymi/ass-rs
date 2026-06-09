//! Resolved-style margin tests for ASS v4++ specification support
//!
//! Verifies that `ResolvedStyle` correctly resolves vertical margins: falling
//! back to `margin_v` for v4+ styles and preferring separate `margin_t` /
//! `margin_b` for v4++ styles.

use ass_core::{
    analysis::styles::resolved_style::ResolvedStyle,
    parser::ast::{Span, Style},
};

#[test]
fn test_resolved_style_margin_logic() {
    // Test ResolvedStyle correctly resolves v4+ vs v4++ margins

    // v4+ style with single vertical margin
    let v4plus_style = Style {
        name: "V4Plus",
        parent: None,
        fontname: "Arial",
        fontsize: "20",
        primary_colour: "&H00FFFFFF",
        secondary_colour: "&H000000FF",
        outline_colour: "&H00000000",
        back_colour: "&H80000000",
        bold: "0",
        italic: "0",
        underline: "0",
        strikeout: "0",
        scale_x: "100",
        scale_y: "100",
        spacing: "0",
        angle: "0",
        border_style: "1",
        outline: "2",
        shadow: "0",
        alignment: "2",
        margin_l: "10",
        margin_r: "10",
        margin_v: "15",
        margin_t: None,
        margin_b: None,
        encoding: "1",
        relative_to: None,
        span: Span::new(0, 0, 0, 0),
    };

    let resolved_v4plus = ResolvedStyle::from_style(&v4plus_style).unwrap();
    assert_eq!(resolved_v4plus.margin_l(), 10);
    assert_eq!(resolved_v4plus.margin_r(), 10);
    assert_eq!(resolved_v4plus.margin_t(), 15); // Should use margin_v
    assert_eq!(resolved_v4plus.margin_b(), 15); // Should use margin_v

    // v4++ style with separate top/bottom margins
    let v4plusplus_style = Style {
        name: "V4PlusPlus",
        parent: None,
        fontname: "Arial",
        fontsize: "20",
        primary_colour: "&H00FFFFFF",
        secondary_colour: "&H000000FF",
        outline_colour: "&H00000000",
        back_colour: "&H80000000",
        bold: "0",
        italic: "0",
        underline: "0",
        strikeout: "0",
        scale_x: "100",
        scale_y: "100",
        spacing: "0",
        angle: "0",
        border_style: "1",
        outline: "2",
        shadow: "0",
        alignment: "2",
        margin_l: "10",
        margin_r: "10",
        margin_v: "0", // Should be ignored when margin_t/margin_b are present
        margin_t: Some("20"),
        margin_b: Some("25"),
        encoding: "1",
        relative_to: Some("1"),
        span: Span::new(0, 0, 0, 0),
    };

    let resolved_v4plusplus = ResolvedStyle::from_style(&v4plusplus_style).unwrap();
    assert_eq!(resolved_v4plusplus.margin_l(), 10);
    assert_eq!(resolved_v4plusplus.margin_r(), 10);
    assert_eq!(resolved_v4plusplus.margin_t(), 20); // Should use margin_t
    assert_eq!(resolved_v4plusplus.margin_b(), 25); // Should use margin_b
}
