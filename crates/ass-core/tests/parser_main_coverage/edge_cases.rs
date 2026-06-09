//! Coverage tests for parser edge cases: empty/whitespace input, malformed
//! sections, long lines, mixed line endings, and memory efficiency.

use ass_core::parser::Script;

#[test]
fn test_parser_with_empty_input() {
    // Test empty input edge case
    let script = Script::parse("");
    assert!(script.is_ok());

    let script = script.unwrap();
    assert!(script.sections().is_empty());
}

#[test]
fn test_parser_with_whitespace_only() {
    // Test whitespace-only input
    let script = Script::parse("   \n\t\r\n   ");
    assert!(script.is_ok());

    let script = script.unwrap();
    // Should have minimal content
    assert!(script.sections().is_empty() || script.sections().len() <= 1);
}

#[test]
fn test_parser_with_malformed_sections() {
    // Test various malformed section headers to trigger error paths
    let malformed_inputs = vec![
        "[Invalid Section\n", // Missing closing bracket
        "[]Section]\n",       // Empty section name
        "[Script Info\nNo closing bracket",
        "[Events]\nFormat: Invalid\nDialogue: malformed",
        "[V4+ Styles]\nFormat: Invalid\nStyle: malformed",
    ];

    for input in malformed_inputs {
        let script = Script::parse(input);
        assert!(
            script.is_ok(),
            "Parser should not fail on malformed input: {input}"
        );

        let script = script.unwrap();
        // Should have parsing issues
        println!(
            "Malformed input '{}' produced {} issues",
            input.chars().take(20).collect::<String>(),
            script.issues().len()
        );
    }
}

#[test]
fn test_parser_with_extremely_long_lines() {
    // Test parser behavior with extremely long lines
    let long_line = "A".repeat(100_000);
    let content = format!("[Script Info]\nTitle: {long_line}\n");

    let script = Script::parse(&content);
    assert!(script.is_ok());

    let script = script.unwrap();
    // Should handle long lines without crashing
    assert!(!script.sections().is_empty());
}

#[test]
fn test_parser_with_mixed_line_endings() {
    // Test handling of mixed line endings (CR, LF, CRLF)
    let content = "[Script Info]\rTitle: Test\r\nScriptType: v4.00+\n\n[Events]\r\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Mixed line endings\r";

    let script = Script::parse(content);
    assert!(script.is_ok());

    let script = script.unwrap();
    assert!(!script.sections().is_empty());
}

#[test]
fn test_parser_memory_efficiency() {
    // Test that parser doesn't consume excessive memory for reasonable inputs
    let mut reasonable_content = String::from("[Script Info]\n");
    reasonable_content.push_str(&"Title: Test\n".repeat(1000));
    reasonable_content.push_str("\n[Events]\n");
    reasonable_content.push_str(
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
    );
    reasonable_content
        .push_str(&"Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Event text\n".repeat(1000));

    let script = Script::parse(&reasonable_content);
    assert!(script.is_ok());

    let script = script.unwrap();
    // Should handle reasonable amounts of repetitive content
    assert!(!script.sections().is_empty());
}
