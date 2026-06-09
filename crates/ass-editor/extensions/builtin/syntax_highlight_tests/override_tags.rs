//! Override-tag tokenization and error-detection tests.
//!
//! Exercises override-tag parsing, tag parameters, unclosed-tag errors, and
//! multiple consecutive override tags.

use crate::core::EditorDocument;
use crate::extensions::builtin::syntax_highlight::{SyntaxHighlightExtension, TokenType};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

#[test]
fn test_override_tag_parsing() {
    let mut ext = SyntaxHighlightExtension::new();
    let doc = EditorDocument::from_content(
        "[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\\pos(100,200)\\fs30\\c&H0000FF&}Text",
    )
    .unwrap();

    let tokens = ext.tokenize_document(&doc).unwrap();

    // Count override tag tokens
    let tag_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::OverrideTag)
        .collect();

    assert!(!tag_tokens.is_empty(), "Should have override tag tokens");

    // Check for tag parameters
    let has_params = tokens
        .iter()
        .any(|t| t.token_type == TokenType::TagParameter);
    assert!(has_params, "Should have tag parameter tokens");
}

#[test]
fn test_error_detection() {
    let mut ext = SyntaxHighlightExtension::new();
    // Unclosed override tag
    let doc = EditorDocument::from_content(
        "[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\\b1 no closing brace",
    )
    .unwrap();

    let tokens = ext.tokenize_document(&doc).unwrap();

    // Should have error token
    let error_token = tokens.iter().find(|t| t.token_type == TokenType::Error);
    assert!(error_token.is_some(), "Should detect unclosed tag as error");
    assert_eq!(
        error_token.unwrap().semantic_info,
        Some("Unclosed override tag".to_string())
    );
}

#[test]
fn test_multiple_override_tags() {
    let mut ext = SyntaxHighlightExtension::new();
    let doc = EditorDocument::from_content(
        "[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\\b1}Bold{\\b0} {\\i1}Italic{\\i0}",
    )
    .unwrap();

    let tokens = ext.tokenize_document(&doc).unwrap();

    // Count override tag open/close braces
    let brace_count = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::OverrideTag)
        .count();

    // Should have at least 8 braces (4 tags * 2 braces each)
    assert!(brace_count >= 8, "Should have multiple override tag braces");
}
