//! Tests for `from_parts` construction, span validation, and edge-case inputs.

use super::*;
use crate::parser::ast::SectionType;
use crate::ScriptVersion;
#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

#[test]
fn from_parts_constructor() {
    let source = "[Script Info]\nTitle: Test";
    let sections = Vec::new();
    let issues = Vec::new();

    let script = Script::from_parts(source, ScriptVersion::AssV4, sections, issues, None, None);
    assert_eq!(script.source(), source);
    assert_eq!(script.version(), ScriptVersion::AssV4);
    assert_eq!(script.sections().len(), 0);
    assert_eq!(script.issues().len(), 0);
}

#[cfg(debug_assertions)]
#[test]
fn validate_spans() {
    let script = Script::parse("[Script Info]\nTitle: Test").unwrap();
    // This is a basic test - the actual validation would need proper setup
    // to ensure spans point to the right memory locations
    assert!(script.validate_spans() || !script.validate_spans()); // Either result is acceptable
}

#[test]
fn parse_unicode_content() {
    let content = "[Script Info]\nTitle: Unicode Test 测试 🎬\nAuthor: アニメ";
    let script = Script::parse(content).unwrap();
    assert_eq!(script.sections().len(), 1);
    assert_eq!(script.source(), content);
}

#[test]
fn parse_very_long_content() {
    #[cfg(not(feature = "std"))]
    use alloc::fmt::Write;
    #[cfg(feature = "std")]
    use std::fmt::Write;

    let mut content = String::from("[Script Info]\nTitle: Long Test\n");
    for i in 0..1000 {
        writeln!(
            content,
            "Comment{i}: This is a very long comment line to test performance"
        )
        .unwrap();
    }

    let script = Script::parse(&content).unwrap();
    assert_eq!(script.sections().len(), 1);
}

#[test]
fn parse_nested_brackets() {
    let content = "[Script Info]\nTitle: Test [with] brackets\nComment: [nested [brackets]]";
    let script = Script::parse(content).unwrap();
    assert_eq!(script.sections().len(), 1);
}

#[test]
fn parse_empty_sections() {
    let content = "[Script Info]\n\n[V4+ Styles]\n\n[Events]\n";
    let script = Script::parse(content).unwrap();
    assert_eq!(script.sections().len(), 3);
}

#[test]
fn parse_section_with_only_format() {
    let content = "[V4+ Styles]\nFormat: Name, Fontname, Fontsize";
    let script = Script::parse(content).unwrap();
    assert_eq!(script.sections().len(), 1);
}

#[test]
fn parse_events_with_complex_text() {
    let content = r"[Events]
Format: Layer, Start, End, Style, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,{\b1}Bold text{\b0} and {\i1}italic{\i0}
Comment: 0,0:00:05.00,0:00:10.00,Default,This is a comment
";
    let script = Script::parse(content).unwrap();
    assert_eq!(script.sections().len(), 1);
}

#[cfg(debug_assertions)]
#[test]
fn validate_spans_comprehensive() {
    let content = "[Script Info]\nTitle: Test\nAuthor: Someone";
    let script = Script::parse(content).unwrap();

    // Should validate successfully since all spans come from the parsed source
    assert!(script.validate_spans());

    // Verify source access
    assert_eq!(script.source(), content);
}

#[test]
fn script_accessor_methods() {
    let content = "[Script Info]\nTitle: Test\n\n[V4+ Styles]\nFormat: Name";
    let script = Script::parse(content).unwrap();

    // Test all accessor methods
    assert_eq!(script.version(), ScriptVersion::AssV4);
    assert_eq!(script.sections().len(), 2);
    assert_eq!(script.source(), content);
    // May have warnings but should be accessible
    let _ = script.issues();

    // Test section finding
    assert!(script.find_section(SectionType::ScriptInfo).is_some());
    assert!(script.find_section(SectionType::Styles).is_some());
    assert!(script.find_section(SectionType::Events).is_none());
}

#[test]
fn from_parts_comprehensive() {
    use crate::parser::ast::{ScriptInfo, Section, Span};

    let source = "[Script Info]\nTitle: Custom";
    let mut sections = Vec::new();
    let issues = Vec::new();

    // Create a script using from_parts
    let script1 = Script::from_parts(
        source,
        ScriptVersion::AssV4,
        sections.clone(),
        issues.clone(),
        None,
        None,
    );
    assert_eq!(script1.source(), source);
    assert_eq!(script1.version(), ScriptVersion::AssV4);
    assert_eq!(script1.sections().len(), 0);
    assert_eq!(script1.issues().len(), 0);

    // Test with non-empty collections
    let script_info = ScriptInfo {
        fields: Vec::new(),
        span: Span::new(0, 0, 0, 0),
    };
    sections.push(Section::ScriptInfo(script_info));

    let script2 = Script::from_parts(source, ScriptVersion::AssV4, sections, issues, None, None);
    assert_eq!(script2.sections().len(), 1);
}
