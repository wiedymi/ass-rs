//! Tokenization tests for the syntax highlighting extension.
//!
//! Covers section headers, script-info fields, styles, and event lines.

use crate::core::EditorDocument;
use crate::extensions::builtin::syntax_highlight::{SyntaxHighlightExtension, TokenType};

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn test_section_header_tokenization() {
    let mut ext = SyntaxHighlightExtension::new();
    let doc =
        EditorDocument::from_content("[Script Info]\n[V4+ Styles]\n[Events]\n[Unknown Section]")
            .unwrap();

    let tokens = ext.tokenize_document(&doc).unwrap();

    // Check that all lines are tokenized as section headers
    assert_eq!(tokens.len(), 4);
    for token in &tokens {
        assert_eq!(token.token_type, TokenType::SectionHeader);
    }

    // Check semantic info
    assert_eq!(tokens[0].semantic_info, Some("Script Info".to_string()));
    assert_eq!(tokens[1].semantic_info, Some("V4+ Styles".to_string()));
    assert_eq!(tokens[2].semantic_info, Some("Events".to_string()));
    assert_eq!(tokens[3].semantic_info, Some("Unknown Section".to_string()));
}

#[test]
fn test_script_info_tokenization() {
    let mut ext = SyntaxHighlightExtension::new();
    let doc = EditorDocument::from_content(
        "[Script Info]\nTitle: My Subtitle\nPlayResX: 1920\nPlayResY: 1080\n; This is a comment",
    )
    .unwrap();

    let tokens = ext.tokenize_document(&doc).unwrap();

    // Section header
    assert_eq!(tokens[0].token_type, TokenType::SectionHeader);

    // Field names and values
    assert_eq!(tokens[1].token_type, TokenType::FieldName);
    assert_eq!(tokens[1].semantic_info, Some("Title".to_string()));
    assert_eq!(tokens[2].token_type, TokenType::FieldValue);

    assert_eq!(tokens[3].token_type, TokenType::FieldName);
    assert_eq!(tokens[3].semantic_info, Some("PlayResX".to_string()));
    assert_eq!(tokens[4].token_type, TokenType::FieldValue);

    // Comment
    assert_eq!(tokens[7].token_type, TokenType::Comment);
}

#[test]
fn test_style_tokenization() {
    let mut ext = SyntaxHighlightExtension::new();
    let doc = EditorDocument::from_content(
        "[V4+ Styles]\nFormat: Name, Fontname, Fontsize\nStyle: Default,Arial,20",
    )
    .unwrap();

    let tokens = ext.tokenize_document(&doc).unwrap();

    // Find style name token
    let style_name_token = tokens
        .iter()
        .find(|t| t.token_type == TokenType::StyleName)
        .unwrap();
    assert_eq!(style_name_token.semantic_info, Some("Default".to_string()));
}

#[test]
fn test_event_tokenization() {
    let mut ext = SyntaxHighlightExtension::new();
    let doc = EditorDocument::from_content(
        "[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello {\\b1}world{\\b0}!",
    )
    .unwrap();

    let tokens = ext.tokenize_document(&doc).unwrap();

    // Check for different token types
    let has_event_type = tokens.iter().any(|t| t.token_type == TokenType::EventType);
    let has_timecode = tokens.iter().any(|t| t.token_type == TokenType::TimeCode);
    let has_style_name = tokens.iter().any(|t| t.token_type == TokenType::StyleName);
    let has_override_tag = tokens
        .iter()
        .any(|t| t.token_type == TokenType::OverrideTag);
    let has_text = tokens.iter().any(|t| t.token_type == TokenType::Text);

    assert!(has_event_type, "Should have event type token");
    assert!(has_timecode, "Should have timecode tokens");
    assert!(has_style_name, "Should have style name token");
    assert!(has_override_tag, "Should have override tag tokens");
    assert!(has_text, "Should have text tokens");
}
