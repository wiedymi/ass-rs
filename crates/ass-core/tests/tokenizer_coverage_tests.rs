//! Comprehensive tokenizer coverage tests
//!
//! Tests targeting specific uncovered code paths in the tokenizer module
//! to improve test coverage and ensure robust tokenization.

use ass_core::tokenizer::{AssTokenizer, TokenType};

#[cfg(test)]
mod tokenizer_coverage_tests {
    use super::*;

    #[test]
    fn test_section_header_transitions() {
        let input = "[Script Info]\nTitle: Test";
        let mut tokenizer = AssTokenizer::new(input);

        // Test section header - should trigger context change to SectionHeader
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::SectionHeader));

        // After section header, get section close
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::SectionClose));

        // Then get newline
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Newline));

        // Then get field content
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Text));
    }

    #[test]
    fn test_field_value_context_transitions() {
        let input = "Title: Test Value";
        let mut tokenizer = AssTokenizer::new(input);

        // Get the title text
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Text));

        // Test colon - should trigger context change to FieldValue
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Colon));

        // Skip whitespace
        let token = tokenizer.next_token().unwrap().unwrap();
        if matches!(token.token_type, TokenType::Whitespace) {
            // Get the actual value
            let token = tokenizer.next_token().unwrap().unwrap();
            assert!(matches!(token.token_type, TokenType::Text));
        } else {
            assert!(matches!(token.token_type, TokenType::Text));
        }
    }

    #[test]
    fn test_style_override_context_transitions() {
        let input = "{\\b1}text{\\b0}";
        let mut tokenizer = AssTokenizer::new(input);

        // Test first style override block
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::OverrideBlock));

        // Test first override close
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::OverrideClose));

        // Test text after override
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Text));

        // Test second override block
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::OverrideBlock));

        // Test second override close
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::OverrideClose));
    }

    #[test]
    fn test_comma_delimiter_handling() {
        let input = "Layer,Start,End,Style";
        let mut tokenizer = AssTokenizer::new(input);

        // Get first field
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Text));

        // Test comma delimiter
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Comma));

        // Get next field
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Text));
    }

    #[test]
    fn test_newline_context_reset() {
        let input = "Title: Test\nNext Line";
        let mut tokenizer = AssTokenizer::new(input);

        // Skip to colon to enter FieldValue context
        while let Some(token) = tokenizer.next_token().unwrap() {
            if matches!(token.token_type, TokenType::Colon) {
                break;
            }
        }

        // Skip to newline
        while let Some(token) = tokenizer.next_token().unwrap() {
            if matches!(token.token_type, TokenType::Newline) {
                break;
            }
        }

        // After newline, context should reset to Document
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Text));
    }

    #[test]
    fn test_carriage_return_handling() {
        let input = "Line1\r\nLine2";
        let mut tokenizer = AssTokenizer::new(input);

        // Get first line text
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Text));

        // Test carriage return handling
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Newline));

        // Get second line text
        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Text));
    }

    #[test]
    fn test_comment_detection_semicolon() {
        let input = "; This is a comment";
        let mut tokenizer = AssTokenizer::new(input);

        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Comment));
    }

    #[test]
    fn test_comment_detection_exclamation() {
        let input = "!: This is also a comment";
        let mut tokenizer = AssTokenizer::new(input);

        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Comment));
    }

    #[test]
    fn test_whitespace_handling_different_contexts() {
        let input = " \t spaces and tabs \t ";
        let mut tokenizer = AssTokenizer::new(input);

        let token = tokenizer.next_token().unwrap().unwrap();
        // Whitespace is typically tokenized as text in document context
        assert!(matches!(token.token_type, TokenType::Text));
    }

    #[test]
    fn test_empty_input_eof() {
        let input = "";
        let mut tokenizer = AssTokenizer::new(input);

        let result = tokenizer.next_token().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_position_advancement_validation() {
        // This should test the infinite loop protection
        let input = "normal text";
        let mut tokenizer = AssTokenizer::new(input);

        let mut token_count = 0;
        while let Some(_token) = tokenizer.next_token().unwrap() {
            token_count += 1;
            assert!(
                token_count <= 100,
                "Too many tokens, possible infinite loop"
            );
        }

        assert!(token_count > 0);
    }

    #[test]
    fn test_token_span_boundaries() {
        let input = "test";
        let mut tokenizer = AssTokenizer::new(input);

        let token = tokenizer.next_token().unwrap().unwrap();
        assert_eq!(token.span, "test");
        assert!(token.line > 0);
        assert!(token.column > 0);
    }

    #[test]
    fn test_tokenizer_reset_functionality() {
        let input = "test input";
        let mut tokenizer = AssTokenizer::new(input);

        // Get first token
        let first_token = tokenizer.next_token().unwrap().unwrap();

        // Reset tokenizer
        tokenizer.reset();

        // Should get same first token again
        let reset_token = tokenizer.next_token().unwrap().unwrap();
        assert_eq!(first_token.span, reset_token.span);
    }

    #[test]
    fn test_tokenizer_issues_collection() {
        let input = "test input";
        let tokenizer = AssTokenizer::new(input);

        // Should start with no issues
        assert!(tokenizer.issues().is_empty());
    }

    #[test]
    fn test_tokenizer_position_tracking() {
        let input = "test";
        let tokenizer = AssTokenizer::new(input);

        assert_eq!(tokenizer.position(), 0);
        assert_eq!(tokenizer.line(), 1);
        assert_eq!(tokenizer.column(), 1);
    }

    #[test]
    fn test_tokenize_all_method() {
        let input = "test input";
        let mut tokenizer = AssTokenizer::new(input);

        let tokens = tokenizer.tokenize_all().unwrap();
        assert!(!tokens.is_empty());

        // All tokens should have valid spans
        for token in &tokens {
            assert!(!token.span.is_empty());
        }
    }

    #[test]
    fn test_mixed_delimiters_and_content() {
        let input = "[Section]:value,field{tag}";
        let mut tokenizer = AssTokenizer::new(input);

        let mut token_types = Vec::new();
        while let Some(token) = tokenizer.next_token().unwrap() {
            token_types.push(token.token_type);
        }

        // Should have mixed token types
        assert!(token_types.len() > 3);
        assert!(token_types
            .iter()
            .any(|t| matches!(t, TokenType::SectionHeader)));
        assert!(token_types.iter().any(|t| matches!(t, TokenType::Colon)));
        assert!(token_types.iter().any(|t| matches!(t, TokenType::Comma)));
        assert!(token_types
            .iter()
            .any(|t| matches!(t, TokenType::OverrideBlock)));
    }

    #[test]
    fn test_unicode_content_handling() {
        let input = "测试 Unicode テスト";
        let mut tokenizer = AssTokenizer::new(input);

        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Text));
        // The token span should contain the full text until a delimiter
        assert!(token.span.contains("测试"));
    }

    #[test]
    fn test_nested_context_transitions() {
        let input = "[V4+ Styles]: {\\b1}test{\\b0}, value";
        let mut tokenizer = AssTokenizer::new(input);

        let mut contexts_seen = 0;
        while let Some(_token) = tokenizer.next_token().unwrap() {
            contexts_seen += 1;
        }

        assert!(contexts_seen > 8); // Should see multiple context transitions
    }

    #[test]
    fn test_malformed_section_handling() {
        let input = "[Unclosed Section\nNext line";
        let mut tokenizer = AssTokenizer::new(input);

        // Should handle unclosed section gracefully
        let mut token_count = 0;
        while let Some(_token) = tokenizer.next_token().unwrap() {
            token_count += 1;
            if token_count > 10 {
                break; // Prevent infinite loop in malformed input
            }
        }

        assert!(token_count > 0);
    }

    #[test]
    fn test_special_characters_in_text() {
        let input = "text with # @ $ % special chars";
        let mut tokenizer = AssTokenizer::new(input);

        let token = tokenizer.next_token().unwrap().unwrap();
        assert!(matches!(token.token_type, TokenType::Text));
        assert!(token.span.contains("text"));
    }

    #[test]
    fn test_line_column_tracking_multiline() {
        let input = "line1\nline2\nline3";
        let mut tokenizer = AssTokenizer::new(input);

        // First token should be at line 1
        let token1 = tokenizer.next_token().unwrap().unwrap();
        assert_eq!(token1.line, 1);

        // Skip to newline
        while let Some(token) = tokenizer.next_token().unwrap() {
            if matches!(token.token_type, TokenType::Newline) {
                break;
            }
        }

        // Next text token should be at line 2
        let token2 = tokenizer.next_token().unwrap().unwrap();
        assert_eq!(token2.line, 2);
    }
}
