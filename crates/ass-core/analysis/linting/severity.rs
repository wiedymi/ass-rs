//! Severity levels for lint issues.
//!
//! Defines [`IssueSeverity`], the ordered scale used to rank linting
//! findings from informational hints up to critical errors.

use core::fmt;

/// Severity level for lint issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IssueSeverity {
    /// Informational message - no action required
    Info,
    /// Hint for improvement - optional fix
    Hint,
    /// Warning - should be addressed but not critical
    Warning,
    /// Error - must be fixed for proper functionality
    Error,
    /// Critical error - script may not work at all
    Critical,
}

impl fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Hint => write!(f, "hint"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Critical => write!(f, "critical"),
        }
    }
}
