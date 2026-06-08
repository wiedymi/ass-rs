//! Severity levels for style validation issues.
//!
//! Defines the ordered [`ValidationSeverity`] scale used to classify style
//! validation issues from informational notes through warnings to errors.

use core::fmt;

/// Severity level for style validation issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    /// Informational message about style properties
    Info,
    /// Warning about potential rendering or performance issues
    Warning,
    /// Error that violates ASS specification or causes problems
    Error,
}

impl fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "info"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
        }
    }
}
