//! Edge-case tests for [`FontsParser`]: malformed entries, comments, and EOF.

use super::*;
use crate::parser::ast::Section;

#[test]
fn fonts_parser_colon_in_filename() {
    let source = "fontname: C:\\Fonts\\arial.ttf\ndata1\n";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].filename, "C:\\Fonts\\arial.ttf");
    } else {
        panic!("Expected Fonts section");
    }
}

#[test]
fn fonts_parser_malformed_entry_no_colon() {
    let source = "invalid_font_entry\ndata1\ndata2\n";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        // Should skip malformed entries without colon
        assert!(fonts.is_empty());
    } else {
        panic!("Expected Fonts section");
    }
}

#[test]
fn fonts_parser_empty_filename() {
    let source = "fontname: \ndata1\ndata2\n";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].filename, "");
        assert_eq!(fonts[0].data_lines.len(), 2);
    } else {
        panic!("Expected Fonts section");
    }
}

#[test]
fn fonts_parser_whitespace_only_filename() {
    let source = "fontname:   \ndata1\ndata2\n";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].filename, "");
        assert_eq!(fonts[0].data_lines.len(), 2);
    } else {
        panic!("Expected Fonts section");
    }
}

#[test]
fn fonts_parser_comments_between_data_lines() {
    let source = "fontname: arial.ttf\ndata1\n; Comment line\ndata2\n! Another comment\ndata3\n";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].filename, "arial.ttf");
        // Comments should be skipped, only data lines included
        assert_eq!(fonts[0].data_lines.len(), 3);
        assert_eq!(fonts[0].data_lines[0], "data1");
        assert_eq!(fonts[0].data_lines[1], "data2");
        assert_eq!(fonts[0].data_lines[2], "data3");
    } else {
        panic!("Expected Fonts section");
    }
}

#[test]
fn fonts_parser_empty_lines_between_data() {
    let source = "fontname: arial.ttf\ndata1\n\n\ndata2\n   \ndata3\n";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].filename, "arial.ttf");
        // Parser stops at first empty line
        assert_eq!(fonts[0].data_lines.len(), 1);
        assert_eq!(fonts[0].data_lines[0], "data1");
    } else {
        panic!("Expected Fonts section");
    }
}

#[test]
fn fonts_parser_entry_at_end_of_file() {
    let source = "fontname: arial.ttf\ndata1\ndata2";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].filename, "arial.ttf");
        assert_eq!(fonts[0].data_lines.len(), 2);
    } else {
        panic!("Expected Fonts section");
    }
}

#[test]
fn fonts_parser_mixed_comment_styles() {
    let source = "fontname: arial.ttf\ndata1\n; Semicolon comment\ndata2\n! Exclamation comment\ndata3\n# Hash comment\ndata4\n";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert_eq!(fonts.len(), 1);
        // Hash comments are not skipped, so parsing stops at # Hash comment
        assert_eq!(fonts[0].data_lines.len(), 3);
    } else {
        panic!("Expected Fonts section");
    }
}

#[test]
fn fonts_parser_multiple_entries_with_edge_cases() {
    let source = "fontname: font1.ttf\ndata1_1\ndata1_2\n\ninvalid_entry_no_colon\n\nfontname: font2.ttf\n; Comment\ndata2_1\n\nfontname: \ndata3_1\n";
    let (section, _, _) = FontsParser::parse(source, 0, 1);

    if let Section::Fonts(fonts) = section {
        assert_eq!(fonts.len(), 3); // All valid font entries should be parsed

        assert_eq!(fonts[0].filename, "font1.ttf");
        assert_eq!(fonts[0].data_lines.len(), 2);

        assert_eq!(fonts[1].filename, "font2.ttf");
        assert_eq!(fonts[1].data_lines.len(), 1);

        assert_eq!(fonts[2].filename, "");
        assert_eq!(fonts[2].data_lines.len(), 1);
    } else {
        panic!("Expected Fonts section");
    }
}
