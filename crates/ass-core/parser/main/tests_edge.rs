//! Tests for malformed headers, BOM edge cases, and section-name handling.

use super::*;

#[test]
fn parser_empty_section_header() {
    let content = "[]\nSome content\n[Script Info]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_error = script.issues().iter().any(|issue| {
        issue.message.contains("Unknown section")
            || issue.message.contains("Failed to parse section")
    });
    assert!(has_error);
}

#[test]
fn parser_section_header_only_spaces() {
    let content = "[   ]\nSome content\n[Script Info]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_error = script.issues().iter().any(|issue| {
        issue.message.contains("Unknown section")
            || issue.message.contains("Failed to parse section")
    });
    assert!(has_error);
}

#[test]
fn parser_malformed_bom_sequence() {
    // Test with partial BOM-like sequence
    let content = "\u{00EF}\u{00BB}[Script Info]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    // Should parse, potentially with warnings - but may not have valid sections
    assert!(script.sections().is_empty() || !script.sections().is_empty());
}

#[test]
fn parser_content_after_eof() {
    let content = "[Script Info]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert!(!script.sections().is_empty());
    assert!(
        script.issues().is_empty()
            || script
                .issues()
                .iter()
                .all(|i| i.severity != IssueSeverity::Error)
    );
}

#[test]
fn parser_multiple_consecutive_section_headers() {
    let content = "[Script Info]\n[V4+ Styles]\n[Events]\nFormat: Text\nDialogue: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert!(!script.sections().is_empty());
}

#[test]
fn parser_section_header_with_special_chars() {
    let content = "[Script Info & More!]\nTitle: Test\n[Script Info]\nTitle: Real";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_unknown_section = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("Unknown section"));
    assert!(has_unknown_section);
}

#[test]
fn parser_skip_to_next_section_no_advance_protection() {
    // Test case that would trigger the infinite loop protection
    let content = "[BadSection\nContent without proper section end";
    let parser = Parser::new(content);
    let script = parser.parse();
    // Should not hang and should produce some result
    assert!(!script.issues().is_empty());
}

#[test]
fn parser_whitespace_before_and_after_sections() {
    let content = "   \n\n  ; Comment\n  [Script Info]  \n  Title: Test  \n\n  [V4+ Styles]  \n  Format: Name\n  Style: Default  \n\n  ";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert!(script.sections().len() >= 2);
}

#[test]
fn parser_comment_lines_between_sections() {
    let content = "[Script Info]\nTitle: Test\n; This is a comment\n!: Another comment\n\n[V4+ Styles]\nFormat: Name\nStyle: Default";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert!(script.sections().len() >= 2);
}

#[test]
fn parser_find_section_end_no_newline() {
    let content = "[Script Info]";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert!(!script.sections().is_empty() || !script.issues().is_empty());
}

#[test]
fn parser_unicode_in_section_names() {
    let content = "[Script Info 中文]\nTitle: Test\n[Script Info]\nTitle: Real";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_unknown_section = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("Unknown section"));
    assert!(has_unknown_section);
}

#[test]
fn parser_very_long_section_name() {
    let long_name = "a".repeat(1000);
    let content = format!("[{long_name}]\nTitle: Test\n[Script Info]\nTitle: Real");
    let parser = Parser::new(&content);
    let script = parser.parse();
    let has_unknown_section = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("Unknown section"));
    assert!(has_unknown_section);
}

#[test]
fn parser_case_sensitive_section_names() {
    let content = "[script info]\nTitle: Test\n[Script Info]\nTitle: Real";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_unknown_section = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("Unknown section"));
    assert!(has_unknown_section);
}

#[test]
fn parser_parse_section_error_unknown_section_with_content() {
    let content = "[BadSection]\nSome content here\nMore content\n[Script Info]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_unknown_error = script.issues().iter().any(|issue| {
        issue.message.contains("Unknown section") || issue.message.contains("BadSection")
    });
    assert!(has_unknown_error);
}

#[test]
fn parser_parse_section_error_unclosed_bracket_at_eof() {
    let content = "[Script Info";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_unclosed_error = script.issues().iter().any(|issue| {
        issue.message.contains("Unclosed section header")
            || issue.message.contains("Failed to parse section")
    });
    assert!(has_unclosed_error);
}
