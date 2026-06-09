//! Token rendering and comment-event tests.
//!
//! Verifies ANSI color codes, CSS class names, and comment event typing.

use crate::core::EditorDocument;
use crate::extensions::builtin::syntax_highlight::{SyntaxHighlightExtension, TokenType};

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn test_ansi_colors() {
    assert_eq!(TokenType::SectionHeader.ansi_color(), "\x1b[1;34m");
    assert_eq!(TokenType::FieldName.ansi_color(), "\x1b[36m");
    assert_eq!(TokenType::OverrideTag.ansi_color(), "\x1b[1;31m");
    assert_eq!(TokenType::Text.ansi_color(), "\x1b[0m");
}

#[test]
fn test_css_classes() {
    assert_eq!(TokenType::SectionHeader.css_class(), "ass-section-header");
    assert_eq!(TokenType::FieldName.css_class(), "ass-field-name");
    assert_eq!(TokenType::OverrideTag.css_class(), "ass-override-tag");
    assert_eq!(TokenType::Text.css_class(), "ass-text");
}

#[test]
fn test_comment_events() {
    let mut ext = SyntaxHighlightExtension::new();
    let doc = EditorDocument::from_content(
        "[Events]\nComment: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,This is a comment",
    )
    .unwrap();

    let tokens = ext.tokenize_document(&doc).unwrap();

    // Should have Comment event type
    let event_token = tokens.iter().find(|t| t.token_type == TokenType::EventType);

    assert!(
        event_token.is_some(),
        "Should have found an EventType token"
    );
    assert_eq!(
        event_token.unwrap().semantic_info,
        Some("Comment".to_string())
    );
}
