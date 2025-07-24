//! Token definitions for ASS script tokenization
//!
//! Provides zero-copy token types for lexical analysis of ASS subtitle scripts.
//! All tokens maintain references to the original source text via lifetime parameters.
//!
//! # Token Design
//!
//! - Zero-copy via `&'a str` spans referencing source
//! - Location tracking for error reporting and editor integration
//! - Semantic token types for context-aware parsing
//! - Efficient discriminant matching for hot parsing paths
//!
//! # Example
//!
//! ```rust
//! use ass_core::tokenizer::{Token, TokenType};
//!
//! let source = "[Script Info]";
//! // Token would be created by tokenizer with span referencing source
//! let token = Token {
//!     token_type: TokenType::SectionHeader,
//!     span: &source[0..12], // "[Script Info"
//!     line: 1,
//!     column: 1,
//! };
//! ```

use core::fmt;

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

/// Token type discriminant for efficient pattern matching
///
/// Represents the semantic type of a lexical unit in ASS scripts.
/// Ordered roughly by parsing frequency for optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    /// Plain text content
    Text,

    /// Numeric value (integer or float)
    Number,

    /// Hexadecimal value (colors, etc.)
    HexValue,

    /// Field separator (:)
    Colon,

    /// Value separator (,)
    Comma,

    /// Line ending (\n, \r\n)
    Newline,

    /// Section header opening [
    SectionOpen,

    /// Section header closing ]
    SectionClose,

    /// Section name inside brackets
    SectionName,

    /// Complete section header token
    SectionHeader,

    /// Style override opening {
    OverrideOpen,

    /// Style override closing }
    OverrideClose,

    /// Style override block content
    OverrideBlock,

    /// Comment line (; or !:)
    Comment,

    /// Whitespace (spaces, tabs)
    Whitespace,

    /// Drawing mode scale indicator (\p)
    DrawingScale,

    /// UU-encoded data line
    UuEncodedLine,

    /// Font filename declaration
    FontFilename,

    /// Graphic filename declaration
    GraphicFilename,

    /// Format declaration line
    FormatLine,

    /// Event type (Dialogue, Comment, etc.)
    EventType,

    /// Time value (H:MM:SS.CC)
    TimeValue,

    /// Boolean value (-1, 0, 1)
    BooleanValue,

    /// Percentage value (scale, alpha)
    PercentageValue,

    /// String literal (quoted text)
    StringLiteral,

    /// Invalid/unrecognized token
    Invalid,

    /// End of file marker
    Eof,
}

impl TokenType {
    /// Check if token type represents a delimiter
    #[must_use]
    pub const fn is_delimiter(self) -> bool {
        matches!(
            self,
            Self::Colon
                | Self::Comma
                | Self::SectionOpen
                | Self::SectionClose
                | Self::OverrideOpen
                | Self::OverrideClose
        )
    }

    /// Check if token type represents structural elements
    #[must_use]
    pub const fn is_structural(self) -> bool {
        matches!(
            self,
            Self::SectionHeader
                | Self::SectionOpen
                | Self::SectionClose
                | Self::FormatLine
                | Self::Newline
        )
    }

    /// Check if token type represents data content
    #[must_use]
    pub const fn is_content(self) -> bool {
        matches!(
            self,
            Self::Text
                | Self::Number
                | Self::HexValue
                | Self::TimeValue
                | Self::BooleanValue
                | Self::PercentageValue
                | Self::StringLiteral
        )
    }

    /// Check if token type can be skipped during parsing
    #[must_use]
    pub const fn is_skippable(self) -> bool {
        matches!(self, Self::Whitespace | Self::Comment)
    }

