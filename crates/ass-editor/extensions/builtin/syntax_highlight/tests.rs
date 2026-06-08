//! Unit tests for the syntax highlighting extension.

use super::*;
use crate::core::EditorDocument;
use crate::extensions::{EditorExtension, ExtensionCapability};
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn test_token_types() {
    assert_eq!(TokenType::SectionHeader.css_class(), "ass-section-header");
    assert_eq!(TokenType::OverrideTag.ansi_color(), "\x1b[1;31m");
}

#[test]
fn test_syntax_highlight_extension_creation() {
    let ext = SyntaxHighlightExtension::new();
    assert_eq!(ext.info().name, "syntax-highlight");
    assert!(ext
        .info()
        .has_capability(&ExtensionCapability::SyntaxHighlighting));
}

#[test]
fn test_simple_tokenization() {
    let mut ext = SyntaxHighlightExtension::new();
    let doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();

    let tokens = ext.tokenize_document(&doc).unwrap();
    assert!(!tokens.is_empty());

    // First token should be section header
    assert_eq!(tokens[0].token_type, TokenType::SectionHeader);
    assert_eq!(tokens[0].semantic_info, Some("Script Info".to_string()));
}

#[test]
fn test_config_schema() {
    let ext = SyntaxHighlightExtension::new();
    let schema = ext.config_schema();

    assert!(schema.contains_key("syntax.semantic_highlighting"));
    assert!(schema.contains_key("syntax.highlight_tags"));
}
