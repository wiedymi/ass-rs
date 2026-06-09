//! Token, delimiter-type, and position component-level tokenizer tests.

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn tokenizer_token_validation() {
    use crate::tokenizer::tokens::{Token, TokenType};

    // Test token validation and properties
    let token = Token::new(TokenType::Text, "test", 1, 1);

    assert_eq!(token.len(), 4);
    assert!(!token.is_empty());
    assert_eq!(token.end_column(), 5);
    assert!(!token.is_whitespace());
    assert!(!token.is_delimiter());
    assert!(token.is_content());
    assert!(token.validate_utf8());

    // Test empty token
    let empty_token = Token::new(TokenType::Text, "", 1, 1);
    assert!(empty_token.is_empty());
    assert_eq!(empty_token.len(), 0);
    assert_eq!(empty_token.end_column(), 1);
}

#[test]
fn tokenizer_delimiter_type_comprehensive() {
    use crate::tokenizer::tokens::DelimiterType;

    // Test all delimiter types
    let delimiters = vec![
        DelimiterType::FieldSeparator,
        DelimiterType::ValueSeparator,
        DelimiterType::SectionBoundary,
        DelimiterType::OverrideBoundary,
        DelimiterType::CommentMarker,
        DelimiterType::LineTerminator,
        DelimiterType::DrawingSeparator,
        DelimiterType::TimeSeparator,
        DelimiterType::ColorSeparator,
    ];

    for delimiter in delimiters {
        let chars = delimiter.chars();
        assert!(
            !chars.is_empty(),
            "Delimiter should have characters: {delimiter:?}"
        );

        // Test that each character matches the delimiter
        for &ch in chars {
            assert!(
                delimiter.matches(ch),
                "Character '{ch}' should match delimiter: {delimiter:?}"
            );
        }
    }
}

#[test]
fn tokenizer_position_unicode_handling() {
    use crate::tokenizer::tokens::TokenPosition;

    let mut pos = TokenPosition::new(0, 1, 1);

    // Test advancing by Unicode string
    pos = pos.advance_by_str("Hello, 世界!");

    // Position should advance by byte length, not character count
    assert!(pos.offset > 10); // Should be more than ASCII length due to Unicode
    assert_eq!(pos.line, 1);
    assert!(pos.column > 10);

    // Test with newlines
    let mut pos_newline = TokenPosition::new(0, 1, 1);
    pos_newline = pos_newline.advance_by_str("line1\nline2");
    assert_eq!(pos_newline.line, 2);
    assert_eq!(pos_newline.column, 6); // "line2" = 5 chars + 1 for 1-based indexing
}
