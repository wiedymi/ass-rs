//! Tests for section errors, complex recovery, and size/BOM boundaries.

use super::*;

#[test]
fn parser_parse_section_error_empty_section_name() {
    let content = "[]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_empty_section_error = script.issues().iter().any(|issue| {
        issue.message.contains("Unknown section")
            || issue.message.contains("Failed to parse section")
    });
    assert!(has_empty_section_error);
}

#[test]
fn parser_parse_section_error_whitespace_only_section() {
    let content = "[   ]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_whitespace_error = script.issues().iter().any(|issue| {
        issue.message.contains("Unknown section")
            || issue.message.contains("Failed to parse section")
    });
    assert!(has_whitespace_error);
}

#[test]
fn parser_error_recovery_multiple_unknown_sections() {
    let content = "[BadSection1]\nStyle: Default,Arial\n[BadSection2]\nDialogue: 0:00:00.00,0:00:05.00,Test\n[Script Info]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    let style_suggestion_count = script
        .issues()
        .iter()
        .filter(|issue| issue.message.contains("[V4+ Styles]"))
        .count();
    let events_suggestion_count = script
        .issues()
        .iter()
        .filter(|issue| issue.message.contains("[Events]"))
        .count();
    assert!(style_suggestion_count >= 1);
    assert!(events_suggestion_count >= 1);
}

#[test]
fn parser_skip_to_next_section_no_protection_edge_case() {
    let content = "[UnknownSection]\nLine without next section";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_unknown_error = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("Unknown section"));
    assert!(has_unknown_error);
}

#[test]
fn parser_find_section_end_at_exact_boundary() {
    let content = "[Script Info]\nTitle: Test\n[V4+ Styles]";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert!(!script.sections().is_empty());
}

#[test]
fn parser_section_header_without_content() {
    let content = "[Script Info]\n[V4+ Styles]\nFormat: Name\nStyle: Default,Arial";
    let parser = Parser::new(content);
    let script = parser.parse();
    assert!(script.sections().len() >= 2);
}

#[test]
fn parser_malformed_section_headers_mixed() {
    let content = "[Script Info\nTitle: Test\n]NotASection[\n[V4+ Styles]\nFormat: Name\nStyle: Default,Arial";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_errors = script.issues().iter().any(|issue| {
        issue.message.contains("Unclosed section header")
            || issue.message.contains("Unknown section")
            || issue.message.contains("Failed to parse section")
    });
    assert!(has_errors);
}

#[test]
fn parser_nested_bracket_edge_cases() {
    let content = "[[Script Info]]\nTitle: Test\n[V4+ Styles]\nFormat: Name\nStyle: Default,Arial";
    let parser = Parser::new(content);
    let script = parser.parse();
    let has_unknown_error = script.issues().iter().any(|issue| {
        issue.message.contains("Unknown section") || issue.message.contains("[Script Info]")
    });
    assert!(has_unknown_error);
}

#[test]
fn parser_section_with_trailing_characters() {
    let content = "[Script Info] Extra Text\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    // Should parse successfully - trailing text after ] is ignored
    assert!(!script.sections().is_empty());
    // Should not generate unknown section errors
    let has_unknown_error = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("Unknown section"));
    assert!(!has_unknown_error);
}

#[test]
fn parser_complex_error_recovery_scenario() {
    let content = "[BadSection1]\nStyle: Test,Arial,20\nComment: 0,0:00:00.00,0:00:01.00,,Comment text\n[BadSection2]\nDialogue: 0,0:00:00.00,0:00:01.00,Default,,0,0,0,,Test\n[Script Info]\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();

    let has_style_suggestion = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("[V4+ Styles]"));
    let has_events_suggestion = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("[Events]"));

    assert!(has_style_suggestion);
    assert!(has_events_suggestion);
}

#[test]
fn parser_input_size_limit_exactly_at_boundary() {
    let content = "a".repeat(50 * 1024 * 1024 - 1);
    let parser = Parser::new(&content);
    let script = parser.parse();
    // Should not have size limit error since we're just under the limit
    let has_size_error = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("Input size limit exceeded"));
    assert!(!has_size_error);
}

#[test]
fn parser_bom_detection_partial_sequences() {
    // Create content with partial UTF-8 BOM (0xEF, 0xBB without 0xBF)
    let bytes = &[
        0xEF, 0xBB, b'[', b'S', b'c', b'r', b'i', b'p', b't', b' ', b'I', b'n', b'f', b'o', b']',
        b'\n', b'T', b'i', b't', b'l', b'e', b':', b' ', b'T', b'e', b's', b't',
    ];
    let content_partial_bom = String::from_utf8_lossy(bytes);
    let parser = Parser::new(&content_partial_bom);
    let script = parser.parse();
    let has_bom_warning = script
        .issues()
        .iter()
        .any(|issue| issue.message.contains("BOM") || issue.message.contains("byte order mark"));
    assert!(has_bom_warning);
}

#[test]
fn parser_version_detection_edge_cases() {
    let content = "[Script Info]\nScriptType: v4.00++\nTitle: Test";
    let parser = Parser::new(content);
    let script = parser.parse();
    // Should handle malformed script type gracefully
    assert!(!script.sections().is_empty());
}
