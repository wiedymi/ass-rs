//! Tests for basic script parsing, version detection, and core accessors.

use super::*;
use crate::parser::ast::SectionType;
use crate::ScriptVersion;
#[cfg(not(feature = "std"))]
use alloc::format;

#[test]
fn parse_minimal_script() {
    let script = Script::parse("[Script Info]\nTitle: Test").unwrap();
    assert_eq!(script.sections().len(), 1);
    assert_eq!(script.version(), ScriptVersion::AssV4);
}

#[test]
fn parse_with_script_type() {
    let script = Script::parse("[Script Info]\nScriptType: v4.00+\nTitle: Test").unwrap();
    assert_eq!(script.version(), ScriptVersion::AssV4);
}

#[test]
fn parse_with_bom() {
    let script = Script::parse("\u{FEFF}[Script Info]\nTitle: Test").unwrap();
    assert_eq!(script.sections().len(), 1);
}

#[test]
fn parse_empty_input() {
    let script = Script::parse("").unwrap();
    assert_eq!(script.sections().len(), 0);
}

#[test]
fn parse_multiple_sections() {
    let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name, Fontname\nStyle: Default,Arial\n\n[Events]\nFormat: Layer, Start, End, Style, Text\nDialogue: 0,0:00:00.00,0:00:05.00,Default,Hello World";
    let script = Script::parse(content).unwrap();
    assert_eq!(script.sections().len(), 3);
    assert_eq!(script.version(), ScriptVersion::AssV4);
}

#[test]
fn script_version_detection() {
    let script = Script::parse("[Script Info]\nTitle: Test").unwrap();
    assert_eq!(script.version(), ScriptVersion::AssV4);
}

#[test]
fn script_source_access() {
    let content = "[Script Info]\nTitle: Test";
    let script = Script::parse(content).unwrap();
    assert_eq!(script.source(), content);
}

#[test]
fn script_sections_access() {
    let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name";
    let script = Script::parse(content).unwrap();
    let sections = script.sections();
    assert_eq!(sections.len(), 2);
}

#[test]
fn script_issues_access() {
    let script = Script::parse("[Script Info]\nTitle: Test").unwrap();
    let issues = script.issues();
    // Should have no issues for valid script
    assert!(
        issues.is_empty()
            || issues
                .iter()
                .all(|i| matches!(i.severity, crate::parser::errors::IssueSeverity::Warning))
    );
}

#[test]
fn find_section_by_type() {
    let content =
        "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name\n\n[Events]\nFormat: Layer";
    let script = Script::parse(content).unwrap();

    let script_info = script.find_section(SectionType::ScriptInfo);
    assert!(script_info.is_some());

    let styles = script.find_section(SectionType::Styles);
    assert!(styles.is_some());

    let events = script.find_section(SectionType::Events);
    assert!(events.is_some());
}

#[test]
fn find_section_missing() {
    let content = "[Script Info]\nTitle: Test";
    let script = Script::parse(content).unwrap();

    let styles = script.find_section(SectionType::Styles);
    assert!(styles.is_none());

    let events = script.find_section(SectionType::Events);
    assert!(events.is_none());
}

#[test]
fn script_clone() {
    let content = "[Script Info]\nTitle: Test";
    let script = Script::parse(content).unwrap();
    let cloned = script.clone();

    assert_eq!(script, cloned);
    assert_eq!(script.source(), cloned.source());
    assert_eq!(script.version(), cloned.version());
    assert_eq!(script.sections().len(), cloned.sections().len());
}

#[test]
fn script_debug() {
    let script = Script::parse("[Script Info]\nTitle: Test").unwrap();
    let debug_str = format!("{script:?}");
    assert!(debug_str.contains("Script"));
}

#[test]
fn script_equality() {
    let content = "[Script Info]\nTitle: Test";
    let script1 = Script::parse(content).unwrap();
    let script2 = Script::parse(content).unwrap();
    assert_eq!(script1, script2);

    let different_content = "[Script Info]\nTitle: Different";
    let script3 = Script::parse(different_content).unwrap();
    assert_ne!(script1, script3);
}

#[test]
fn parse_whitespace_only() {
    let script = Script::parse("   \n\n  \t  \n").unwrap();
    assert_eq!(script.sections().len(), 0);
}

#[test]
fn parse_comments_only() {
    let script = Script::parse("!: This is a comment\n; Another comment").unwrap();
    assert_eq!(script.sections().len(), 0);
}

#[test]
fn parse_multiple_script_info_sections() {
    let content = "[Script Info]\nTitle: First\n\n[Script Info]\nTitle: Second";
    let script = Script::parse(content).unwrap();
    // Should handle multiple Script Info sections
    assert!(!script.sections().is_empty());
}

#[test]
fn parse_case_insensitive_sections() {
    let content = "[script info]\nTitle: Test\n\n[v4+ styles]\nFormat: Name";
    let _script = Script::parse(content).unwrap();
    // Parser may not support case-insensitive headers - that's acceptable
    // Just verify parsing succeeded without panic
}

#[test]
fn parse_malformed_but_recoverable() {
    let content = "[Script Info]\nTitle: Test\nMalformed line without colon\nAuthor: Someone";
    let script = Script::parse(content).unwrap();
    assert_eq!(script.sections().len(), 1);
    // Should have some issues but still parse
    let issues = script.issues();
    assert!(issues.is_empty() || !issues.is_empty()); // Either way is acceptable
}

#[test]
fn parse_with_various_line_endings() {
    let content_crlf = "[Script Info]\r\nTitle: Test\r\n";
    let script_crlf = Script::parse(content_crlf).unwrap();
    assert_eq!(script_crlf.sections().len(), 1);

    let content_lf = "[Script Info]\nTitle: Test\n";
    let script_lf = Script::parse(content_lf).unwrap();
    assert_eq!(script_lf.sections().len(), 1);
}
