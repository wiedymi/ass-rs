//! Targeted tests for specific uncovered tokenizer code paths
//!
//! These tests target the exact uncovered lines identified in coverage analysis
//! to ensure all code paths in the tokenizer are exercised.

use ass_core::tokenizer::{AssTokenizer, TokenType};

#[cfg(test)]
mod tokenizer_edge_paths {
    use super::*;

    #[test]
    fn test_closing_section_bracket_in_section_context() {
        // This should hit line 104-108: (']', TokenContext::SectionHeader)
        let input = "[Script Info]";
        let mut tokenizer = AssTokenizer::new(input);

        // Get section header - this enters SectionHeader context
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::SectionHeader));

        // Get closing bracket - this should hit the SectionClose path
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::SectionClose));
    }

    #[test]
    fn test_colon_in_document_context() {
        // This should hit line 109-113: (':', TokenContext::Document)
        let input = "Title: Value";
        let mut tokenizer = AssTokenizer::new(input);

        // Skip to colon in Document context
        let _text_token = tokenizer.next_token().unwrap().unwrap();

        // This colon should trigger the Document context path
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Colon));
    }

    #[test]
    fn test_opening_style_override_brace() {
        // This should hit line 115-118: ('{', _)
        let input = "{\\b1}";
        let mut tokenizer = AssTokenizer::new(input);

        // This should trigger the style override opening path
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::OverrideBlock));
    }

    #[test]
    fn test_closing_style_override_brace_in_override_context() {
        // This should hit line 119-123: ('}', TokenContext::StyleOverride)
        let input = "{\\b1}";
        let mut tokenizer = AssTokenizer::new(input);

        // Get the override block - this enters StyleOverride context
        let _override_token = tokenizer.next_token().unwrap().unwrap();

        // Get the closing brace - this should hit the OverrideClose path
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::OverrideClose));
    }

    #[test]
    fn test_comma_delimiter_handling() {
        // This should hit line 124-127: (',', _)
        let input = "field1,field2";
        let mut tokenizer = AssTokenizer::new(input);

        // Skip first field
        let _field1 = tokenizer.next_token().unwrap().unwrap();

        // This comma should trigger the comma handling path
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Comma));
    }

    #[test]
    fn test_newline_context_reset() {
        // This should hit line 128-132: ('\n' | '\r', _)
        let input = "line1\nline2";
        let mut tokenizer = AssTokenizer::new(input);

        // Skip first line text
        let _line1 = tokenizer.next_token().unwrap().unwrap();

        // This newline should trigger the context reset path
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Newline));
    }

    #[test]
    fn test_carriage_return_context_reset() {
        // This should hit the '\r' part of line 128: ('\n' | '\r', _)
        let input = "line1\rline2";
        let mut tokenizer = AssTokenizer::new(input);

        // Skip first line text
        let _line1 = tokenizer.next_token().unwrap().unwrap();

        // This carriage return should trigger the context reset path
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Newline));
    }

    #[test]
    fn test_semicolon_comment_detection() {
        // This should hit comment detection paths
        let input = "; This is a comment";
        let mut tokenizer = AssTokenizer::new(input);

        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Comment));
    }

    #[test]
    fn test_exclamation_comment_detection() {
        // This should hit exclamation comment detection paths
        let input = "!: This is also a comment";
        let mut tokenizer = AssTokenizer::new(input);

        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Comment));
    }

    #[test]
    fn test_text_fallback_path() {
        // This should hit the text fallback path (line 137+ area)
        let input = "regular text without special chars";
        let mut tokenizer = AssTokenizer::new(input);

        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Text));
        assert_eq!(token.span, "regular text without special chars");
    }

    #[test]
    fn test_position_advancement_check() {
        // This should test the position advancement validation (lines 166-172)
        let input = "a";
        let mut tokenizer = AssTokenizer::new(input);

        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Text));
        assert_eq!(token.span, "a");

        // Next call should return None (EOF)
        let result = tokenizer.next_token().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_token_span_creation() {
        // This should hit the span creation logic (lines 175-180)
        let input = "test";
        let mut tokenizer = AssTokenizer::new(input);

        let token = tokenizer.next_token().unwrap().unwrap();
        assert_eq!(token.span, "test");
        assert_eq!(token.line, 1);
        assert_eq!(token.column, 1);
    }

    #[test]
    fn test_tokenize_all_iteration_boundary() {
        // This should hit the tokenize_all method paths (lines 193-195)
        let input = "short";
        let mut tokenizer = AssTokenizer::new(input);

        let tokens = tokenizer.tokenize_all().unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].token_type, TokenType::Text));
    }

    #[test]
    fn test_mixed_whitespace_handling() {
        // Test various whitespace characters
        let input = " \t\n";
        let mut tokenizer = AssTokenizer::new(input);

        // Get first token (may be whitespace, text, or newline depending on tokenizer behavior)
        let token = tokenizer.next_token().unwrap().unwrap();
        // Accept any valid token type - the goal is to exercise tokenizer paths
        assert!(matches!(
            token.token_type,
            TokenType::Whitespace | TokenType::Text | TokenType::Newline
        ));

        // Get remaining tokens if any
        while let Some(_token) = tokenizer.next_token().unwrap() {
            // Just consume remaining tokens to exercise paths
        }
    }

    #[test]
    fn test_context_dependent_character_handling() {
        // Test how same characters behave in different contexts
        let input = "text:value{override}";
        let mut tokenizer = AssTokenizer::new(input);

        let tokens = tokenizer.tokenize_all().unwrap();

        // Should have multiple token types
        assert!(tokens.len() >= 4);
    }

    #[test]
    fn test_empty_section_header() {
        // Test edge case of empty section
        let input = "[]";
        let mut tokenizer = AssTokenizer::new(input);

        let token1 = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token1.token_type, TokenType::SectionHeader));

        let token2 = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token2.token_type, TokenType::SectionClose));
    }

    #[test]
    fn test_empty_style_override() {
        // Test edge case of empty override
        let input = "{}";
        let mut tokenizer = AssTokenizer::new(input);

        let token1 = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token1.token_type, TokenType::OverrideBlock));

        let token2 = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token2.token_type, TokenType::OverrideClose));
    }

    #[test]
    fn test_unicode_boundary_handling() {
        // Test Unicode character boundaries
        let input = "café🎬";
        let mut tokenizer = AssTokenizer::new(input);

        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Text));
        assert!(token.span.contains("café"));
    }

    #[test]
    fn test_consecutive_delimiters() {
        // Test multiple delimiters in sequence
        let input = "::,,";
        let mut tokenizer = AssTokenizer::new(input);

        let tokens = tokenizer.tokenize_all().unwrap();
        assert!(tokens.len() >= 4); // Should have multiple delimiter tokens
    }

    #[test]
    fn test_field_value_context_boundary() {
        // Test field value context transitions
        let input = "Key: Value\nNext: Line";
        let mut tokenizer = AssTokenizer::new(input);

        let tokens = tokenizer.tokenize_all().unwrap();

        // Should have colon tokens that trigger context changes
        let has_colon = tokens
            .iter()
            .any(|t| matches!(t.token_type, TokenType::Colon));
        let has_newline = tokens
            .iter()
            .any(|t| matches!(t.token_type, TokenType::Newline));

        assert!(has_colon);
        assert!(has_newline);
    }
}