    /// Get human-readable name for error messages
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Number => "number",
            Self::HexValue => "hex value",
            Self::Colon => "colon",
            Self::Comma => "comma",
            Self::Newline => "newline",
            Self::SectionOpen => "section open",
            Self::SectionClose => "section close",
            Self::SectionName => "section name",
            Self::SectionHeader => "section header",
            Self::OverrideOpen => "override open",
            Self::OverrideClose => "override close",
            Self::OverrideBlock => "override block",
            Self::Comment => "comment",
            Self::Whitespace => "whitespace",
            Self::DrawingScale => "drawing scale",
            Self::UuEncodedLine => "UU-encoded line",
            Self::FontFilename => "font filename",
            Self::GraphicFilename => "graphic filename",
            Self::FormatLine => "format line",
            Self::EventType => "event type",
            Self::TimeValue => "time value",
            Self::BooleanValue => "boolean value",
            Self::PercentageValue => "percentage value",
            Self::StringLiteral => "string literal",
            Self::Invalid => "invalid token",
            Self::Eof => "end of file",
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Delimiter type for context-aware tokenization
///
/// Helps tokenizer understand context when encountering delimiter characters
/// that may have different meanings in different sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DelimiterType {
    /// Field separator in key-value pairs
    FieldSeparator,

    /// Value separator in CSV format
    ValueSeparator,

    /// Section boundary marker
    SectionBoundary,

    /// Style override boundary
    OverrideBoundary,

    /// Comment marker
    CommentMarker,

    /// Line terminator
    LineTerminator,

    /// Drawing command separator
    DrawingSeparator,

    /// Time component separator
    TimeSeparator,

    /// Color component separator
    ColorSeparator,
}

impl DelimiterType {
    /// Get expected character(s) for this delimiter type
    #[must_use]
    pub const fn chars(self) -> &'static [char] {
        match self {
            Self::FieldSeparator => &[':'],
            Self::ValueSeparator => &[','],
            Self::SectionBoundary => &['[', ']'],
            Self::OverrideBoundary => &['{', '}'],
            Self::CommentMarker => &[';'],
            Self::LineTerminator => &['\n', '\r'],
            Self::DrawingSeparator => &[' ', '\t'],
            Self::TimeSeparator => &[':', '.'],
            Self::ColorSeparator => &['&', 'H'],
        }
    }

    /// Check if character matches this delimiter type
    #[must_use]
    pub fn matches(self, ch: char) -> bool {
        self.chars().contains(&ch)
    }
}

/// Token stream position for streaming tokenization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenPosition {
    /// Byte offset in source
    pub offset: usize,

    /// Line number (1-based)
    pub line: usize,

    /// Column number (1-based)
    pub column: usize,
}

impl TokenPosition {
    /// Create new position
    #[must_use]
    pub const fn new(offset: usize, line: usize, column: usize) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }

    /// Create position at start of input
    #[must_use]
    pub const fn start() -> Self {
        Self::new(0, 1, 1)
    }

    /// Advance position by one character
    #[must_use]
    pub const fn advance(mut self, ch: char) -> Self {
        self.offset += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        self
    }

    /// Advance position by string length
    #[must_use]
    pub fn advance_by_str(mut self, s: &str) -> Self {
        for ch in s.chars() {
            self = self.advance(ch);
        }
        self
    }
}

