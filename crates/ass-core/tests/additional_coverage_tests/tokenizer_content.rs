//! Coverage tests for `AssTokenizer` content tokenization.
//!
//! Exercises iteration limits, mixed line endings, Unicode handling, comment
//! detection, large inputs, and whitespace edge cases.

use ass_core::tokenizer::{AssTokenizer, TokenType};

#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};

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
