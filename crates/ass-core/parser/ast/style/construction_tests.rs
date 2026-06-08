//! Construction and default-value tests for the `Style` AST node.

use super::super::Span;
use super::*;

#[test]
fn style_default_values() {
    let style = Style::default();
    assert_eq!(style.name, "Default");
    assert_eq!(style.fontname, "Arial");
    assert_eq!(style.fontsize, "20");
    assert_eq!(style.primary_colour, "&Hffffff");
    assert_eq!(style.alignment, "2");
}

#[test]
fn style_default_all_fields() {
    let style = Style::default();

    // Test all default field values
    assert_eq!(style.name, "Default");
    assert_eq!(style.fontname, "Arial");
    assert_eq!(style.fontsize, "20");
    assert_eq!(style.primary_colour, "&Hffffff");
    assert_eq!(style.secondary_colour, "&H0000ff");
    assert_eq!(style.outline_colour, "&H000000");
    assert_eq!(style.back_colour, "&H000000");
    assert_eq!(style.bold, "0");
    assert_eq!(style.italic, "0");
    assert_eq!(style.underline, "0");
    assert_eq!(style.strikeout, "0");
    assert_eq!(style.scale_x, "100");
    assert_eq!(style.scale_y, "100");
    assert_eq!(style.spacing, "0");
    assert_eq!(style.angle, "0");
    assert_eq!(style.border_style, "1");
    assert_eq!(style.outline, "0");
    assert_eq!(style.shadow, "0");
    assert_eq!(style.alignment, "2");
    assert_eq!(style.margin_l, "10");
    assert_eq!(style.margin_r, "10");
    assert_eq!(style.margin_v, "10");
    assert_eq!(style.encoding, "1");
}

#[test]
fn style_default_construction() {
    // Test that Default::default() works correctly
    let style: Style = Style::default();
    assert_eq!(style.name, "Default");
    assert_eq!(style.fontname, "Arial");
    assert_eq!(style.fontsize, "20");
    assert_eq!(style.primary_colour, "&Hffffff");
    assert_eq!(style.secondary_colour, "&H0000ff");
    assert_eq!(style.outline_colour, "&H000000");
    assert_eq!(style.back_colour, "&H000000");
    assert_eq!(style.bold, "0");
    assert_eq!(style.italic, "0");
    assert_eq!(style.underline, "0");
    assert_eq!(style.strikeout, "0");
    assert_eq!(style.scale_x, "100");
    assert_eq!(style.scale_y, "100");
    assert_eq!(style.spacing, "0");
    assert_eq!(style.angle, "0");
    assert_eq!(style.border_style, "1");
    assert_eq!(style.outline, "0");
    assert_eq!(style.shadow, "0");
    assert_eq!(style.alignment, "2");
    assert_eq!(style.margin_l, "10");
    assert_eq!(style.margin_r, "10");
    assert_eq!(style.margin_v, "10");
    assert_eq!(style.encoding, "1");
}

#[test]
fn style_struct_creation() {
    // Test direct struct creation syntax
    let style = Style {
        name: "TestName",
        parent: None,
        fontname: "TestFont",
        fontsize: "12",
        primary_colour: "&H123456",
        secondary_colour: "&H654321",
        outline_colour: "&Habcdef",
        back_colour: "&Hfedcba",
        bold: "1",
        italic: "1",
        underline: "1",
        strikeout: "1",
        scale_x: "150",
        scale_y: "75",
        spacing: "5",
        angle: "90",
        border_style: "2",
        outline: "1",
        shadow: "3",
        alignment: "5",
        margin_l: "25",
        margin_r: "35",
        margin_v: "20",
        margin_t: None,
        margin_b: None,
        encoding: "3",
        relative_to: None,
        span: Span::new(0, 0, 0, 0),
    };

    // Verify all fields are set correctly
    assert_eq!(style.name, "TestName");
    assert_eq!(style.fontname, "TestFont");
    assert_eq!(style.fontsize, "12");
    assert_eq!(style.primary_colour, "&H123456");
    assert_eq!(style.secondary_colour, "&H654321");
    assert_eq!(style.outline_colour, "&Habcdef");
    assert_eq!(style.back_colour, "&Hfedcba");
    assert_eq!(style.bold, "1");
    assert_eq!(style.italic, "1");
    assert_eq!(style.underline, "1");
    assert_eq!(style.strikeout, "1");
    assert_eq!(style.scale_x, "150");
    assert_eq!(style.scale_y, "75");
    assert_eq!(style.spacing, "5");
    assert_eq!(style.angle, "90");
    assert_eq!(style.border_style, "2");
    assert_eq!(style.outline, "1");
    assert_eq!(style.shadow, "3");
    assert_eq!(style.alignment, "5");
    assert_eq!(style.margin_l, "25");
    assert_eq!(style.margin_r, "35");
    assert_eq!(style.margin_v, "20");
    assert_eq!(style.encoding, "3");
}

#[test]
fn style_mix_default_and_custom() {
    // Test struct update syntax with defaults
    let style = Style {
        name: "MixedStyle",
        parent: None,
        fontsize: "22",
        bold: "1",
        italic: "1",
        primary_colour: "&Hff00ff",
        alignment: "7",
        ..Style::default()
    };

    // Custom fields
    assert_eq!(style.name, "MixedStyle");
    assert_eq!(style.fontsize, "22");
    assert_eq!(style.bold, "1");
    assert_eq!(style.italic, "1");
    assert_eq!(style.primary_colour, "&Hff00ff");
    assert_eq!(style.alignment, "7");

    // Default fields
    assert_eq!(style.fontname, "Arial");
    assert_eq!(style.underline, "0");
    assert_eq!(style.strikeout, "0");
    assert_eq!(style.scale_x, "100");
    assert_eq!(style.encoding, "1");
}
