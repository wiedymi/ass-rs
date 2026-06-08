//! Tests for full styles-section parsing via [`StylesParser::parse`].

use super::*;
use crate::parser::ast::Section;
use alloc::format;

#[test]
fn parse_empty_section() {
    let parser = StylesParser::new("", 0, 1);
    let result = parser.parse();
    assert!(result.is_ok());
    let (section, ..) = result.unwrap();
    if let Section::Styles(styles) = section {
        assert!(styles.is_empty());
    } else {
        panic!("Expected Styles section");
    }
}

#[test]
fn parse_basic_style() {
    let content = "Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1\n";
    let parser = StylesParser::new(content, 0, 1);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, ..) = result.unwrap();
    if let Section::Styles(styles) = section {
        assert_eq!(styles.len(), 1);
        let style = &styles[0];
        assert_eq!(style.name, "Default");
        assert_eq!(style.fontname, "Arial");
        assert_eq!(style.fontsize, "20");
        // Check span
        assert!(style.span.start > 0);
        assert!(style.span.end > style.span.start);
    } else {
        panic!("Expected Styles section");
    }
}

#[test]
fn parse_without_format_line() {
    let content = "Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1\n";
    let parser = StylesParser::new(content, 0, 1);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, ..) = result.unwrap();
    if let Section::Styles(styles) = section {
        assert_eq!(styles.len(), 1);
        assert_eq!(styles[0].name, "Default");
    } else {
        panic!("Expected Styles section");
    }
}

#[test]
fn parse_with_inheritance() {
    let content = "Style: *Default,NewStyle,Arial,24,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,1,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1\n";
    let parser = StylesParser::new(content, 0, 1);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, ..) = result.unwrap();
    if let Section::Styles(styles) = section {
        assert_eq!(styles.len(), 1);
        assert_eq!(styles[0].name, "NewStyle");
        assert_eq!(styles[0].parent, Some("Default"));
    } else {
        panic!("Expected Styles section");
    }
}

#[test]
fn parse_with_position_tracking() {
    // Create a larger content that simulates a full file
    let prefix = "a".repeat(100); // 100 bytes of padding
    let section_content = "Style: Test,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,0,0,2,0,0,0,1\n";
    let full_content = format!("{prefix}{section_content}");

    // Parser starts at position 100
    let parser = StylesParser::new(&full_content, 100, 10);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, _, _, final_pos, final_line) = result.unwrap();
    if let Section::Styles(styles) = section {
        assert_eq!(styles.len(), 1);
        let style = &styles[0];
        assert_eq!(style.span.start, 100);
        assert_eq!(style.span.line, 10);
    } else {
        panic!("Expected Styles section");
    }

    assert_eq!(final_pos, 100 + section_content.len());
    assert_eq!(final_line, 11);
}
