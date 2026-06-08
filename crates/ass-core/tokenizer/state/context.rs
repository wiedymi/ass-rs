//! Tokenization context tracking for state-aware ASS tokenization.
//!
//! Defines [`TokenContext`], which records the current lexical context so the
//! tokenizer can apply context-sensitive rules for ASS script elements that
//! have different lexical rules in different contexts.

/// Tokenization context for state-aware parsing
///
/// Tracks current parsing context to enable context-sensitive tokenization
/// of ASS script elements that have different lexical rules in different
/// contexts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TokenContext {
    /// Top-level document parsing
    ///
    /// Default state for processing section headers, comments, and
    /// top-level document structure.
    #[default]
    Document,

    /// Inside section header like `[Events]`
    ///
    /// Special tokenization rules for section names within square brackets.
    SectionHeader,

    /// Inside field definition line
    ///
    /// Field values have different whitespace and delimiter handling than
    /// other contexts.
    FieldValue,

    /// Inside style override block like {\b1}
    ///
    /// Override tags use backslash prefixes and have special syntax rules.
    StyleOverride,

    /// Inside drawing commands (\p1)
    ///
    /// Drawing commands use vector graphics syntax with different
    /// coordinate and command parsing rules.
    DrawingCommands,

    /// Inside UU-encoded data (fonts/graphics)
    ///
    /// Binary data sections use different character validation and
    /// line parsing rules.
    UuEncodedData,
}

impl TokenContext {
    /// Check if context allows whitespace skipping
    #[must_use]
    pub const fn allows_whitespace_skipping(self) -> bool {
        !matches!(self, Self::FieldValue | Self::UuEncodedData)
    }

    /// Check if context is inside a delimited block
    #[must_use]
    pub const fn is_delimited_block(self) -> bool {
        matches!(self, Self::SectionHeader | Self::StyleOverride)
    }

    /// Get expected closing delimiter for context
    #[must_use]
    pub const fn closing_delimiter(self) -> Option<char> {
        match self {
            Self::SectionHeader => Some(']'),
            Self::StyleOverride => Some('}'),
            _ => None,
        }
    }

    /// Transition to field value context after colon
    #[must_use]
    pub const fn enter_field_value(self) -> Self {
        match self {
            Self::Document => Self::FieldValue,
            other => other,
        }
    }

    /// Reset to document context (typically after newline)
    #[must_use]
    pub const fn reset_to_document(self) -> Self {
        Self::Document
    }
}
