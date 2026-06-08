//! Issue severity levels for tokenizer diagnostics.
//!
//! Defines [`IssueLevel`], which categorizes tokenization issues by severity
//! to enable appropriate error handling and recovery strategies.

/// Token issue severity levels
///
/// Categorizes tokenization issues by severity to enable appropriate
/// error handling and recovery strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueLevel {
    /// Warning that doesn't prevent tokenization
    ///
    /// Indicates potential problems that don't break parsing but may
    /// indicate authoring errors or compatibility issues.
    Warning,

    /// Error that may affect parsing
    ///
    /// Indicates problems that could cause incorrect parsing but allow
    /// tokenization to continue with error recovery.
    Error,

    /// Critical error requiring recovery
    ///
    /// Indicates severe problems that require special handling to
    /// continue tokenization safely.
    Critical,
}

impl IssueLevel {
    /// Check if issue level indicates an error condition
    #[must_use]
    pub const fn is_error(self) -> bool {
        matches!(self, Self::Error | Self::Critical)
    }

    /// Check if issue level should stop tokenization
    #[must_use]
    pub const fn should_abort(self) -> bool {
        matches!(self, Self::Critical)
    }

    /// Get string representation for display
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Critical => "critical",
        }
    }
}
