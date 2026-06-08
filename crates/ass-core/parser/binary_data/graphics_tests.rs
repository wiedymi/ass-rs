//! Behavioural tests for [`GraphicsParser`] covering common `[Graphics]` inputs.

use super::*;
use crate::parser::ast::Section;

#[test]
fn graphics_parser_empty_section() {
    let source = "";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert!(graphics.is_empty());
    } else {
        panic!("Expected Graphics section");
    }
}

#[test]
fn graphics_parser_single_graphic() {
    let source = "filename: logo.png\nimage_data1\nimage_data2\n";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert_eq!(graphics.len(), 1);
        assert_eq!(graphics[0].filename, "logo.png");
        assert_eq!(graphics[0].data_lines.len(), 2);
        assert_eq!(graphics[0].data_lines[0], "image_data1");
        assert_eq!(graphics[0].data_lines[1], "image_data2");
    } else {
        panic!("Expected Graphics section");
    }
}

#[test]
fn graphics_parser_multiple_graphics() {
    let source = "filename: img1.png\ndata1\ndata2\n\nfilename: img2.jpg\ndata3\ndata4\n";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert_eq!(graphics.len(), 2);

        assert_eq!(graphics[0].filename, "img1.png");
        assert_eq!(graphics[0].data_lines.len(), 2);
        assert_eq!(graphics[0].data_lines[0], "data1");
        assert_eq!(graphics[0].data_lines[1], "data2");

        assert_eq!(graphics[1].filename, "img2.jpg");
        assert_eq!(graphics[1].data_lines.len(), 2);
        assert_eq!(graphics[1].data_lines[0], "data3");
        assert_eq!(graphics[1].data_lines[1], "data4");
    } else {
        panic!("Expected Graphics section");
    }
}

#[test]
fn graphics_parser_with_comments() {
    let source =
        "; Image section comment\nfilename: test.png\n!: Another comment\nimg_data1\nimg_data2\n";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert_eq!(graphics.len(), 1);
        assert_eq!(graphics[0].filename, "test.png");
        assert_eq!(graphics[0].data_lines.len(), 2);
    } else {
        panic!("Expected Graphics section");
    }
}

#[test]
fn graphics_parser_with_whitespace() {
    let source = "  filename:  logo.png  \n  img_data1  \n  img_data2  \n";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert_eq!(graphics.len(), 1);
        assert_eq!(graphics[0].filename, "logo.png");
        assert_eq!(graphics[0].data_lines.len(), 2);
        assert_eq!(graphics[0].data_lines[0], "  img_data1  ");
        assert_eq!(graphics[0].data_lines[1], "  img_data2  ");
    } else {
        panic!("Expected Graphics section");
    }
}

#[test]
fn graphics_parser_stops_at_next_section() {
    let source =
        "filename: test.png\nimg_data1\nimg_data2\n[Styles]\nFormat: Name, Fontname, Fontsize\n";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert_eq!(graphics.len(), 1);
        assert_eq!(graphics[0].filename, "test.png");
        assert_eq!(graphics[0].data_lines.len(), 2);
    } else {
        panic!("Expected Graphics section");
    }
}

#[test]
fn graphics_parser_malformed_entry() {
    let source = "invalid_line_without_colon\nfilename: valid.png\nimg_data1\n";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert_eq!(graphics.len(), 1);
        assert_eq!(graphics[0].filename, "valid.png");
        assert_eq!(graphics[0].data_lines.len(), 1);
    } else {
        panic!("Expected Graphics section");
    }
}

#[test]
fn graphics_parser_no_data_lines() {
    let source = "filename: empty.png\n[Fonts]\n";
    let (section, _, _) = GraphicsParser::parse(source, 0, 1);

    if let Section::Graphics(graphics) = section {
        assert_eq!(graphics.len(), 1);
        assert_eq!(graphics[0].filename, "empty.png");
        assert!(graphics[0].data_lines.is_empty());
    } else {
        panic!("Expected Graphics section");
    }
}
