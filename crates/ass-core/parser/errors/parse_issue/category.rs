//! Issue categories for filtering and editor integration
//!
//! Defines [`IssueCategory`], grouping related recoverable parsing problems
//! for easier filtering, linting, and editor handling.

use core::fmt;

/// Issue categories for filtering and editor integration
///
/// Groups related issues together for easier filtering and handling.
/// Useful for editor extensions and linting tool integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueCategory {
    /// Script structure issues
    Structure,

    /// Style definition problems
    Style,

    /// Event/dialogue issues
    Event,

    /// Timing-related problems
    Timing,

    /// Color format issues
    Color,

    /// Font/typography issues
    Font,

    /// Drawing command problems
    Drawing,

    /// Performance warnings
    Performance,

    /// Compatibility warnings
    Compatibility,

    /// Security warnings
    Security,

    /// General format issues
    Format,
}

impl fmt::Display for IssueCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Structure => write!(f, "structure"),
            Self::Style => write!(f, "style"),
            Self::Event => write!(f, "event"),
            Self::Timing => write!(f, "timing"),
            Self::Color => write!(f, "color"),
            Self::Font => write!(f, "font"),
            Self::Drawing => write!(f, "drawing"),
            Self::Performance => write!(f, "performance"),
            Self::Compatibility => write!(f, "compatibility"),
            Self::Security => write!(f, "security"),
            Self::Format => write!(f, "format"),
        }
    }
}
