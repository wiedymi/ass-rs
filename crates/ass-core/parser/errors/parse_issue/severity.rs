//! Issue severity levels for partial parse recovery
//!
//! Defines [`IssueSeverity`], describing how serious a recoverable parsing
//! problem is and whether it should block further processing.

use core::fmt;

/// Parse issue severity levels for partial recovery
///
/// Determines how serious an issue is and whether it should block processing.
/// Lower severity issues can often be ignored or worked around.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueSeverity {
    /// Information that may be useful but doesn't affect functionality
    Info,

    /// Warning about potential problems or non-standard usage
    Warning,

    /// Error that was recovered from but may affect rendering
    Error,

    /// Critical error that will likely cause rendering problems
    Critical,
}

impl fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Critical => write!(f, "critical"),
        }
    }
}
