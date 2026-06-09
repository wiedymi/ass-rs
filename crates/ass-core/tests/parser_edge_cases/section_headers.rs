//! Tests for section-header parsing and recovery in the ASS parser.
//!
//! Covers malformed sections, missing or unclosed header brackets,
//! abrupt file endings, and empty or whitespace-only section names.

use ass_core::{
    parser::{IssueCategory, IssueSeverity},
    Script,
};

/// Test malformed section causing `parse_section` error (L113-L126)
#[test]
fn test_malformed_section_error_recovery() {
    let malformed_script = r"
[Script Info]
Title: Test

[Malformed Section
This section has no closing bracket

[Events]
Format: Start, End, Style, Text
Dialogue: 0:00:00.00,0:00:05.00,Default,Test
";

    let script = Script::parse(malformed_script).expect("Script parsing should work");

    // Should have issues for malformed section (might be warnings, not necessarily errors)
    assert!(script
        .issues()
        .iter()
        .any(|issue| { matches!(issue.category, IssueCategory::Structure) }));

    // Should still parse the valid sections
    assert!(!script.sections().is_empty());
}

/// Test `ExpectedSectionHeader` error (L131-L133)
#[test]
fn test_expected_section_header_error() {
    let script_no_bracket = r"
Script Info]
Title: Test
";

    let script = Script::parse(script_no_bracket).expect("Script parsing should work");

    assert!(script
        .issues()
        .iter()
        .any(|issue| matches!(issue.severity, IssueSeverity::Error)));
}

/// Test `UnclosedSectionHeader` error (L135-L137)
#[test]
fn test_unclosed_section_header_error() {
    let script_unclosed = r"
[Script Info
Title: Test
";

    let script = Script::parse(script_unclosed).expect("Script parsing should work");

    assert!(script
        .issues()
        .iter()
        .any(|issue| matches!(issue.severity, IssueSeverity::Error)));
}

/// Test file ending abruptly during parsing
#[test]
fn test_abrupt_file_ending() {
    let truncated_script = "[Script Info]\nTitle: Test\n[Events";

    let script = Script::parse(truncated_script).expect("Script parsing should work");

    // Should handle truncated file gracefully
    assert!(script
        .issues()
        .iter()
        .any(|issue| matches!(issue.severity, IssueSeverity::Error)));
}

/// Test empty section name
#[test]
fn test_empty_section_name() {
    let empty_section = "[]";

    let script = Script::parse(empty_section).expect("Script parsing should work");

    // Should have some kind of issue (error or warning) for empty section name
    assert!(!script.issues().is_empty());
}

/// Test section name with only whitespace
#[test]
fn test_whitespace_only_section_name() {
    let whitespace_section = "[   \t  ]";

    let script = Script::parse(whitespace_section).expect("Script parsing should work");

    // Should have some kind of issue (error or warning) for whitespace-only section name
    assert!(!script.issues().is_empty());
}
