//! Categories for lint issues.
//!
//! Defines [`IssueCategory`], used to group linting findings by the
//! kind of problem they describe (timing, styling, performance, etc.).

use core::fmt;

/// Category of lint issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueCategory {
    /// Timing-related issues
    Timing,
    /// Style definition problems
    Styling,
    /// Text content issues
    Content,
    /// Performance concerns
    Performance,
    /// Spec compliance violations
    Compliance,
    /// Accessibility concerns
    Accessibility,
    /// Encoding or character issues
    Encoding,
}

impl fmt::Display for IssueCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timing => write!(f, "timing"),
            Self::Styling => write!(f, "styling"),
            Self::Content => write!(f, "content"),
            Self::Performance => write!(f, "performance"),
            Self::Compliance => write!(f, "compliance"),
            Self::Accessibility => write!(f, "accessibility"),
            Self::Encoding => write!(f, "encoding"),
        }
    }
}
