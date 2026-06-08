//! Behavioural tests for [`FontsParser`] covering common `[Fonts]` inputs.

use super::*;
use crate::parser::ast::Section;

#[test]
fn fonts_parser_empty_section() {
    let source = "";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert!(fonts.is_empty());
    } else {
        panic!("Expected Fonts section");
    }
}

#[test]
fn fonts_parser_single_font() {
    let source = "fontname: arial.ttf\ndata1\ndata2\n";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].filename, "arial.ttf");
        assert_eq!(fonts[0].data_lines.len(), 2);
        assert_eq!(fonts[0].data_lines[0], "data1");
        assert_eq!(fonts[0].data_lines[1], "data2");
    } else {
        panic!("Expected Fonts section");
    }
}

#[test]
fn fonts_parser_multiple_fonts() {
    let source = "fontname: font1.ttf\ndata1\ndata2\n\nfontname: font2.ttf\ndata3\ndata4\n";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert_eq!(fonts.len(), 2);

        assert_eq!(fonts[0].filename, "font1.ttf");
        assert_eq!(fonts[0].data_lines.len(), 2);
        assert_eq!(fonts[0].data_lines[0], "data1");
        assert_eq!(fonts[0].data_lines[1], "data2");

        assert_eq!(fonts[1].filename, "font2.ttf");
        assert_eq!(fonts[1].data_lines.len(), 2);
        assert_eq!(fonts[1].data_lines[0], "data3");
        assert_eq!(fonts[1].data_lines[1], "data4");
    } else {
        panic!("Expected Fonts section");
    }
}

#[test]
fn fonts_parser_with_comments() {
    let source = "; This is a comment\nfontname: test.ttf\n!: Another comment\ndata1\ndata2\n";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].filename, "test.ttf");
        assert_eq!(fonts[0].data_lines.len(), 2);
    } else {
        panic!("Expected Fonts section");
    }
}

#[test]
fn fonts_parser_with_whitespace() {
    let source = "  fontname:  arial.ttf  \n  data1  \n  data2  \n";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].filename, "arial.ttf");
        assert_eq!(fonts[0].data_lines.len(), 2);
        assert_eq!(fonts[0].data_lines[0], "  data1  ");
        assert_eq!(fonts[0].data_lines[1], "  data2  ");
    } else {
        panic!("Expected Fonts section");
    }
}

#[test]
fn fonts_parser_stops_at_next_section() {
    let source = "fontname: test.ttf\ndata1\ndata2\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].filename, "test.ttf");
        assert_eq!(fonts[0].data_lines.len(), 2);
    } else {
        panic!("Expected Fonts section");
    }
}

#[test]
fn fonts_parser_malformed_entry() {
    let source = "invalid_line\nfontname: valid.ttf\ndata1\n";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].filename, "valid.ttf");
        assert_eq!(fonts[0].data_lines.len(), 1);
    } else {
        panic!("Expected Fonts section");
    }
}

#[test]
fn fonts_parser_no_data_lines() {
    let source = "fontname: empty.ttf\n[Events]\n";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].filename, "empty.ttf");
        assert!(fonts[0].data_lines.is_empty());
    } else {
        panic!("Expected Fonts section");
    }
}
