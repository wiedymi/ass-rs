//! Edge-case tests for [`GraphicsParser`]: malformed entries, comments, and EOF.

use super::*;
use crate::parser::ast::Section;

#[test]
fn graphics_parser_colon_in_filename() {
    let source = "filename: D:\\Images\\logo.png\nimg_data1\n";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert_eq!(graphics.len(), 1);
        assert_eq!(graphics[0].filename, "D:\\Images\\logo.png");
    } else {
        panic!("Expected Graphics section");
    }
}

#[test]
fn graphics_parser_malformed_entry_no_colon() {
    let source = "invalid_graphic_entry\nimg_data1\nimg_data2\n";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        // Should skip malformed entries without colon
        assert!(graphics.is_empty());
    } else {
        panic!("Expected Graphics section");
    }
}

#[test]
fn graphics_parser_empty_filename() {
    let source = "filename: \nimg_data1\nimg_data2\n";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert_eq!(graphics.len(), 1);
        assert_eq!(graphics[0].filename, "");
        assert_eq!(graphics[0].data_lines.len(), 2);
    } else {
        panic!("Expected Graphics section");
    }
}

#[test]
fn graphics_parser_whitespace_only_filename() {
    let source = "filename:   \nimg_data1\nimg_data2\n";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert_eq!(graphics.len(), 1);
        assert_eq!(graphics[0].filename, "");
        assert_eq!(graphics[0].data_lines.len(), 2);
    } else {
        panic!("Expected Graphics section");
    }
}

#[test]
fn graphics_parser_comments_between_data_lines() {
    let source =
        "filename: logo.png\nimg_data1\n; Comment line\nimg_data2\n! Another comment\nimg_data3\n";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert_eq!(graphics.len(), 1);
        assert_eq!(graphics[0].filename, "logo.png");
        // Comments should be skipped, only data lines included
        assert_eq!(graphics[0].data_lines.len(), 3);
        assert_eq!(graphics[0].data_lines[0], "img_data1");
        assert_eq!(graphics[0].data_lines[1], "img_data2");
        assert_eq!(graphics[0].data_lines[2], "img_data3");
    } else {
        panic!("Expected Graphics section");
    }
}

#[test]
fn graphics_parser_empty_lines_between_data() {
    let source = "filename: logo.png\nimg_data1\n\n\nimg_data2\n   \nimg_data3\n";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert_eq!(graphics.len(), 1);
        assert_eq!(graphics[0].filename, "logo.png");
        // Parser stops at first empty line
        assert_eq!(graphics[0].data_lines.len(), 1);
        assert_eq!(graphics[0].data_lines[0], "img_data1");
    } else {
        panic!("Expected Graphics section");
    }
}

#[test]
fn graphics_parser_entry_at_end_of_file() {
    let source = "filename: logo.png\nimg_data1\nimg_data2";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert_eq!(graphics.len(), 1);
        assert_eq!(graphics[0].filename, "logo.png");
        assert_eq!(graphics[0].data_lines.len(), 2);
    } else {
        panic!("Expected Graphics section");
    }
}

#[test]
fn graphics_parser_mixed_comment_styles() {
    let source = "filename: logo.png\nimg_data1\n; Semicolon comment\nimg_data2\n! Exclamation comment\nimg_data3\n# Hash comment\nimg_data4\n";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert_eq!(graphics.len(), 1);
        // Hash comments are not skipped, so parsing stops at # Hash comment
        assert_eq!(graphics[0].data_lines.len(), 3);
    } else {
        panic!("Expected Graphics section");
    }
}

#[test]
fn graphics_parser_multiple_entries_with_edge_cases() {
    let source = "filename: image1.png\nimg1_1\nimg1_2\n\ninvalid_entry_no_colon\n\nfilename: image2.png\n; Comment\nimg2_1\n\nfilename: \nimg3_1\n";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert_eq!(graphics.len(), 3); // All valid graphic entries should be parsed

        assert_eq!(graphics[0].filename, "image1.png");
        assert_eq!(graphics[0].data_lines.len(), 2);

        assert_eq!(graphics[1].filename, "image2.png");
        assert_eq!(graphics[1].data_lines.len(), 1);

        assert_eq!(graphics[2].filename, "");
        assert_eq!(graphics[2].data_lines.len(), 1);
    } else {
        panic!("Expected Graphics section");
    }
}
