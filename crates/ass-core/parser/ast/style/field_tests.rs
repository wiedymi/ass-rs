//! Field-access and lifetime tests for the `Style` AST node.

use super::super::Span;
use super::*;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[test]
fn style_field_access() {
    let style = Style {
        name: "TestStyle",
        parent: None,
        fontname: "Comic Sans",
        fontsize: "24",
        bold: "1",
        italic: "1",
        ..Style::default()
    };

    assert_eq!(style.name, "TestStyle");
    assert_eq!(style.fontname, "Comic Sans");
    assert_eq!(style.fontsize, "24");
    assert_eq!(style.bold, "1");
    assert_eq!(style.italic, "1");
}

#[test]
fn style_field_access_comprehensive() {
    let style = Style {
        name: "ComprehensiveTest",
        parent: None,
        fontname: "Verdana",
        fontsize: "14",
        primary_colour: "&Hff0000",
        secondary_colour: "&H00ff00",
        outline_colour: "&H0000ff",
        back_colour: "&Hffffff",
        bold: "-1",
        italic: "1",
        underline: "1",
        strikeout: "0",
        scale_x: "125",
        scale_y: "75",
        spacing: "3",
        angle: "45",
        border_style: "3",
        outline: "3",
        shadow: "2",
        alignment: "9",
        margin_l: "20",
        margin_r: "30",
        margin_v: "15",
        margin_t: None,
        margin_b: None,
        encoding: "2",
        relative_to: None,
        span: Span::new(0, 0, 0, 0),
    };

    // Test all field accesses
    assert_eq!(style.name, "ComprehensiveTest");
    assert_eq!(style.fontname, "Verdana");
    assert_eq!(style.fontsize, "14");
    assert_eq!(style.primary_colour, "&Hff0000");
    assert_eq!(style.secondary_colour, "&H00ff00");
    assert_eq!(style.outline_colour, "&H0000ff");
    assert_eq!(style.back_colour, "&Hffffff");
    assert_eq!(style.bold, "-1");
    assert_eq!(style.italic, "1");
    assert_eq!(style.underline, "1");
    assert_eq!(style.strikeout, "0");
    assert_eq!(style.scale_x, "125");
    assert_eq!(style.scale_y, "75");
    assert_eq!(style.spacing, "3");
    assert_eq!(style.angle, "45");
    assert_eq!(style.border_style, "3");
    assert_eq!(style.outline, "3");
    assert_eq!(style.shadow, "2");
    assert_eq!(style.alignment, "9");
    assert_eq!(style.margin_l, "20");
    assert_eq!(style.margin_r, "30");
    assert_eq!(style.margin_v, "15");
    assert_eq!(style.encoding, "2");
}

#[test]
fn style_empty_strings() {
    let style = Style {
        name: "",
        parent: None,
        fontname: "",
        fontsize: "",
        primary_colour: "",
        secondary_colour: "",
        outline_colour: "",
        back_colour: "",
        bold: "",
        italic: "",
        underline: "",
        strikeout: "",
        scale_x: "",
        scale_y: "",
        spacing: "",
        angle: "",
        border_style: "",
        outline: "",
        shadow: "",
        alignment: "",
        margin_l: "",
        margin_r: "",
        margin_v: "",
        margin_t: None,
        margin_b: None,
        encoding: "",
        relative_to: None,
        span: Span::new(0, 0, 0, 0),
    };

    // All fields should be empty strings
    assert_eq!(style.name, "");
    assert_eq!(style.fontname, "");
    assert_eq!(style.fontsize, "");
    assert_eq!(style.primary_colour, "");
    assert_eq!(style.alignment, "");
    assert_eq!(style.encoding, "");
}

#[test]
fn style_lifetimes() {
    let source = String::from("TestStyle,Times,16");
    let style = {
        let parts: Vec<&str> = source.split(',').collect();
        Style {
            name: parts[0],
            parent: None,
            fontname: parts[1],
            fontsize: parts[2],
            ..Style::default()
        }
    };

    assert_eq!(style.name, "TestStyle");
    assert_eq!(style.fontname, "Times");
    assert_eq!(style.fontsize, "16");
}
