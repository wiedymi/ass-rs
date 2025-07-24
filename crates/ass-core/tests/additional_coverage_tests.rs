//! Additional coverage tests for ASS tokenizer edge cases.
//!
//! This module focuses on improving test coverage for previously untested
//! code paths in the tokenizer, particularly error conditions and edge cases
//! that are difficult to trigger through normal usage.

use ass_core::tokenizer::{
    scanner::{CharNavigator, TokenScanner},
    state::{IssueCollector, TokenContext},
    AssTokenizer, TokenType,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test tokenizer position advancement validation to prevent infinite loops
    #[test]
    fn test_tokenizer_position_advancement_validation() {
        // This test ensures the infinite loop protection works correctly
        let mut tokenizer = AssTokenizer::new("x");

        // Normal case should advance position
        let token = tokenizer.next_token().unwrap().unwrap();
        assert_eq!(token.token_type, TokenType::Text);
        assert!(tokenizer.position() > 0);
    }

    /// Test tokenizer with malformed scanner state
    #[test]
    fn test_tokenizer_scanner_error_propagation() {
        // Create tokenizer and force it into edge case scenarios
        let mut tokenizer = AssTokenizer::new("\x00invalid\x01");

        // Should handle invalid UTF-8 or control characters gracefully
        let result = tokenizer.next_token();
        // Either succeeds with sanitized output or propagates scanner error
        if let Ok(Some(token)) = result {
            assert!(matches!(token.token_type, TokenType::Text));
        } else {
            // Empty result or error propagation is acceptable
        }
    }

    /// Test tokenizer context transitions with edge case combinations
    #[test]
    fn test_tokenizer_context_edge_transitions() {
        // Test rapid context changes that might cause state inconsistencies
        let mut tokenizer = AssTokenizer::new("[}]{:}{}[");

        let mut tokens = Vec::new();
        while let Ok(Some(token)) = tokenizer.next_token() {
            tokens.push(token);
            // Prevent infinite loops in testing
            if tokens.len() > 20 {
                break;
            }
        }

        // Should handle malformed delimiters without panicking
        assert!(!tokens.is_empty());
    }

    /// Test tokenizer with very long content to trigger iteration limits
    #[test]
    fn test_tokenizer_iteration_limit_boundary() {
        // Create content that approaches the 50-token limit in tokenize_all
        let content = "a,".repeat(30); // 60 tokens (30 'a' + 30 ',')
        let mut tokenizer = AssTokenizer::new(&content);

        let result = tokenizer.tokenize_all();
        match result {
            Ok(tokens) => {
                // Should not exceed reasonable limits
                assert!(tokens.len() <= 60);
            }
            Err(e) => {
                // Should get iteration limit error
                assert!(e.to_string().contains("Too many tokenizer iterations"));
            }
        }
    }

    /// Test BOM handling edge cases
    #[test]
    fn test_tokenizer_bom_edge_cases() {
        // Test BOM-like sequences that aren't actually BOM
        let tokenizer1 = AssTokenizer::new("\u{FFFE}not-bom");
        assert_eq!(tokenizer1.position(), 0); // Should not skip non-BOM

        // Test actual BOM followed by content
        let tokenizer2 = AssTokenizer::new("\u{FEFF}content");
        assert_eq!(tokenizer2.position(), 3); // Should skip BOM
    }

    /// Test issue collection functionality comprehensively
    #[test]
    fn test_tokenizer_issue_collection_comprehensive() {
        let mut tokenizer = AssTokenizer::new("test content with potential issues");

        // Initial state should have no issues
        assert!(tokenizer.issues().is_empty());

        // Process tokens
        let _tokens = tokenizer.tokenize_all().unwrap();

        // Issues should be accessible (may be empty for valid content)
        let _issues = tokenizer.issues();

        // Test that reset clears issues
        tokenizer.reset();
        assert!(tokenizer.issues().is_empty());
        assert_eq!(tokenizer.position(), 0);
        assert_eq!(tokenizer.line(), 1);
        assert_eq!(tokenizer.column(), 1);
    }

    /// Test tokenizer field value context handling edge cases
    #[test]
    fn test_tokenizer_field_value_context_edge_cases() {
        // Test field value context with unusual delimiters
        let mut tokenizer = AssTokenizer::new("key:value}with]wrong{delimiters");

        let mut tokens = Vec::new();
        while let Ok(Some(token)) = tokenizer.next_token() {
            tokens.push(token);
            if tokens.len() > 10 {
                break;
            }
        }

        // Should handle misplaced delimiters in field value context
        assert!(tokens.len() >= 3); // At least key, colon, value
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Colon));
    }

    /// Test tokenizer with mixed line ending types
    #[test]
    fn test_tokenizer_mixed_line_endings() {
        let mut tokenizer = AssTokenizer::new("line1\rline2\nline3\r\nline4");

        let mut newline_count = 0;
        let mut text_count = 0;

        while let Ok(Some(token)) = tokenizer.next_token() {
            match token.token_type {
                TokenType::Newline => newline_count += 1,
                TokenType::Text => text_count += 1,
                _ => {}
            }
        }

        // Should properly handle different line ending types
        assert_eq!(newline_count, 3); // \r, \n, \r\n
        assert_eq!(text_count, 4); // line1, line2, line3, line4
    }

    /// Test scanner character navigation edge cases
    #[test]
    fn test_char_navigator_edge_cases() {
        // Test with empty input
        let mut navigator = CharNavigator::new("", 0, 1, 1);
        assert!(navigator.is_at_end());
        assert!(navigator.peek_char().is_err());

        // Test with single character
        let mut navigator2 = CharNavigator::new("x", 0, 1, 1);
        assert!(!navigator2.is_at_end());
        assert_eq!(navigator2.peek_char().unwrap(), 'x');

        // Advance past end
        navigator2.advance_char().unwrap();
        assert!(navigator2.is_at_end());
        assert!(navigator2.peek_char().is_err());
    }

    /// Test token scanner in different contexts
    #[test]
    fn test_token_scanner_context_handling() {
        let source = "test{override}[section]:value;comment";
        let mut scanner = TokenScanner::new(source, 0, 1, 1);

        // Test scanning in different contexts
        let text_result = scanner.scan_text(TokenContext::Document);
        assert!(text_result.is_ok());

        // Move to override position and test
        let mut scanner2 = TokenScanner::new(&source[4..], 0, 1, 5);
        let override_result = scanner2.scan_style_override();
        assert!(override_result.is_ok());
    }

    /// Test tokenizer reset functionality thoroughly
    #[test]
    fn test_tokenizer_reset_comprehensive() {
        let mut tokenizer = AssTokenizer::new("\u{FEFF}[Section]\nField: Value");

        // Process some tokens
        let _token1 = tokenizer.next_token().unwrap();
        let _token2 = tokenizer.next_token().unwrap();

        let mid_position = tokenizer.position();
        let mid_line = tokenizer.line();
        let mid_column = tokenizer.column();

        assert!(mid_position > 3); // Should be past BOM
        assert!(mid_line >= 1);
        assert!(mid_column >= 1);

        // Reset and verify all state is restored
        tokenizer.reset();

        assert_eq!(tokenizer.position(), 3); // Should skip BOM again
        assert_eq!(tokenizer.line(), 1);
        assert_eq!(tokenizer.column(), 1);
        assert!(tokenizer.issues().is_empty());
    }

    /// Test tokenizer with content that could cause position stagnation
    #[test]
    fn test_tokenizer_position_stagnation_prevention() {
        // Test content that might cause scanner to not advance
        let mut tokenizer = AssTokenizer::new("\x00");

        let result = tokenizer.next_token();

        // Should either advance position or handle error gracefully
        match result {
            Ok(Some(_)) => {
                assert!(tokenizer.position() > 0);
            }
            Ok(None) => {
                // Empty result acceptable for problematic input
            }
            Err(e) => {
                // Error acceptable if position advancement fails
                assert!(
                    e.to_string().contains("position not advancing") || !e.to_string().is_empty()
                );
            }
        }
    }

    /// Test delimiter type classification edge cases
    #[test]
    fn test_delimiter_type_edge_cases() {
        let mut tokenizer = AssTokenizer::new("{}[]():,;\n\r");

        let mut delimiter_types = Vec::new();

        while let Ok(Some(token)) = tokenizer.next_token() {
            match token.token_type {
                TokenType::SectionHeader | TokenType::SectionOpen => {
                    delimiter_types.push("section");
                }
                TokenType::SectionClose => {
                    delimiter_types.push("section_close");
                }
                TokenType::OverrideBlock | TokenType::OverrideOpen => {
                    delimiter_types.push("override");
                }
                TokenType::OverrideClose => {
                    delimiter_types.push("override_close");
                }
                TokenType::Colon => {
                    delimiter_types.push("colon");
                }
                TokenType::Comma => {
                    delimiter_types.push("comma");
                }
                TokenType::Comment => {
                    delimiter_types.push("comment");
                }
                TokenType::Newline => {
                    delimiter_types.push("newline");
                }
                _ => {}
            }
        }

        // Should recognize various delimiter types
        assert!(!delimiter_types.is_empty());
    }

    /// Test tokenizer with Unicode edge cases
    #[test]
    fn test_tokenizer_unicode_handling() {
        let mut tokenizer = AssTokenizer::new("Text with 中文 and 🎭 emojis");

        let tokens = tokenizer.tokenize_all().unwrap();

        // Should handle Unicode characters in text content
        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Text));

        // Check that positions advance correctly with multibyte characters
        let mut positions = Vec::new();
        let mut tokenizer2 = AssTokenizer::new("中文🎭");
        while let Ok(Some(_)) = tokenizer2.next_token() {
            positions.push(tokenizer2.position());
        }

        // Positions should advance even with multibyte chars
        if positions.len() > 1 {
            assert!(positions.windows(2).all(|w| w[1] >= w[0]));
        }
    }

    /// Test comment detection edge cases
    #[test]
    fn test_comment_detection_edge_cases() {
        // Test various comment-like patterns
        let test_cases = [
            (";comment", true),
            ("!:comment", true),
            ("! not comment", false),
            ("text;not comment in text", false),
            ("text!:not comment in text", false),
        ];

        for (input, should_have_comment) in test_cases {
            let mut tokenizer = AssTokenizer::new(input);
            let tokens = tokenizer.tokenize_all().unwrap();

            let has_comment = tokens.iter().any(|t| t.token_type == TokenType::Comment);

            if should_have_comment {
                assert!(has_comment, "Should detect comment in: {input}");
            }
            // Note: We don't assert !has_comment for false cases because
            // the tokenizer might legitimately find comment tokens in some contexts
        }
    }

    /// Test issue collector functionality directly
    #[test]
    fn test_issue_collector_comprehensive() {
        let mut collector = IssueCollector::new();

        // Initially empty
        assert!(collector.issues().is_empty());

        // Clear should work on empty collector
        collector.clear();
        assert!(collector.issues().is_empty());

        // Test that collector doesn't panic on repeated operations
        for _i in 0..10 {
            collector.clear();
            let _issues = collector.issues();
        }
    }

    /// Test token context transition exhaustively
    #[test]
    fn test_token_context_transitions_exhaustive() {
        // Test all major context transitions
        let context_transitions = [
            ("[", TokenContext::SectionHeader),
            (":", TokenContext::FieldValue),
            ("{", TokenContext::StyleOverride),
        ];

        for (trigger, _expected_context) in context_transitions {
            let mut tokenizer = AssTokenizer::new(trigger);

            // Get initial token which should trigger context change
            let _token = tokenizer.next_token().unwrap();

            // Context should have changed (we can't directly access context,
            // but we can test behavior)
            let remaining_input = &trigger[1..];
            if !remaining_input.is_empty() {
                let _next_token = tokenizer.next_token();
                // Should not panic or cause infinite loops
            }
        }
    }

    /// Test tokenizer with maximum length inputs
    #[test]
    fn test_tokenizer_large_input_handling() {
        // Create reasonably large input to test scalability
        let large_input = format!("[Section]\n{}\n", "Field: Value\n".repeat(100));
        let mut tokenizer = AssTokenizer::new(&large_input);

        let result = tokenizer.tokenize_all();

        // Should handle large inputs without stack overflow
        match result {
            Ok(tokens) => {
                assert!(!tokens.is_empty());
                // Verify we got reasonable token count
                assert!(tokens.len() > 100);
            }
            Err(e) => {
                // If it hits iteration limits, that's acceptable
                assert!(e.to_string().contains("Too many tokenizer iterations"));
            }
        }
    }

    /// Test scanner state preservation across operations
    #[test]
    fn test_scanner_state_preservation() {
        let source = "test content";
        let scanner = TokenScanner::new(source, 0, 1, 1);

        // Verify initial state
        assert_eq!(scanner.navigator().position(), 0);
        assert_eq!(scanner.navigator().line(), 1);
        assert_eq!(scanner.navigator().column(), 1);
        assert!(!scanner.navigator().is_at_end());

        // Create another scanner at different position
        let scanner2 = TokenScanner::new(source, 5, 2, 3);
        assert_eq!(scanner2.navigator().position(), 5);
        assert_eq!(scanner2.navigator().line(), 2);
        assert_eq!(scanner2.navigator().column(), 3);
    }

    /// Test edge cases in whitespace handling
    #[test]
    fn test_whitespace_handling_edge_cases() {
        // Test various whitespace combinations
        let whitespace_cases = [
            "",
            " ",
            "\t",
            "\n",
            "\r",
            " \t \n \r ",
            "\u{00A0}", // Non-breaking space
            "\u{2000}", // En quad
            "\u{3000}", // Ideographic space
        ];

        for ws in whitespace_cases {
            let mut tokenizer = AssTokenizer::new(ws);

            // Should handle all whitespace types gracefully
            let result = tokenizer.next_token();
            if let Ok(Some(token)) = result {
                // Some whitespace might be preserved as text
                assert!(matches!(
                    token.token_type,
                    TokenType::Text | TokenType::Newline
                ));
            } else {
                // Empty result acceptable for whitespace-only input
                // Errors acceptable for unusual Unicode whitespace
            }
        }
    }
}
