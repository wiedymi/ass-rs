//! Token type definitions and highlighted-token representation.

use crate::core::Range;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Token types for syntax highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// Section headers like [Script Info]
    SectionHeader,
    /// Field names like Title:, PlayResX:
    FieldName,
    /// Field values
    FieldValue,
    /// Event type (Dialogue, Comment)
    EventType,
    /// Style name references
    StyleName,
    /// Time codes
    TimeCode,
    /// Override tags like {\pos(100,200)}
    OverrideTag,
    /// Tag parameters
    TagParameter,
    /// Comments
    Comment,
    /// Plain text
    Text,
    /// Errors or invalid syntax
    Error,
}

impl TokenType {
    /// Get CSS class name for web-based highlighting
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::SectionHeader => "ass-section-header",
            Self::FieldName => "ass-field-name",
            Self::FieldValue => "ass-field-value",
            Self::EventType => "ass-event-type",
            Self::StyleName => "ass-style-name",
            Self::TimeCode => "ass-timecode",
            Self::OverrideTag => "ass-override-tag",
            Self::TagParameter => "ass-tag-param",
            Self::Comment => "ass-comment",
            Self::Text => "ass-text",
            Self::Error => "ass-error",
        }
    }

    /// Get ANSI color code for terminal highlighting
    pub fn ansi_color(&self) -> &'static str {
        match self {
            Self::SectionHeader => "\x1b[1;34m", // Bright Blue
            Self::FieldName => "\x1b[36m",       // Cyan
            Self::FieldValue => "\x1b[37m",      // White
            Self::EventType => "\x1b[1;32m",     // Bright Green
            Self::StyleName => "\x1b[35m",       // Magenta
            Self::TimeCode => "\x1b[33m",        // Yellow
            Self::OverrideTag => "\x1b[1;31m",   // Bright Red
            Self::TagParameter => "\x1b[31m",    // Red
            Self::Comment => "\x1b[90m",         // Bright Black (Gray)
            Self::Text => "\x1b[0m",             // Reset
            Self::Error => "\x1b[1;91m",         // Bright Red
        }
    }
}

/// A highlighted token in the document
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightToken {
    /// Range of the token in the document
    pub range: Range,
    /// Type of the token
    pub token_type: TokenType,
    /// Optional semantic information
    pub semantic_info: Option<String>,
}
