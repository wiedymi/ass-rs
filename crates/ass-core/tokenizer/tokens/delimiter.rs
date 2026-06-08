//! Delimiter classification for context-aware tokenization.
//!
//! Defines the [`DelimiterType`] enum describing the meaning of delimiter
//! characters that may differ across ASS script sections.

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
