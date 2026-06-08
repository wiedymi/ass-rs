//! Tests for the Script Info section parser.

use super::*;
use crate::parser::{ast::Section, errors::IssueSeverity};
#[cfg(not(feature = "std"))]
use alloc::format;

#[test]
fn parse_empty_section() {
    let parser = ScriptInfoParser::new("", 0, 1);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, version, ..) = result.unwrap();
    if let Section::ScriptInfo(info) = section {
        assert!(info.fields.is_empty());
        assert_eq!(info.span.start, 0);
        assert_eq!(info.span.end, 0);
    } else {
        panic!("Expected ScriptInfo section");
    }
    assert!(version.is_none());
}

#[test]
fn parse_basic_fields() {
    let content = "Title: Test Script\nScriptType: v4.00+\n";
    let parser = ScriptInfoParser::new(content, 0, 1);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, version, ..) = result.unwrap();
    if let Section::ScriptInfo(info) = section {
        assert_eq!(info.fields.len(), 2);
        assert_eq!(info.get_field("Title"), Some("Test Script"));
        assert_eq!(info.get_field("ScriptType"), Some("v4.00+"));
        assert_eq!(info.span.start, 0);
        assert_eq!(info.span.end, content.len());
        assert_eq!(info.span.line, 1);
        assert_eq!(info.span.column, 1);
    } else {
        panic!("Expected ScriptInfo section");
    }
    assert!(version.is_some());
}

#[test]
fn skip_comments_and_whitespace() {
    let content = "; Comment\n# Another comment\n\nTitle: Test\n";
    let parser = ScriptInfoParser::new(content, 0, 1);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, ..) = result.unwrap();
    if let Section::ScriptInfo(info) = section {
        assert_eq!(info.fields.len(), 1);
        assert_eq!(info.get_field("Title"), Some("Test"));
    } else {
        panic!("Expected ScriptInfo section");
    }
}

#[test]
fn handle_invalid_lines() {
    let content = "Title: Test\nInvalidLine\nAuthor: Someone\n";
    let parser = ScriptInfoParser::new(content, 0, 1);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, _, issues, ..) = result.unwrap();
    if let Section::ScriptInfo(info) = section {
        assert_eq!(info.fields.len(), 2);
        assert_eq!(info.get_field("Title"), Some("Test"));
        assert_eq!(info.get_field("Author"), Some("Someone"));
    } else {
        panic!("Expected ScriptInfo section");
    }

    // Should have a warning about the invalid line
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].severity, IssueSeverity::Warning);
}

#[test]
fn parse_with_position_tracking() {
    // Create a larger content that simulates a full file
    let prefix = "Some prefix\n"; // 12 bytes
    let section_content = "Title: Test\nAuthor: Someone\n";
    let full_content = format!("{prefix}{section_content}");

    // Parser starts at position 12 (after prefix)
    let parser = ScriptInfoParser::new(&full_content, 12, 2);
    let result = parser.parse();
    assert!(result.is_ok());

    let (section, _, _, final_pos, final_line) = result.unwrap();
    if let Section::ScriptInfo(info) = section {
        assert_eq!(info.fields.len(), 2);
        assert_eq!(info.fields[0], ("Title", "Test"));
        assert_eq!(info.fields[1], ("Author", "Someone"));
        assert_eq!(info.span.start, 12);
        assert_eq!(info.span.end, 12 + section_content.len());
        assert_eq!(info.span.line, 2);
        assert_eq!(info.span.column, 1);
    } else {
        panic!("Expected ScriptInfo section");
    }

    assert_eq!(final_pos, 12 + section_content.len());
    assert_eq!(final_line, 4); // Started at line 2, added 2 lines
}
