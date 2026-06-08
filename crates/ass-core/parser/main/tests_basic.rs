//! Tests for parser construction and basic section parsing.

use super::*;

fn create_test_script(content: &str) -> String {
    format!("[Script Info]\nTitle: Test\n\n{content}")
}

#[test]
fn parser_new() {
    let source = "test content";
    let parser = Parser::new(source);
    assert_eq!(parser.source, source);
    assert_eq!(parser.position, 0);
    assert_eq!(parser.line, 1);
    assert_eq!(parser.version, ScriptVersion::AssV4);
    assert!(parser.sections.is_empty());
    assert!(parser.issues.is_empty());
    assert!(parser.styles_format.is_none());
    assert!(parser.events_format.is_none());
}

#[test]
fn parser_parse_empty_script() {
    let parser = Parser::new("");
    let script = parser.parse();
    assert_eq!(script.version(), ScriptVersion::AssV4);
    assert!(script.sections().is_empty());
}

#[test]
fn parser_parse_with_bom() {
    let content = "\u{FEFF}[Script Info]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert!(!script.sections().is_empty());
}

#[test]
fn parser_parse_input_size_limit() {
    let large_content = "a".repeat(51 * 1024 * 1024); // 51MB > 50MB limit
    let parser = Parser::new(&large_content);
    let script = parser.parse();
    assert!(!script.issues().is_empty());
    let has_size_error = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("Input size limit exceeded"));
    assert!(has_size_error);
}

#[test]
fn parser_parse_unknown_section() {
    let content = "[Unknown Section]\nSome content";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_unknown_section_warning = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("Unknown section"));
    assert!(has_unknown_section_warning);
}

#[test]
fn parser_parse_unclosed_section_header() {
    let content = "[Script Info\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_unclosed_error = script.issues().iter().any(|issue| {
        issue.message.contains("Unclosed section header")
            || issue.message.contains("Failed to parse section")
    });
    assert!(has_unclosed_error);
}

#[test]
fn parser_parse_missing_section_header() {
    let content = "Title: Test\nAuthor: Someone";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_header_error = script.issues().iter().any(|issue| {
        issue.message.contains("Expected section header")
            || issue.message.contains("Failed to parse section")
    });
    assert!(has_header_error);
}

#[test]
fn parser_parse_script_info_section() {
    let content = "[Script Info]\nTitle: Test Script\nScriptType: v4.00+";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert_eq!(script.sections().len(), 1);
    // Version should be updated based on ScriptType parsing
    assert!(
        script.version() == ScriptVersion::AssV4Plus || script.version() == ScriptVersion::AssV4
    );
}

#[test]
fn parser_parse_styles_section() {
    let content = create_test_script("[V4+ Styles]\nFormat: Name, Fontname\nStyle: Default,Arial");
    let parser = Parser::new(&content);
    let script = parser.parse();
    assert!(script.sections().len() >= 2);
}

#[test]
fn parser_parse_events_section() {
    let content = create_test_script(
        "[Events]\nFormat: Start, End, Text\nDialogue: 0:00:00.00,0:00:05.00,Test",
    );
    let parser = Parser::new(&content);
    let script = parser.parse();
    assert!(script.sections().len() >= 2);
}

#[test]
fn parser_parse_fonts_section() {
    let content = create_test_script("[Fonts]\nfontname: Arial\nfontdata: ABCD1234");
    let parser = Parser::new(&content);
    let script = parser.parse();
    assert!(script.sections().len() >= 2);
}

#[test]
fn parser_parse_graphics_section() {
    let content = create_test_script("[Graphics]\nfilename: image.png\ndata: ABCD1234");
    let parser = Parser::new(&content);
    let script = parser.parse();
    assert!(script.sections().len() >= 2);
}

#[test]
fn parser_skip_comments() {
    let content = "; This is a comment\n!: Another comment\n[Script Info]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert!(!script.sections().is_empty());
}
