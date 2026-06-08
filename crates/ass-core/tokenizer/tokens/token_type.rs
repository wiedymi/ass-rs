//! Token type discriminant for the ASS tokenizer.
//!
//! Defines the [`TokenType`] enum classifying lexical units along with helper
//! predicates and human-readable names used throughout parsing.

use core::fmt;

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
