//! Clone, debug, and equality tests for the `Style` AST node.

use super::super::Span;
use super::*;
#[cfg(not(feature = "std"))]
use alloc::format;

#[test]
fn style_clone() {
    let style = Style::default();
    let cloned = style.clone();
    assert_eq!(style, cloned);
}

#[test]
fn style_clone_custom() {
    let style = Style {
        name: "CustomStyle",
        parent: None,
        fontname: "Times New Roman",
        fontsize: "18",
        primary_colour: "&H00ff00",
        secondary_colour: "&Hff0000",
        outline_colour: "&H0000ff",
        back_colour: "&H808080",
        bold: "1",
        italic: "1",
        underline: "0",
        strikeout: "1",
        scale_x: "110",
        scale_y: "90",
        spacing: "2",
        angle: "15",
        border_style: "3",
        outline: "2",
        shadow: "1",
        alignment: "5",
        margin_l: "15",
        margin_r: "25",
        margin_v: "20",
        margin_t: None,
        margin_b: None,
        encoding: "0",
        relative_to: None,
        span: Span::new(0, 0, 0, 0),
    };

    let cloned = style.clone();
    assert_eq!(style, cloned);
    assert_eq!(cloned.name, "CustomStyle");
    assert_eq!(cloned.fontname, "Times New Roman");
    assert_eq!(cloned.bold, "1");
}

#[test]
fn style_debug() {
    let style = Style::default();
    let debug_str = format!("{style:?}");
    assert!(debug_str.contains("Style"));
    assert!(debug_str.contains("Default"));
}

#[test]
fn style_debug_custom() {
    let style = Style {
        name: "DebugTest",
        parent: None,
        fontname: "Helvetica",
        fontsize: "18",
        ..Style::default()
    };

    let debug_str = format!("{style:?}");
    assert!(debug_str.contains("Style"));
    assert!(debug_str.contains("DebugTest"));
    assert!(debug_str.contains("Helvetica"));
    assert!(debug_str.contains("18"));
}

#[test]
fn style_partial_eq() {
    let style1 = Style::default();
    let style2 = Style::default();
    assert_eq!(style1, style2);

    let style3 = Style {
        name: "Custom",
        parent: None,
        ..Style::default()
    };
    assert_ne!(style1, style3);
}

#[test]
fn style_partial_eq_different_fields() {
    let base = Style::default();

    // Test inequality with different fields
    let name_diff = Style {
        name: "Different",
        parent: None,
        ..Style::default()
    };
    assert_ne!(base, name_diff);

    let font_diff = Style {
        fontname: "Comic Sans",
        parent: None,
        ..Style::default()
    };
    assert_ne!(base, font_diff);

    let size_diff = Style {
        fontsize: "24",
        parent: None,
        ..Style::default()
    };
    assert_ne!(base, size_diff);

    let color_diff = Style {
        primary_colour: "&H00ff00",
        parent: None,
        ..Style::default()
    };
    assert_ne!(base, color_diff);
}
