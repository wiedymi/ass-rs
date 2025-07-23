//! Error categorization and display utilities for ASS-RS
//!
//! Provides error categorization for filtering, grouping, and user interface
//! organization. Includes suggestion system for common error scenarios to
//! help users resolve issues quickly.

use super::CoreError;
use core::fmt;

/// Error category for filtering and user interface organization
///
/// Provides a way to group related errors for better organization in user
/// interfaces, logging systems, and error handling workflows. Each category
/// represents a different class of problems that may require different
/// handling strategies.
///
/// # Examples
///
/// ```rust
/// use ass_core::utils::errors::{CoreError, ErrorCategory};
///
/// let error = CoreError::parse("Invalid syntax");
/// assert_eq!(error.category(), ErrorCategory::Parsing);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// Parsing and tokenization errors
    ///
    /// Issues with understanding the structure and syntax of ASS files.
    /// Usually indicates malformed input that doesn't follow ASS specification.
    Parsing,

    /// Analysis and linting errors
    ///
    /// Problems found during semantic analysis of parsed ASS content.
    /// May include style inconsistencies, timing issues, or logic errors.
    Analysis,

    /// Plugin system errors
    ///
    /// Failures in plugin loading, execution, or communication.
    /// Indicates issues with extensibility features.
    Plugin,

    /// Format validation errors
    ///
    /// Problems with specific value formats like colors, numbers, or times.
    /// Usually indicates data that is syntactically correct but semantically invalid.
    Format,

    /// Text encoding errors
    ///
    /// Issues with character encoding, UTF-8 validation, or text processing.
    /// Often indicates file encoding problems or character set issues.
    Encoding,

    /// I/O and file system errors
    ///
    /// Problems reading from or writing to files, network resources, or other I/O.
    /// Usually indicates system-level issues outside the library's control.
    Io,

    /// Resource and memory errors
    ///
    /// Memory allocation failures, resource limit violations, or system constraints.
    /// Often indicates insufficient resources or resource exhaustion attacks.
    Resource,

    /// Configuration errors
    ///
    /// Problems with library configuration, feature flags, or environment setup.
    /// Usually indicates incorrect setup or missing dependencies.
    Configuration,

    /// Data validation errors
    ///
    /// Issues with content validation beyond basic format checking.
    /// May include cross-reference validation, constraint checking, etc.
    Validation,

    /// Compatibility and version errors
    ///
    /// Problems with version compatibility, feature availability, or platform support.
    /// Indicates environment or configuration mismatches.
    Compatibility,

    /// Security policy violations
    ///
    /// Issues related to security constraints, access control, or safety policies.
    /// Indicates potentially malicious or unsafe content.
    Security,

    /// Internal library bugs
    ///
    /// Errors that indicate bugs in the library itself rather than user issues.
    /// These should be reported to maintainers for investigation.
    Internal,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parsing => write!(f, "parsing"),
            Self::Analysis => write!(f, "analysis"),
            Self::Plugin => write!(f, "plugin"),
            Self::Format => write!(f, "format"),
            Self::Encoding => write!(f, "encoding"),
            Self::Io => write!(f, "io"),
            Self::Resource => write!(f, "resource"),
            Self::Configuration => write!(f, "configuration"),
            Self::Validation => write!(f, "validation"),
            Self::Compatibility => write!(f, "compatibility"),
            Self::Security => write!(f, "security"),
            Self::Internal => write!(f, "internal"),
        }
    }
}

impl ErrorCategory {
    /// Get human-readable category name
    ///
    /// Returns a descriptive name for the category suitable for display
    /// in user interfaces or error reports.
    #[must_use] pub const fn name(self) -> &'static str {
        match self {
            Self::Parsing => "Parsing",
            Self::Analysis => "Analysis",
            Self::Plugin => "Plugin",
            Self::Format => "Format",
            Self::Encoding => "Encoding",
            Self::Io => "I/O",
            Self::Resource => "Resource",
            Self::Configuration => "Configuration",
            Self::Validation => "Validation",
            Self::Compatibility => "Compatibility",
            Self::Security => "Security",
            Self::Internal => "Internal",
        }
    }

    /// Check if errors in this category are typically user-fixable
    ///
    /// Returns `true` for categories where the user can typically resolve
    /// the issue by modifying their input or configuration.
    #[must_use] pub const fn is_user_fixable(self) -> bool {
        match self {
            Self::Parsing
            | Self::Format
            | Self::Encoding
            | Self::Configuration
            | Self::Validation => true,

            Self::Analysis | Self::Compatibility => true,

            Self::Plugin
            | Self::Io
            | Self::Resource
            | Self::Security
            | Self::Internal => false,
        }
    }

    /// Get severity level for this category
    ///
    /// Returns a relative severity level where higher numbers indicate
    /// more severe issues that require immediate attention.
    #[must_use] pub const fn severity_level(self) -> u8 {
        match self {
            Self::Internal | Self::Security => 5,
            Self::Resource | Self::Io => 4,
            Self::Plugin | Self::Compatibility => 3,
            Self::Parsing | Self::Validation => 2,
            Self::Analysis | Self::Configuration => 1,
            Self::Format | Self::Encoding => 1,
        }
    }
}

