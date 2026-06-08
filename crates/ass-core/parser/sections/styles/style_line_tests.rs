//! Tests for the standalone [`StylesParser::parse_style_line`] entry point.

use super::*;
use crate::parser::errors::ParseError;
use alloc::vec;

#[test]
fn test_public_parse_style_line() {
    let format = vec![
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
    let line = "Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1";

    let result = StylesParser::parse_style_line(line, &format, 1);
    assert!(result.is_ok());

    let style = result.unwrap();
    assert_eq!(style.name, "Default");
    assert_eq!(style.fontname, "Arial");
    assert_eq!(style.fontsize, "20");
    assert!(style.parent.is_none());
}

#[test]
fn test_parse_style_line_with_inheritance() {
    let format = vec![
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
    let line = "*Default,NewStyle,Arial,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1";

    let result = StylesParser::parse_style_line(line, &format, 1);
    assert!(result.is_ok());

    let style = result.unwrap();
    assert_eq!(style.name, "NewStyle");
    assert_eq!(style.parent, Some("Default"));
    assert_eq!(style.fontsize, "24");
}

#[test]
fn test_parse_style_line_insufficient_fields() {
    let format = vec![
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
    let line = "Default,Arial,20"; // Missing fields

    let result = StylesParser::parse_style_line(line, &format, 1);
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(matches!(e, ParseError::InsufficientFields { .. }));
    }
}

#[test]
fn test_parse_style_line_with_empty_format() {
    // Test with empty format array - should use default
    let format = vec![];
    let line = "Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1";

    let result = StylesParser::parse_style_line(line, &format, 1);
    assert!(result.is_ok());

    let style = result.unwrap();
    assert_eq!(style.name, "Default");
    assert_eq!(style.fontname, "Arial");
}
