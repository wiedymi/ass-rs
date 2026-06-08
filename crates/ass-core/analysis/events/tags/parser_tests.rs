//! Tests for override-block tag parsing and diagnostics.

use super::*;
use alloc::vec::Vec;

#[test]
fn test_parse_override_block_simple() {
    let mut tags = Vec::new();
    let mut diagnostics = Vec::new();

    parse_override_block("\\b1", 0, &mut tags, &mut diagnostics);

    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name(), "b");
    assert_eq!(tags[0].args(), "1");
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_parse_override_block_multiple() {
    let mut tags = Vec::new();
    let mut diagnostics = Vec::new();

    parse_override_block("\\b1\\i1\\pos(100,200)", 0, &mut tags, &mut diagnostics);

    assert_eq!(tags.len(), 3);
    assert_eq!(tags[0].name(), "b");
    assert_eq!(tags[1].name(), "i");
    assert_eq!(tags[2].name(), "pos");
    assert_eq!(tags[2].args(), "(100,200)");
}

#[test]
fn test_parse_override_block_empty() {
    let mut tags = Vec::new();
    let mut diagnostics = Vec::new();

    parse_override_block("", 0, &mut tags, &mut diagnostics);

    assert_eq!(tags.len(), 0);
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_override_tag_getters() {
    let mut tags = Vec::new();
    let mut diagnostics = Vec::new();

    parse_override_block(
        "\\b1\\move(100,200,300,400)",
        10,
        &mut tags,
        &mut diagnostics,
    );

    assert_eq!(tags.len(), 2);

    // Test complexity() getter (lines 76-77)
    assert_eq!(tags[0].complexity(), 1); // 'b' has complexity 1
    assert_eq!(tags[1].complexity(), 3); // 'move' has complexity 3

    // Test position() getter (lines 82-83)
    assert_eq!(tags[0].position(), 10); // First tag at offset 10
    assert_eq!(tags[1].position(), 13); // Second tag starts at 10 + 3 characters
}

#[test]
fn test_parse_override_block_empty_tag_name() {
    let mut tags = Vec::new();
    let mut diagnostics = Vec::new();

    // Test with empty tag name (just \\ followed by another tag)
    parse_override_block("\\\\b1", 0, &mut tags, &mut diagnostics);

    // Should have one diagnostic for empty override
    assert!(!diagnostics.is_empty());
    assert!(matches!(diagnostics[0].kind, DiagnosticKind::EmptyOverride));
}

#[test]
fn test_parse_override_block_valid_tag_creation() {
    let mut tags = Vec::new();
    let mut diagnostics = Vec::new();

    // Test valid tag creation (lines 142-145)
    parse_override_block("\\c&H00FF00&", 5, &mut tags, &mut diagnostics);

    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name(), "c");
    assert_eq!(tags[0].args(), "&H00FF00&");
    assert_eq!(tags[0].position(), 5);
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_parse_override_block_non_ascii_tag_args() {
    let mut tags = Vec::new();
    let mut diagnostics = Vec::new();
    parse_override_block("\\fn微软雅黑", 0, &mut tags, &mut diagnostics);
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].complexity(), 1);
    assert_eq!(tags[0].name(), "fn");
    assert_eq!(tags[0].args(), "微软雅黑");

    let mut tags = Vec::new();
    let mut diagnostics = Vec::new();
    parse_override_block_with_registry("\\fn微软雅黑", 0, &mut tags, &mut diagnostics, None);
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].complexity(), 1);
    assert_eq!(tags[0].name(), "fn");
    assert_eq!(tags[0].args(), "微软雅黑");
}

#[test]
fn test_parse_fn_tag_with_ascii_font_name() {
    for input in ["\\fnArial", "\\fnTimes New Roman"] {
        let expected_args = &input[3..];

        let mut tags = Vec::new();
        let mut diagnostics = Vec::new();
        parse_override_block(input, 0, &mut tags, &mut diagnostics);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name(), "fn");
        assert_eq!(tags[0].args(), expected_args);
        assert_eq!(diagnostics.len(), 0);

        let mut tags = Vec::new();
        let mut diagnostics = Vec::new();
        parse_override_block_with_registry(input, 0, &mut tags, &mut diagnostics, None);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name(), "fn");
        assert_eq!(tags[0].args(), expected_args);
        assert_eq!(diagnostics.len(), 0);
    }
}

#[test]
fn test_parse_r_tag_with_and_without_ascii_style() {
    for (input, expected_args) in [("\\rAlternate", "Alternate"), ("\\r", "")] {
        let mut tags = Vec::new();
        let mut diagnostics = Vec::new();
        parse_override_block(input, 0, &mut tags, &mut diagnostics);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name(), "r");
        assert_eq!(tags[0].args(), expected_args);

        let mut tags = Vec::new();
        let mut diagnostics = Vec::new();
        parse_override_block_with_registry(input, 0, &mut tags, &mut diagnostics, None);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name(), "r");
        assert_eq!(tags[0].args(), expected_args);
    }
}

#[test]
fn test_parse_fn_and_r_followed_by_more_tags() {
    let mut tags = Vec::new();
    let mut diagnostics = Vec::new();
    parse_override_block("\\fnArial\\fs20\\b1", 0, &mut tags, &mut diagnostics);
    assert_eq!(tags.len(), 3);
    assert_eq!((tags[0].name(), tags[0].args()), ("fn", "Arial"));
    assert_eq!((tags[1].name(), tags[1].args()), ("fs", "20"));
    assert_eq!((tags[2].name(), tags[2].args()), ("b", "1"));

    let mut tags = Vec::new();
    let mut diagnostics = Vec::new();
    parse_override_block("\\rStyle\\b1", 0, &mut tags, &mut diagnostics);
    assert_eq!(tags.len(), 2);
    assert_eq!((tags[0].name(), tags[0].args()), ("r", "Style"));
    assert_eq!((tags[1].name(), tags[1].args()), ("b", "1"));
}

#[test]
fn test_fn_break_does_not_affect_other_f_or_r_tags() {
    let mut tags = Vec::new();
    let mut diagnostics = Vec::new();
    parse_override_block(
        "\\frz45\\fscx150\\fad(100,200)",
        0,
        &mut tags,
        &mut diagnostics,
    );
    assert_eq!(tags.len(), 3);
    assert_eq!((tags[0].name(), tags[0].args()), ("frz", "45"));
    assert_eq!((tags[1].name(), tags[1].args()), ("fscx", "150"));
    assert_eq!((tags[2].name(), tags[2].args()), ("fad", "(100,200)"));
}