impl CoreError {
    /// Get error category for filtering/grouping
    ///
    /// Returns the category that best describes the type of error,
    /// useful for organizing errors in user interfaces or logs.
    #[must_use] pub const fn category(&self) -> ErrorCategory {
        match self {
            Self::Parse(_) | Self::Tokenization(_) => ErrorCategory::Parsing,
            Self::Analysis(_) => ErrorCategory::Analysis,
            Self::Plugin(_) => ErrorCategory::Plugin,
            Self::InvalidColor(_) | Self::InvalidNumeric(_) | Self::InvalidTime(_) => {
                ErrorCategory::Format
            }
            Self::Utf8Error { .. } => ErrorCategory::Encoding,
            Self::Io(_) => ErrorCategory::Io,
            Self::OutOfMemory(_) | Self::ResourceLimitExceeded { .. } => ErrorCategory::Resource,
            Self::Config(_) => ErrorCategory::Configuration,
            Self::Validation(_) => ErrorCategory::Validation,
            Self::FeatureNotSupported { .. } | Self::VersionIncompatible { .. } => {
                ErrorCategory::Compatibility
            }
            Self::SecurityViolation(_) => ErrorCategory::Security,
            Self::Internal(_) => ErrorCategory::Internal,
        }
    }

    /// Get suggested action for this error
    ///
    /// Provides actionable advice for resolving common error scenarios.
    /// Returns `None` for errors that don't have standard solutions.
    #[must_use] pub const fn suggestion(&self) -> Option<&'static str> {
        match self {
            Self::InvalidColor(_) => Some("Use format like '&H00FF00FF&' for colors"),
            Self::InvalidTime(_) => Some("Use format like '0:01:30.50' for times"),
            Self::InvalidNumeric(_) => Some("Check numeric format and range"),
            Self::FeatureNotSupported { .. } => Some("Enable required feature in Cargo.toml"),
            Self::OutOfMemory(_) => Some("Reduce input size or enable 'arena' feature"),
            Self::ResourceLimitExceeded { .. } => {
                Some("Reduce input complexity or increase limits")
            }
            Self::SecurityViolation(_) => Some("Review script content for security issues"),
            Self::Internal(_) => Some("Please report this bug to the maintainers"),
            Self::Utf8Error { .. } => Some("Check file encoding - ASS files should be UTF-8"),
            Self::Config(_) => Some("Review configuration settings and feature flags"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_categories() {
        assert_eq!(CoreError::parse("test").category(), ErrorCategory::Parsing);
        assert_eq!(
            CoreError::InvalidColor("test".to_string()).category(),
            ErrorCategory::Format
        );
        assert_eq!(
            CoreError::internal("test").category(),
            ErrorCategory::Internal
        );
    }

    #[test]
    fn error_suggestions() {
        assert!(CoreError::InvalidColor("test".to_string())
            .suggestion()
            .is_some());
        assert!(CoreError::InvalidTime("test".to_string())
            .suggestion()
            .is_some());
        assert!(CoreError::internal("test").suggestion().is_some());
        assert!(CoreError::parse("test").suggestion().is_none());
    }

    #[test]
    fn category_display() {
        assert_eq!(format!("{}", ErrorCategory::Parsing), "parsing");
        assert_eq!(format!("{}", ErrorCategory::Format), "format");
        assert_eq!(format!("{}", ErrorCategory::Internal), "internal");
    }

    #[test]
    fn category_names() {
        assert_eq!(ErrorCategory::Parsing.name(), "Parsing");
        assert_eq!(ErrorCategory::Format.name(), "Format");
        assert_eq!(ErrorCategory::Internal.name(), "Internal");
    }

    #[test]
    fn category_user_fixable() {
        assert!(ErrorCategory::Parsing.is_user_fixable());
        assert!(ErrorCategory::Format.is_user_fixable());
        assert!(!ErrorCategory::Internal.is_user_fixable());
        assert!(!ErrorCategory::Resource.is_user_fixable());
    }

    #[test]
    fn category_severity() {
        assert_eq!(ErrorCategory::Internal.severity_level(), 5);
        assert_eq!(ErrorCategory::Security.severity_level(), 5);
        assert_eq!(ErrorCategory::Resource.severity_level(), 4);
        assert_eq!(ErrorCategory::Parsing.severity_level(), 2);
        assert_eq!(ErrorCategory::Format.severity_level(), 1);
    }

    #[test]
    fn category_equality() {
        assert_eq!(ErrorCategory::Parsing, ErrorCategory::Parsing);
        assert_ne!(ErrorCategory::Parsing, ErrorCategory::Format);
    }

    #[test]
    fn category_copy_clone() {
        let category = ErrorCategory::Parsing;
        let copied = category;
        let cloned = category;

        assert_eq!(category, copied);
        assert_eq!(category, cloned);
    }
}
