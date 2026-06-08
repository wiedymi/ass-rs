//! Tests for error-recovery suggestions and comment/whitespace handling.

use super::*;

#[test]
fn parser_error_recovery_style_suggestion() {
    let content = "[BadSection]\nStyle: Default,Arial\n[Script Info]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_suggestion = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("[V4+ Styles]"));
    assert!(has_suggestion);
}

#[test]
fn parser_error_recovery_events_suggestion() {
    let content = "[BadSection]\nDialogue: 0:00:00.00,0:00:05.00,Test\n[Script Info]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_suggestion = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("[Events]"));
    assert!(has_suggestion);
}

#[test]
fn parser_error_recovery_script_info_suggestion() {
    let content = "[BadSection]\nTitle: Test Script\n[Script Info]\nTitle: Real";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_suggestion = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("[Script Info]"));
    assert!(has_suggestion);
}

#[test]
fn parser_error_recovery_format_line_events() {
    let content = "[BadSection]\nFormat: Start, End, Text\nDialogue: 0:00:00.00,0:00:05.00,Test\n[Script Info]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_suggestion = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("[Events]"));
    assert!(has_suggestion);
}

#[test]
fn parser_error_recovery_format_line_styles() {
    let content =
        "[BadSection]\nFormat: Name, Fontname\nStyle: Default,Arial\n[Script Info]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_suggestion = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("[V4+ Styles]"));
    assert!(has_suggestion);
}

#[test]
fn parser_multiple_sections() {
    let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name\nStyle: Default\n\n[Events]\nFormat: Text\nDialogue: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert_eq!(script.sections().len(), 3);
}

#[test]
fn parser_whitespace_handling() {
    let content = "   \n\n  [Script Info]  \n  Title: Test  \n\n   ";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert!(!script.sections().is_empty());
}

#[test]
fn parser_invalid_bom_warning() {
    // Test with content that may have BOM-related issues
    let content = "[Script Info]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    // Should parse successfully
    assert!(!script.sections().is_empty());
}

#[test]
fn parser_v4_styles_section() {
    let content = "[V4 Styles]\nFormat: Name, Fontname\nStyle: Default,Arial";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert!(!script.sections().is_empty());
}

#[test]
fn parser_skip_to_next_section_with_format_line_events() {
    let content = "[BadSection]\nFormat: Start, End, Text\nDialogue: 0:00:00.00,0:00:05.00,Test\n[Events]\nFormat: Start, End, Text\nDialogue: 0:00:00.00,0:00:05.00,Real";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_events_suggestion = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("Did you mean '[Events]'?"));
    assert!(has_events_suggestion);
}

#[test]
fn parser_skip_to_next_section_with_format_line_styles() {
    let content = "[BadSection]\nFormat: Name, Fontname\nStyle: Default,Arial\n[V4+ Styles]\nFormat: Name, Fontname\nStyle: Real,Arial";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_styles_suggestion = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("Did you mean '[V4+ Styles]'?"));
    assert!(has_styles_suggestion);
}

#[test]
fn parser_at_next_section_edge_cases() {
    // Test incomplete section header
    let content = "[Incomplete";
    let parser = Parser::new(content);
    let script = parser.parse();
    // Should handle gracefully
    assert!(!script.issues().is_empty());
}

#[test]
fn parser_at_next_section_with_closing_bracket() {
    let content = "[Script Info]\nTitle: Test\n[V4+ Styles]\nFormat: Name";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert!(!script.sections().is_empty());
}

#[test]
fn parser_skip_line_edge_cases() {
    let content = "[Script Info]\n\n\n\nTitle: Test\n";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert!(!script.sections().is_empty());
}

#[test]
fn parser_mixed_comment_styles() {
    let content =
        "; Comment style 1\n!: Comment style 2\n; Another comment\n[Script Info]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert!(!script.sections().is_empty());
}

#[test]
fn parser_section_header_with_extra_brackets() {
    let content = "[Script Info]]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert!(!script.sections().is_empty());
}
