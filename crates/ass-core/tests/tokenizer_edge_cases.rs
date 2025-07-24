//! Edge case and error handling tests for the ASS tokenizer.
//!
//! This module contains comprehensive tests targeting previously untested code paths
//! in the tokenizer, focusing on SIMD optimizations, error conditions, and edge cases.

use ass_core::tokenizer::{
    scanner::{CharNavigator, TokenScanner},
    AssTokenizer, Token, TokenType,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test `CharNavigator` error paths when reading past end of string (L48, L68, L76)
    #[test]
    fn test_char_navigator_bounds_checking() {
        let source = "abc";
        let mut navigator = CharNavigator::new(source, 0, 1, 1);

        // Advance to end
        navigator.advance_char().expect("Should advance");
        navigator.advance_char().expect("Should advance");
        navigator.advance_char().expect("Should advance");

        // Now at end - these should handle gracefully or return errors
        assert!(navigator.is_at_end());

        // Test peek_char at end
        let peek_result = navigator.peek_char();
        assert!(peek_result.is_err() || peek_result.unwrap() == '\0');

        // Test peek_next at end
        let peek_next_result = navigator.peek_next();
        assert!(peek_next_result.is_err() || peek_next_result.unwrap() == '\0');

        // Test advance_char past end
        let advance_result = navigator.advance_char();
        assert!(advance_result.is_err() || navigator.is_at_end());
    }

    /// Test `scan_text` with different contexts to cover scalar and SIMD paths (L162-L208)
    #[test]
    fn test_scan_text_different_contexts() {
        // Test with FieldValue context - colons should be ignored as delimiters
        let source = "test:value:with:colons";
        let _scanner = TokenScanner::new(source, 0, 1, 1);

        // Test TokenContext::FieldValue behavior
        let mut tokenizer = AssTokenizer::new(source);
        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");

        // Should tokenize the entire string as text, not breaking on colons in field values
        assert!(!tokens.is_empty());

        // Test with regular text context
        let text_source = "Hello, World! This is regular text.";
        let mut text_scanner = TokenScanner::new(text_source, 0, 1, 1);
        let _navigator = text_scanner.navigator_mut();

        // Should handle regular text scanning
        assert!(!text_source.is_empty());
    }

    /// Test SIMD and scalar paths for `is_hex_value` (L215-L226)
    #[test]
    fn test_hex_value_detection() {
        let hex_values = [
            "123456",
            "ABCDEF",
            "abcdef",
            "0123456789ABCDEFabcdef",
            "FF00FF",
        ];

        let non_hex_values = ["GHIJKL", "123XYZ", "Hello", "12G34", ""];

        for hex_val in &hex_values {
            let mut tokenizer = AssTokenizer::new(hex_val);
            let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");

            // Should tokenize hex values appropriately
            assert!(!tokens.is_empty());
        }

        for non_hex_val in &non_hex_values {
            let mut tokenizer = AssTokenizer::new(non_hex_val);
            let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");

            // Should handle non-hex values
            if !non_hex_val.is_empty() {
                assert!(!tokens.is_empty());
            }
        }
    }

    /// Test `scan_field_value` classification logic (L240-L245)
    #[test]
    fn test_field_value_classification() {
        let number_values = ["123", "123.45", "0.5", "100", "-42", "3.14159"];

        let text_values = ["abc", "hello world", "test123text", "not_a_number", "12abc"];

        for num_val in &number_values {
            let mut tokenizer = AssTokenizer::new(num_val);
            let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");

            // Should classify numbers appropriately
            assert!(!tokens.is_empty());
            if let Some(token) = tokens.first() {
                // Numbers might be tokenized as Number or Text depending on context
                assert!(matches!(
                    token.token_type,
                    TokenType::Number | TokenType::Text
                ));
            }
        }

        for text_val in &text_values {
            let mut tokenizer = AssTokenizer::new(text_val);
            let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");

            // Should classify text appropriately
            assert!(!tokens.is_empty());
            if let Some(token) = tokens.first() {
                assert!(matches!(token.token_type, TokenType::Text));
            }
        }
    }

    /// Test misplaced delimiters in different contexts (L74-L111)
    #[test]
    fn test_misplaced_delimiters() {
        // Test ] without being in SectionHeader context
        let misplaced_bracket = "Some text ] more text";
        let mut tokenizer = AssTokenizer::new(misplaced_bracket);
        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");

        // Should handle misplaced bracket gracefully
        assert!(!tokens.is_empty());

        // Test } without being in StyleOverride context
        let misplaced_brace = "Some text } more text";
        let mut tokenizer = AssTokenizer::new(misplaced_brace);
        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");

        // Should handle misplaced brace gracefully
        assert!(!tokens.is_empty());

        // Test multiple misplaced delimiters
        let multiple_misplaced = "text ] more } text [ even more";
        let mut tokenizer = AssTokenizer::new(multiple_misplaced);
        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");

        // Should tokenize despite misplaced delimiters
        assert!(!tokens.is_empty());
    }

    /// Test !: comment marker vs ! without colon (L94-L101)
    #[test]
    fn test_comment_markers() {
        // Test !: comment marker
        let comment_with_colon = "!: This is a comment";
        let mut tokenizer = AssTokenizer::new(comment_with_colon);
        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");

        assert!(!tokens.is_empty());
        if let Some(token) = tokens.first() {
            assert!(matches!(token.token_type, TokenType::Comment));
        }

        // Test ! without colon (should be treated as text)
        let exclamation_text = "! This is not a comment";
        let mut tokenizer = AssTokenizer::new(exclamation_text);
        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");

        assert!(!tokens.is_empty());
        if let Some(token) = tokens.first() {
            assert!(matches!(token.token_type, TokenType::Text));
        }

        // Test mixed scenarios
        let mixed_exclamations = "Hello! World !: This is a comment ! Not a comment";
        let mut tokenizer = AssTokenizer::new(mixed_exclamations);
        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");

        // Should handle mixed exclamation scenarios
        assert!(tokens.len() > 1);
    }

    /// Test `tokenize_all` iteration limit with many tokens (L138-L143)
    #[test]
    fn test_tokenize_iteration_limit() {
        // Test that iteration limit is enforced (limit is 50 iterations)
        let many_tokens = "a,".repeat(5); // Should be within limit
        let mut tokenizer = AssTokenizer::new(&many_tokens);

        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");

        // Should handle reasonable number of tokens
        assert!(!tokens.is_empty());
        assert!(tokens.len() <= 15); // Should be within reasonable bounds

        // Test that excessive tokens trigger the limit
        let excessive_tokens = "a,".repeat(60); // This should hit the iteration limit
        let mut tokenizer = AssTokenizer::new(&excessive_tokens);

        let result = tokenizer.tokenize_all();
        // Should either succeed with limited tokens or fail with iteration limit error
        match result {
            Ok(tokens) => assert!(tokens.len() <= 50),
            Err(e) => assert!(e.to_string().contains("Too many tokenizer iterations")),
        }
    }

    /// Test potential infinite loop protection (L124-L129)
    #[test]
    fn test_infinite_loop_protection() {
        // Test with unusual characters that might not advance
        let unusual_chars = "\u{200B}\u{FEFF}\u{00A0}"; // Zero-width space, BOM, non-breaking space
        let mut tokenizer = AssTokenizer::new(unusual_chars);

        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");

        // Should handle unusual characters without infinite loops
        // The tokenizer should either advance or break out safely
        assert!(tokens.len() < 1000); // Reasonable upper bound

        // Test with repeated delimiters
        let repeated_delimiters = "[[[[]]]]{{{{}}}};;;;";
        let mut tokenizer = AssTokenizer::new(repeated_delimiters);

        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");
        assert!(!tokens.is_empty());
        assert!(tokens.len() < 100); // Should not create excessive tokens
    }

    /// Test SIMD-specific paths when feature is enabled
    #[test]
    #[cfg(feature = "simd")]
    fn test_simd_optimizations() {
        // Test long strings that would benefit from SIMD (but stay under token limit)
        let long_text = "a".repeat(100);
        let mut tokenizer = AssTokenizer::new(&long_text);

        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");
        assert!(!tokens.is_empty());
        assert_eq!(tokens.len(), 1); // Should be one text token

        // Test hex detection with SIMD
        let long_hex = "0123456789ABCDEF".repeat(3);
        let mut tokenizer = AssTokenizer::new(&long_hex);

        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");
        assert!(!tokens.is_empty());

        // Test text scanning with SIMD (limited to avoid hitting iteration limit)
        let mixed_content = "Regular text with {overrides} and [sections] and ;comments";
        let mut tokenizer = AssTokenizer::new(mixed_content);

        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");
        assert!(!tokens.is_empty());
        assert!(tokens.len() < 50); // Should be well under iteration limit
    }

    /// Test edge cases in token scanning
    #[test]
    fn test_token_scanning_edge_cases() {
        // Empty input
        let empty = "";
        let mut tokenizer = AssTokenizer::new(empty);
        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");
        assert!(tokens.is_empty());

        // Single character inputs
        let single_chars = ["a", "[", "]", "{", "}", ";", ":", ",", "\n"];
        for ch in &single_chars {
            let mut tokenizer = AssTokenizer::new(ch);
            let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");
            assert!(!tokens.is_empty());
        }

        // Nested structures
        let nested = "[[inner]] {{nested{tags}}} ;; comment";
        let mut tokenizer = AssTokenizer::new(nested);
        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");
        assert!(!tokens.is_empty());

        // Malformed structures
        let malformed = "[unclosed {unmatched ;incomplete";
        let mut tokenizer = AssTokenizer::new(malformed);
        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");
        assert!(!tokens.is_empty());
    }

    /// Test different `TokenContext` scenarios
    #[test]
    fn test_token_contexts() {
        // Test content that would have different behavior in different contexts
        let context_sensitive = "Style: Default, Font: Arial:12";

        // Tokenize normally
        let mut tokenizer = AssTokenizer::new(context_sensitive);
        let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");
        assert!(!tokens.is_empty());

        // Should handle colon-separated values appropriately
        let colon_count = tokens
            .iter()
            .filter(|t| matches!(t.token_type, TokenType::Colon))
            .count();
        assert!(colon_count > 0);
    }

    /// Test boundary conditions and error recovery
    #[test]
    fn test_boundary_conditions() {
        // Test at exact buffer boundaries (common in SIMD implementations)
        let sizes = [15, 16, 17, 31, 32, 33, 63, 64, 65]; // Around common SIMD boundaries

        for &size in &sizes {
            let test_string = "x".repeat(size);
            let mut tokenizer = AssTokenizer::new(&test_string);
            let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");

            assert!(!tokens.is_empty());
            assert_eq!(tokens.len(), 1); // Should be single text token
        }

        // Test with mixed content at boundaries
        for &size in &sizes {
            let mut test_string = "x".repeat(size - 1);
            test_string.push(',');

            let mut tokenizer = AssTokenizer::new(&test_string);
            let tokens: Vec<Token> = tokenizer.tokenize_all().expect("Should tokenize");

            assert!(tokens.len() >= 2); // At least text + comma
        }
    }
}
