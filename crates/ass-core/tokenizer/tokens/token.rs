//! Zero-copy token produced by the ASS tokenizer.
//!
//! Defines the [`Token`] struct pairing a [`TokenType`] discriminant with a
//! `&'a str` span and source location for error reporting and editor integration.

use core::fmt;

use super::TokenType;

/// Token produced by ASS tokenizer with zero-copy span
///
/// Represents a lexical unit in ASS script with location information.
/// The span references the original source text to avoid allocations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a> {
    /// Token type discriminant
    pub token_type: TokenType,

    /// Zero-copy span referencing source text
    pub span: &'a str,

    /// Line number where token starts (1-based)
    pub line: usize,

    /// Column number where token starts (1-based)
    pub column: usize,
}

impl<'a> Token<'a> {
    /// Create new token with full location information
    #[must_use]
    pub const fn new(token_type: TokenType, span: &'a str, line: usize, column: usize) -> Self {
        Self {
            token_type,
            span,
            line,
            column,
        }
    }

    /// Get token length in characters
    #[must_use]
    pub fn len(&self) -> usize {
        self.span.chars().count()
    }

    /// Check if token is empty (should not happen in normal tokenization)
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.span.is_empty()
    }

    /// Get end column position
    #[must_use]
    pub fn end_column(&self) -> usize {
        self.column + self.len()
    }

    /// Check if this token represents whitespace
    #[must_use]
    pub const fn is_whitespace(&self) -> bool {
        matches!(self.token_type, TokenType::Whitespace)
    }

    /// Check if this token represents a delimiter
    #[must_use]
    pub const fn is_delimiter(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::Comma
                | TokenType::Colon
                | TokenType::SectionOpen
                | TokenType::SectionClose
                | TokenType::OverrideOpen
                | TokenType::OverrideClose
        )
    }

    /// Check if this token represents content (text, numbers, etc.)
    #[must_use]
    pub const fn is_content(&self) -> bool {
        matches!(
            self.token_type,
            TokenType::Text
                | TokenType::Number
                | TokenType::HexValue
                | TokenType::SectionName
                | TokenType::OverrideBlock
        )
    }

    /// Validate that span references valid UTF-8
    #[must_use]
    pub const fn validate_utf8(&self) -> bool {
        true
    }
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}@{}:{} '{}'",
            self.token_type, self.line, self.column, self.span
        )
    }
}
