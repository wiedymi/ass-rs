//! Extended tests for syntax highlighting extension

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(test)]
mod tests {
    use crate::core::EditorDocument;
    use crate::extensions::builtin::syntax_highlight::{SyntaxHighlightExtension, TokenType};
    use crate::extensions::{EditorExtension, ExtensionManager, ExtensionState};
    #[cfg(not(feature = "std"))]
    use alloc::collections::BTreeMap as HashMap;
    #[cfg(not(feature = "std"))]
    use alloc::{string::ToString, vec::Vec};
    #[cfg(feature = "std")]
    use std::collections::HashMap;

    #[test]
    fn test_section_header_tokenization() {
        let mut ext = SyntaxHighlightExtension::new();
        let doc = EditorDocument::from_content(
            "[Script Info]\n[V4+ Styles]\n[Events]\n[Unknown Section]",
        )
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
    fn test_cache_functionality() {
        let mut ext = SyntaxHighlightExtension::new();
        let doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();

        // First tokenization
        let tokens1 = ext.tokenize_document(&doc).unwrap();

        // Second tokenization should use cache
        let tokens2 = ext.tokenize_document(&doc).unwrap();

        assert_eq!(tokens1.len(), tokens2.len());

        // Clear cache
        ext.clear_cache();

        // Should tokenize again after cache clear
        let tokens3 = ext.tokenize_document(&doc).unwrap();
        assert_eq!(tokens1.len(), tokens3.len());
    }

    #[test]
    fn test_extension_lifecycle() {
        let mut ext = SyntaxHighlightExtension::new();
        let mut manager = ExtensionManager::new();
        let mut doc = EditorDocument::new();
        let mut context = manager
            .create_context("test".to_string(), Some(&mut doc))
            .unwrap();

        // Initialize
        assert_eq!(ext.state(), ExtensionState::Uninitialized);
        ext.initialize(&mut *context).unwrap();
        assert_eq!(ext.state(), ExtensionState::Active);

        // Execute command
        let result = ext
            .execute_command("syntax.highlight", &HashMap::new(), &mut *context)
            .unwrap();
        assert!(result.success);

        // Shutdown
        ext.shutdown(&mut *context).unwrap();
        assert_eq!(ext.state(), ExtensionState::Shutdown);
    }

    #[test]
    fn test_config_loading() {
        let mut ext = SyntaxHighlightExtension::new();
        let mut manager = ExtensionManager::new();

        // Set configuration
        manager.set_config("syntax.highlight_tags".to_string(), "false".to_string());
        manager.set_config("syntax.max_tokens".to_string(), "5000".to_string());

        let mut doc = EditorDocument::new();
        let mut context = manager
            .create_context("test".to_string(), Some(&mut doc))
            .unwrap();

        // Initialize should load config
        ext.initialize(&mut *context).unwrap();

        // Config should be loaded
        assert!(!ext.config.highlight_tags);
        assert_eq!(ext.config.max_tokens, 5000);
    }

    #[test]
    fn test_max_tokens_limit() {
        let mut ext = SyntaxHighlightExtension::new();
        ext.config.max_tokens = 3;

        let doc = EditorDocument::from_content(
            "[Script Info]\nTitle: Line 1\nAuthor: Line 2\nVersion: Line 3\nExtra: Line 4",
        )
        .unwrap();

        let tokens = ext.tokenize_document(&doc).unwrap();

        // Should stop at max_tokens
        assert_eq!(tokens.len(), 3);
    }

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
}