impl Default for TokenPosition {
    fn default() -> Self {
        Self::start()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_creation() {
        let span = "test";
        let token = Token::new(TokenType::Text, span, 1, 5);

        assert_eq!(token.token_type, TokenType::Text);
        assert_eq!(token.span, "test");
        assert_eq!(token.line, 1);
        assert_eq!(token.column, 5);
        assert_eq!(token.len(), 4);
        assert_eq!(token.end_column(), 9);
    }

    #[test]
    fn token_empty_check() {
        let empty_token = Token::new(TokenType::Text, "", 1, 1);
        assert!(empty_token.is_empty());

        let normal_token = Token::new(TokenType::Text, "text", 1, 1);
        assert!(!normal_token.is_empty());
    }

    #[test]
    fn token_type_checks() {
        assert!(TokenType::Comma.is_delimiter());
        assert!(TokenType::SectionHeader.is_structural());
        assert!(TokenType::Text.is_content());
        assert!(TokenType::Whitespace.is_skippable());
        assert!(TokenType::Comment.is_skippable());
    }

    #[test]
    fn token_classification() {
        let text_token = Token::new(TokenType::Text, "hello", 1, 1);
        assert!(text_token.is_content());
        assert!(!text_token.is_delimiter());
        assert!(!text_token.is_whitespace());

        let comma_token = Token::new(TokenType::Comma, ",", 1, 6);
        assert!(comma_token.is_delimiter());
        assert!(!comma_token.is_content());

        let ws_token = Token::new(TokenType::Whitespace, " ", 1, 7);
        assert!(ws_token.is_whitespace());
        assert!(!ws_token.is_content());
    }

    #[test]
    fn delimiter_type_matching() {
        assert!(DelimiterType::FieldSeparator.matches(':'));
        assert!(DelimiterType::ValueSeparator.matches(','));
        assert!(DelimiterType::SectionBoundary.matches('['));
        assert!(DelimiterType::SectionBoundary.matches(']'));
        assert!(DelimiterType::LineTerminator.matches('\n'));

        assert!(!DelimiterType::FieldSeparator.matches(','));
        assert!(!DelimiterType::ValueSeparator.matches(':'));
    }

    #[test]
    fn token_position_advance() {
        let mut pos = TokenPosition::start();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.offset, 0);

        pos = pos.advance('a');
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 2);
        assert_eq!(pos.offset, 1);

