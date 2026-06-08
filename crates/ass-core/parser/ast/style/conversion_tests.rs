//! ASS string serialization tests for the `Style` AST node.

use super::*;
#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn style_to_ass_string() {
    let style = Style::default();
    let ass_string = style.to_ass_string();

    assert_eq!(
        ass_string,
        "Style: Default,Arial,20,&Hffffff,&H0000ff,&H000000,&H000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1"
    );
}

#[test]
fn style_to_ass_string_custom() {
    let style = Style {
        name: "Custom",
        fontname: "Times New Roman",
        fontsize: "24",
        primary_colour: "&H00ff00",
        bold: "-1",
        italic: "1",
        scale_x: "95",
        scale_y: "105",
        alignment: "5",
        ..Style::default()
    };

    let ass_string = style.to_ass_string();
    assert!(ass_string.contains("Custom,Times New Roman,24"));
    assert!(ass_string.contains("&H00ff00"));
    assert!(ass_string.contains("-1,1")); // bold, italic
    assert!(ass_string.contains("95,105")); // scale_x, scale_y
    assert!(ass_string.contains(",5,")); // alignment
}

#[test]
fn style_to_ass_string_with_format() {
    let style = Style {
        name: "TestStyle",
        fontname: "Arial",
        fontsize: "20",
        ..Style::default()
    };

    // V4+ standard format
    let v4_format = vec![
        "Name",
        "Fontname",
        "Fontsize",
        "PrimaryColour",
        "SecondaryColour",
        "OutlineColour",
        "BackColour",
        "Bold",
        "Italic",
        "Underline",
        "StrikeOut",
        "ScaleX",
        "ScaleY",
        "Spacing",
        "Angle",
        "BorderStyle",
        "Outline",
        "Shadow",
        "Alignment",
        "MarginL",
        "MarginR",
        "MarginV",
        "Encoding",
    ];
    let v4_string = style.to_ass_string_with_format(&v4_format);
    assert_eq!(
        v4_string,
        "Style: TestStyle,Arial,20,&Hffffff,&H0000ff,&H000000,&H000000,0,0,0,0,100,100,0,0,1,0,0,2,10,10,10,1"
    );

    // Minimal format
    let min_format = vec!["Name", "Fontname", "Fontsize"];
    let min_string = style.to_ass_string_with_format(&min_format);
    assert_eq!(min_string, "Style: TestStyle,Arial,20");

    // V4++ format with margin_t/margin_b
    let style_v4pp = Style {
        name: "V4++Style",
        margin_t: Some("15"),
        margin_b: Some("20"),
        relative_to: Some("video"),
        ..Style::default()
    };
    let v4pp_format = vec![
        "Name",
        "Fontname",
        "Fontsize",
        "PrimaryColour",
        "SecondaryColour",
        "OutlineColour",
        "BackColour",
        "Bold",
        "Italic",
        "Underline",
        "StrikeOut",
        "ScaleX",
        "ScaleY",
        "Spacing",
        "Angle",
        "BorderStyle",
        "Outline",
        "Shadow",
        "Alignment",
        "MarginL",
        "MarginR",
        "MarginT",
        "MarginB",
        "Encoding",
        "RelativeTo",
    ];
    let v4pp_string = style_v4pp.to_ass_string_with_format(&v4pp_format);
    assert!(v4pp_string.contains("V4++Style"));
    assert!(v4pp_string.contains(",15,20,")); // margin_t, margin_b
    assert!(v4pp_string.contains(",video")); // relative_to
}