        pos = pos.advance('\n');
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);
        assert_eq!(pos.offset, 2);
    }

    #[test]
    fn token_position_advance_string() {
        let pos = TokenPosition::start();
        let new_pos = pos.advance_by_str("hello\nworld");

        assert_eq!(new_pos.line, 2);
        assert_eq!(new_pos.column, 6); // "world" = 5 chars + 1
        assert_eq!(new_pos.offset, 11); // "hello\nworld".len()
    }

    #[test]
    fn token_type_names() {
        assert_eq!(TokenType::Text.name(), "text");
        assert_eq!(TokenType::Number.name(), "number");
        assert_eq!(TokenType::HexValue.name(), "hex value");
        assert_eq!(TokenType::Invalid.name(), "invalid token");
    }

    #[test]
    fn token_display() {
        let token = Token::new(TokenType::Text, "hello", 2, 5);
        let display = format!("{token}");
        assert!(display.contains("Text"));
        assert!(display.contains("2:5"));
        assert!(display.contains("hello"));
    }

    #[test]
    fn token_utf8_validation() {
        let token = Token::new(TokenType::Text, "valid utf8", 1, 1);
        assert!(token.validate_utf8());

        let unicode_token = Token::new(TokenType::Text, "🎵", 1, 1);
        assert!(unicode_token.validate_utf8());
    }

    #[test]
    fn delimiter_token_types_are_delimiters() {
        assert!(TokenType::Colon.is_delimiter());
        assert!(TokenType::Comma.is_delimiter());
        assert!(TokenType::SectionOpen.is_delimiter());
        assert!(TokenType::SectionClose.is_delimiter());
        assert!(TokenType::OverrideOpen.is_delimiter());
        assert!(TokenType::OverrideClose.is_delimiter());
    }

    #[test]
    fn non_delimiter_token_types_are_not_delimiters() {
        assert!(!TokenType::Text.is_delimiter());
        assert!(!TokenType::Number.is_delimiter());
        assert!(!TokenType::HexValue.is_delimiter());
        assert!(!TokenType::SectionName.is_delimiter());
        assert!(!TokenType::SectionHeader.is_delimiter());
        assert!(!TokenType::OverrideBlock.is_delimiter());
        assert!(!TokenType::Comment.is_delimiter());
        assert!(!TokenType::Whitespace.is_delimiter());
        assert!(!TokenType::Newline.is_delimiter());
        assert!(!TokenType::DrawingScale.is_delimiter());
        assert!(!TokenType::UuEncodedLine.is_delimiter());
        assert!(!TokenType::FontFilename.is_delimiter());
        assert!(!TokenType::GraphicFilename.is_delimiter());
        assert!(!TokenType::FormatLine.is_delimiter());
        assert!(!TokenType::EventType.is_delimiter());
        assert!(!TokenType::TimeValue.is_delimiter());
        assert!(!TokenType::BooleanValue.is_delimiter());
        assert!(!TokenType::PercentageValue.is_delimiter());
        assert!(!TokenType::StringLiteral.is_delimiter());
        assert!(!TokenType::Invalid.is_delimiter());
        assert!(!TokenType::Eof.is_delimiter());
    }

    #[test]
    fn structural_token_types_are_structural() {
        assert!(TokenType::SectionHeader.is_structural());
        assert!(TokenType::SectionOpen.is_structural());
        assert!(TokenType::SectionClose.is_structural());
        assert!(TokenType::FormatLine.is_structural());
        assert!(TokenType::Newline.is_structural());
    }

    #[test]
    fn non_structural_token_types_are_not_structural() {
        assert!(!TokenType::Text.is_structural());
        assert!(!TokenType::Number.is_structural());
        assert!(!TokenType::HexValue.is_structural());
        assert!(!TokenType::Colon.is_structural());
        assert!(!TokenType::Comma.is_structural());
        assert!(!TokenType::SectionName.is_structural());
        assert!(!TokenType::OverrideOpen.is_structural());
        assert!(!TokenType::OverrideClose.is_structural());
        assert!(!TokenType::OverrideBlock.is_structural());
        assert!(!TokenType::Comment.is_structural());
        assert!(!TokenType::Whitespace.is_structural());
        assert!(!TokenType::DrawingScale.is_structural());
        assert!(!TokenType::UuEncodedLine.is_structural());
        assert!(!TokenType::FontFilename.is_structural());
        assert!(!TokenType::GraphicFilename.is_structural());
        assert!(!TokenType::EventType.is_structural());
        assert!(!TokenType::TimeValue.is_structural());
        assert!(!TokenType::BooleanValue.is_structural());
        assert!(!TokenType::PercentageValue.is_structural());
        assert!(!TokenType::StringLiteral.is_structural());
        assert!(!TokenType::Invalid.is_structural());
        assert!(!TokenType::Eof.is_structural());
    }

    #[test]
    fn content_token_types_are_content() {
        assert!(TokenType::Text.is_content());
        assert!(TokenType::Number.is_content());
        assert!(TokenType::HexValue.is_content());
        assert!(TokenType::TimeValue.is_content());
        assert!(TokenType::BooleanValue.is_content());
        assert!(TokenType::PercentageValue.is_content());
        assert!(TokenType::StringLiteral.is_content());
    }

    #[test]
    fn non_content_token_types_are_not_content() {
        assert!(!TokenType::Colon.is_content());
        assert!(!TokenType::Comma.is_content());
        assert!(!TokenType::Newline.is_content());
        assert!(!TokenType::SectionOpen.is_content());
        assert!(!TokenType::SectionClose.is_content());
        assert!(!TokenType::SectionName.is_content());
        assert!(!TokenType::SectionHeader.is_content());
        assert!(!TokenType::OverrideOpen.is_content());
        assert!(!TokenType::OverrideClose.is_content());
        assert!(!TokenType::OverrideBlock.is_content());
        assert!(!TokenType::Comment.is_content());
        assert!(!TokenType::Whitespace.is_content());
        assert!(!TokenType::DrawingScale.is_content());
        assert!(!TokenType::UuEncodedLine.is_content());
        assert!(!TokenType::FontFilename.is_content());
        assert!(!TokenType::GraphicFilename.is_content());
        assert!(!TokenType::FormatLine.is_content());
        assert!(!TokenType::EventType.is_content());
        assert!(!TokenType::Invalid.is_content());
        assert!(!TokenType::Eof.is_content());
    }

    #[test]
    fn skippable_token_types_are_skippable() {
        assert!(TokenType::Whitespace.is_skippable());
        assert!(TokenType::Comment.is_skippable());
    }

    #[test]
    fn basic_non_skippable_token_types() {
        assert!(!TokenType::Text.is_skippable());
        assert!(!TokenType::Number.is_skippable());
        assert!(!TokenType::HexValue.is_skippable());
        assert!(!TokenType::Colon.is_skippable());
        assert!(!TokenType::Comma.is_skippable());
        assert!(!TokenType::Newline.is_skippable());
    }

    #[test]
    fn section_non_skippable_token_types() {
        assert!(!TokenType::SectionOpen.is_skippable());
        assert!(!TokenType::SectionClose.is_skippable());
        assert!(!TokenType::SectionName.is_skippable());
        assert!(!TokenType::SectionHeader.is_skippable());
    }

    #[test]
    fn override_non_skippable_token_types() {
        assert!(!TokenType::OverrideOpen.is_skippable());
        assert!(!TokenType::OverrideClose.is_skippable());
        assert!(!TokenType::OverrideBlock.is_skippable());
    }

    #[test]
    fn special_non_skippable_token_types() {
        assert!(!TokenType::DrawingScale.is_skippable());
        assert!(!TokenType::UuEncodedLine.is_skippable());
        assert!(!TokenType::FontFilename.is_skippable());
        assert!(!TokenType::GraphicFilename.is_skippable());
        assert!(!TokenType::FormatLine.is_skippable());
        assert!(!TokenType::EventType.is_skippable());
        assert!(!TokenType::TimeValue.is_skippable());
        assert!(!TokenType::BooleanValue.is_skippable());
        assert!(!TokenType::PercentageValue.is_skippable());
        assert!(!TokenType::StringLiteral.is_skippable());
        assert!(!TokenType::Invalid.is_skippable());
        assert!(!TokenType::Eof.is_skippable());
    }

    #[test]
    fn basic_token_type_names() {
        assert_eq!(TokenType::Text.name(), "text");
        assert_eq!(TokenType::Number.name(), "number");
        assert_eq!(TokenType::HexValue.name(), "hex value");
        assert_eq!(TokenType::Colon.name(), "colon");
        assert_eq!(TokenType::Comma.name(), "comma");
        assert_eq!(TokenType::Newline.name(), "newline");
        assert_eq!(TokenType::Invalid.name(), "invalid token");
        assert_eq!(TokenType::Eof.name(), "end of file");
    }

    #[test]
    fn section_token_type_names() {
        assert_eq!(TokenType::SectionOpen.name(), "section open");
        assert_eq!(TokenType::SectionClose.name(), "section close");
        assert_eq!(TokenType::SectionName.name(), "section name");
        assert_eq!(TokenType::SectionHeader.name(), "section header");
    }

    #[test]
    fn override_token_type_names() {
        assert_eq!(TokenType::OverrideOpen.name(), "override open");
        assert_eq!(TokenType::OverrideClose.name(), "override close");
        assert_eq!(TokenType::OverrideBlock.name(), "override block");
    }

    #[test]
    fn special_token_type_names() {
        assert_eq!(TokenType::Comment.name(), "comment");
        assert_eq!(TokenType::Whitespace.name(), "whitespace");
        assert_eq!(TokenType::DrawingScale.name(), "drawing scale");
        assert_eq!(TokenType::UuEncodedLine.name(), "UU-encoded line");
        assert_eq!(TokenType::FontFilename.name(), "font filename");
        assert_eq!(TokenType::GraphicFilename.name(), "graphic filename");
        assert_eq!(TokenType::FormatLine.name(), "format line");
        assert_eq!(TokenType::EventType.name(), "event type");
        assert_eq!(TokenType::TimeValue.name(), "time value");
        assert_eq!(TokenType::BooleanValue.name(), "boolean value");
        assert_eq!(TokenType::PercentageValue.name(), "percentage value");
        assert_eq!(TokenType::StringLiteral.name(), "string literal");
    }

    #[test]
    fn token_type_display() {
        assert_eq!(format!("{}", TokenType::Text), "text");
        assert_eq!(format!("{}", TokenType::Number), "number");
        assert_eq!(format!("{}", TokenType::Invalid), "invalid token");
        assert_eq!(format!("{}", TokenType::Eof), "end of file");
    }

    #[test]
    fn all_delimiter_types_chars() {
        assert_eq!(DelimiterType::FieldSeparator.chars(), &[':']);
        assert_eq!(DelimiterType::ValueSeparator.chars(), &[',']);
        assert_eq!(DelimiterType::SectionBoundary.chars(), &['[', ']']);
        assert_eq!(DelimiterType::OverrideBoundary.chars(), &['{', '}']);
        assert_eq!(DelimiterType::CommentMarker.chars(), &[';']);
        assert_eq!(DelimiterType::LineTerminator.chars(), &['\n', '\r']);
        assert_eq!(DelimiterType::DrawingSeparator.chars(), &[' ', '\t']);
        assert_eq!(DelimiterType::TimeSeparator.chars(), &[':', '.']);
        assert_eq!(DelimiterType::ColorSeparator.chars(), &['&', 'H']);
    }

    #[test]
    fn field_and_value_separator_matching() {
        assert!(DelimiterType::FieldSeparator.matches(':'));
        assert!(!DelimiterType::FieldSeparator.matches(','));
        assert!(!DelimiterType::FieldSeparator.matches('['));

        assert!(DelimiterType::ValueSeparator.matches(','));
        assert!(!DelimiterType::ValueSeparator.matches(':'));
        assert!(!DelimiterType::ValueSeparator.matches('['));
    }

    #[test]
    fn boundary_delimiter_matching() {
        assert!(DelimiterType::SectionBoundary.matches('['));
        assert!(DelimiterType::SectionBoundary.matches(']'));
        assert!(!DelimiterType::SectionBoundary.matches('{'));

        assert!(DelimiterType::OverrideBoundary.matches('{'));
        assert!(DelimiterType::OverrideBoundary.matches('}'));
        assert!(!DelimiterType::OverrideBoundary.matches('['));
    }

    #[test]
    fn line_and_comment_delimiter_matching() {
        assert!(DelimiterType::CommentMarker.matches(';'));
        assert!(!DelimiterType::CommentMarker.matches('#'));

        assert!(DelimiterType::LineTerminator.matches('\n'));
        assert!(DelimiterType::LineTerminator.matches('\r'));
        assert!(!DelimiterType::LineTerminator.matches('\t'));
    }

    #[test]
    fn special_delimiter_matching() {
        assert!(DelimiterType::DrawingSeparator.matches(' '));
        assert!(DelimiterType::DrawingSeparator.matches('\t'));
        assert!(!DelimiterType::DrawingSeparator.matches('\n'));

        assert!(DelimiterType::TimeSeparator.matches(':'));
        assert!(DelimiterType::TimeSeparator.matches('.'));
        assert!(!DelimiterType::TimeSeparator.matches(','));

        assert!(DelimiterType::ColorSeparator.matches('&'));
        assert!(DelimiterType::ColorSeparator.matches('H'));
        assert!(!DelimiterType::ColorSeparator.matches('#'));
    }

    #[test]
    fn token_position_edge_cases() {
        let pos = TokenPosition::new(100, 50, 25);
        assert_eq!(pos.offset, 100);
        assert_eq!(pos.line, 50);
        assert_eq!(pos.column, 25);

        // Test default implementation
        let default_pos = TokenPosition::default();
        assert_eq!(default_pos.offset, 0);
        assert_eq!(default_pos.line, 1);
        assert_eq!(default_pos.column, 1);

        // Test start method
        let start_pos = TokenPosition::start();
        assert_eq!(start_pos.offset, 0);
        assert_eq!(start_pos.line, 1);
        assert_eq!(start_pos.column, 1);
    }

    #[test]
    fn token_position_unicode_advance() {
        let mut pos = TokenPosition::start();

        // Test multibyte UTF-8 character
        pos = pos.advance('🎵'); // 4-byte UTF-8 character
        assert_eq!(pos.offset, 4);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 2);

        // Test combination of ASCII and Unicode
        pos = pos.advance('a');
        assert_eq!(pos.offset, 5);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 3);

        // Test newline after Unicode
        pos = pos.advance('\n');
        assert_eq!(pos.offset, 6);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 1);
    }

    #[test]
    fn token_position_advance_by_str_edge_cases() {
        let pos = TokenPosition::start();

        // Empty string
        let pos = pos.advance_by_str("");
        assert_eq!(pos.offset, 0);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);

        // String with only newlines
        let pos = pos.advance_by_str("\n\n\n");
        assert_eq!(pos.offset, 3);
        assert_eq!(pos.line, 4);
        assert_eq!(pos.column, 1);

        // Mixed content with Unicode
        let pos = pos.advance_by_str("hello🎵world\ntest");
        assert_eq!(pos.offset, 3 + 19); // previous 3 + "hello🎵world\ntest".len()
        assert_eq!(pos.line, 5);
        assert_eq!(pos.column, 5); // "test".len() + 1
    }

    #[test]
    fn token_unicode_length() {
        // Test token with Unicode characters
        let unicode_token = Token::new(TokenType::Text, "🎵🎶🎤", 1, 1);
        assert_eq!(unicode_token.len(), 3); // 3 Unicode characters
        assert_eq!(unicode_token.end_column(), 4); // column 1 + 3 chars

        // Test token with mixed ASCII and Unicode
        let mixed_token = Token::new(TokenType::Text, "hello🎵world", 1, 1);
        assert_eq!(mixed_token.len(), 11); // 5 + 1 + 5 characters
        assert_eq!(mixed_token.end_column(), 12);

        // Test empty span
        let empty_token = Token::new(TokenType::Text, "", 1, 1);
        assert_eq!(empty_token.len(), 0);
        assert_eq!(empty_token.end_column(), 1);
    }

    #[test]
    fn token_comprehensive_classification() {
        // Test instance methods match TokenType methods
        let text_token = Token::new(TokenType::Text, "text", 1, 1);
        assert_eq!(text_token.is_content(), TokenType::Text.is_content());
        assert_eq!(text_token.is_delimiter(), TokenType::Text.is_delimiter());

        let comma_token = Token::new(TokenType::Comma, ",", 1, 1);
        assert_eq!(comma_token.is_content(), TokenType::Comma.is_content());
        assert_eq!(comma_token.is_delimiter(), TokenType::Comma.is_delimiter());

        let whitespace_token = Token::new(TokenType::Whitespace, " ", 1, 1);
        assert_eq!(
            whitespace_token.is_whitespace(),
            matches!(TokenType::Whitespace, TokenType::Whitespace)
        );
    }

    #[test]
    fn token_equality_and_cloning() {
        let token1 = Token::new(TokenType::Text, "test", 1, 5);
        let token2 = Token::new(TokenType::Text, "test", 1, 5);
        let token3 = Token::new(TokenType::Number, "test", 1, 5);
        let token4 = Token::new(TokenType::Text, "different", 1, 5);

        assert_eq!(token1, token2);
        assert_ne!(token1, token3);
        assert_ne!(token1, token4);

        let cloned = token1.clone();
        assert_eq!(token1, cloned);
    }

    #[test]
    fn token_debug_formatting() {
        let token = Token::new(TokenType::SectionHeader, "[Script Info]", 2, 1);
        let debug_output = format!("{token:?}");
        assert!(debug_output.contains("SectionHeader"));
        assert!(debug_output.contains("[Script Info]"));
        assert!(debug_output.contains("line: 2"));
        assert!(debug_output.contains("column: 1"));
    }
}
