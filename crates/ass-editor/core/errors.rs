//! Error types for the ass-editor crate
//!
//! Provides the main `EditorError` enum that wraps `CoreError` from ass-core
//! and adds editor-specific error cases. Follows the same philosophy as core:
//! - Use thiserror for structured error handling (no anyhow)
//! - Provide detailed context for debugging
//! - Support error chains with source information
//! - Maintain zero-cost error handling where possible

use ass_core::utils::errors::CoreError;
use core::fmt;

#[cfg(feature = "std")]
use thiserror::Error;

/// Main error type for ass-editor operations
///
/// Wraps `CoreError` from ass-core and adds editor-specific error cases
/// for document management, command execution, and session handling.
#[cfg_attr(feature = "std", derive(Error))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorError {
    /// Errors from ass-core
    #[cfg_attr(feature = "std", error(transparent))]
    Core(CoreError),

    /// Document not found in session
    #[cfg_attr(feature = "std", error("Document not found: {id}"))]
    DocumentNotFound { id: String },

    /// Invalid document position
    #[cfg_attr(
        feature = "std",
        error("Invalid position: line {line}, column {column}")
    )]
    InvalidPosition { line: usize, column: usize },

    /// Position out of bounds
    #[cfg_attr(
        feature = "std",
        error("Position out of bounds: {position} (document length: {length})")
    )]
    PositionOutOfBounds { position: usize, length: usize },

    /// Invalid text range
    #[cfg_attr(
        feature = "std",
        error("Invalid range: start {start}, end {end} (document length: {length})")
    )]
    InvalidRange {
        start: usize,
        end: usize,
        length: usize,
    },

    /// Command execution failed
    #[cfg_attr(feature = "std", error("Command execution failed: {message}"))]
    CommandFailed { message: String },

    /// Undo/redo operation failed
    #[cfg_attr(feature = "std", error("History operation failed: {message}"))]
    HistoryError { message: String },

    /// No operation to undo
    #[cfg_attr(feature = "std", error("Nothing to undo"))]
    NothingToUndo,

    /// No operation to redo
    #[cfg_attr(feature = "std", error("Nothing to redo"))]
    NothingToRedo,

    /// Session limit exceeded
    #[cfg_attr(
        feature = "std",
        error("Session limit exceeded: {current}/{limit} sessions")
    )]
    SessionLimitExceeded { current: usize, limit: usize },

    /// Search index error
    #[cfg_attr(feature = "std", error("Search index error: {message}"))]
    SearchIndexError { message: String },

    /// Extension error
    #[cfg_attr(feature = "std", error("Extension error: {extension}: {message}"))]
    ExtensionError { extension: String, message: String },

    /// Feature not available without specific feature flag
    #[cfg_attr(
        feature = "std",
        error("Feature '{feature}' requires '{required_feature}' feature flag")
    )]
    FeatureNotEnabled {
        feature: String,
        required_feature: String,
    },

    /// Arena allocation error (when using arena feature)
    #[cfg_attr(feature = "std", error("Arena allocation failed: {message}"))]
    ArenaAllocationFailed { message: String },

    /// Section not found in document
    #[cfg_attr(feature = "std", error("Section not found: {section}"))]
    SectionNotFound { section: String },

    /// Rope operation failed (when using rope feature)
    #[cfg_attr(feature = "std", error("Rope operation failed: {message}"))]
    RopeOperationFailed { message: String },

    /// Event channel error
    #[cfg_attr(feature = "std", error("Event channel error: {message}"))]
    EventChannelError { message: String },

    /// Validation error from lazy validator
    #[cfg_attr(feature = "std", error("Validation error: {message}"))]
    ValidationError { message: String },

    /// Import/export error
    #[cfg_attr(feature = "std", error("IO error: {0}"))]
    IoError(String),

    /// Invalid format error
    #[cfg_attr(feature = "std", error("Invalid format: {0}"))]
    InvalidFormat(String),

    /// Unsupported format error
    #[cfg_attr(feature = "std", error("Unsupported format: {0}"))]
    UnsupportedFormat(String),

    /// Thread safety error (multi-thread feature)
    #[cfg_attr(feature = "std", error("Thread safety error: {message}"))]
    ThreadSafetyError { message: String },

    /// Builder validation error
    #[cfg_attr(feature = "std", error("Builder validation error: {message}"))]
    BuilderValidationError { message: String },

    /// Serialization error
    #[cfg_attr(feature = "std", error("Serialization error: {message}"))]
    SerializationError { message: String },

    /// Format line parsing error
    #[cfg_attr(feature = "std", error("Format line error: {message}"))]
    FormatLineError { message: String },
}

impl EditorError {
    /// Create a new command failed error
    pub fn command_failed<T: fmt::Display>(message: T) -> Self {
        Self::CommandFailed {
            message: message.to_string(),
        }
    }

    /// Create a new validation error
    pub fn validation<T: fmt::Display>(message: T) -> Self {
        Self::ValidationError {
            message: message.to_string(),
        }
    }

    /// Create a new IO error
    pub fn io<T: fmt::Display>(message: T) -> Self {
        Self::IoError(message.to_string())
    }

    /// Create a new builder validation error
    pub fn builder_validation<T: fmt::Display>(message: T) -> Self {
        Self::BuilderValidationError {
            message: message.to_string(),
        }
    }

    /// Create a new serialization error
    pub fn serialization<T: fmt::Display>(message: T) -> Self {
        Self::SerializationError {
            message: message.to_string(),
        }
    }

    /// Create a new format line error
    pub fn format_line<T: fmt::Display>(message: T) -> Self {
        Self::FormatLineError {
            message: message.to_string(),
        }
    }

    /// Check if error is recoverable
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        match self {
            Self::Core(core_err) => core_err.is_recoverable(),
            Self::DocumentNotFound { .. }
            | Self::InvalidPosition { .. }
            | Self::PositionOutOfBounds { .. }
            | Self::InvalidRange { .. }
            | Self::CommandFailed { .. }
            | Self::HistoryError { .. }
            | Self::NothingToUndo
            | Self::NothingToRedo
            | Self::SearchIndexError { .. }
            | Self::ExtensionError { .. }
            | Self::FeatureNotEnabled { .. }
            | Self::RopeOperationFailed { .. }
            | Self::EventChannelError { .. }
            | Self::ValidationError { .. }
            | Self::IoError(..)
            | Self::InvalidFormat(..)
            | Self::UnsupportedFormat(..)
            | Self::BuilderValidationError { .. }
            | Self::SerializationError { .. }
            | Self::FormatLineError { .. }
            | Self::SectionNotFound { .. } => true,
            Self::SessionLimitExceeded { .. }
            | Self::ArenaAllocationFailed { .. }
            | Self::ThreadSafetyError { .. } => false,
        }
    }

    /// Check if this is a position-related error
    #[must_use]
    pub const fn is_position_error(&self) -> bool {
        matches!(
            self,
            Self::InvalidPosition { .. }
                | Self::PositionOutOfBounds { .. }
                | Self::InvalidRange { .. }
        )
    }

    /// Check if this is a history-related error
    #[must_use]
    pub const fn is_history_error(&self) -> bool {
        matches!(
            self,
            Self::HistoryError { .. } | Self::NothingToUndo | Self::NothingToRedo
        )
    }

    /// Get the underlying core error if this wraps one
    #[must_use]
    pub const fn as_core_error(&self) -> Option<&CoreError> {
        match self {
            Self::Core(core_err) => Some(core_err),
            _ => None,
        }
    }
}

/// Result type alias for editor operations
pub type Result<T> = core::result::Result<T, EditorError>;

/// Implement From<CoreError> for automatic conversion
impl From<CoreError> for EditorError {
    fn from(err: CoreError) -> Self {
        Self::Core(err)
    }
}

/// nostd compatible Display implementation
#[cfg(not(feature = "std"))]
impl fmt::Display for EditorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Core(err) => write!(f, "{err}"),
            Self::DocumentNotFound { id } => write!(f, "Document not found: {id}"),
            Self::InvalidPosition { line, column } => {
                write!(f, "Invalid position: line {line}, column {column}")
            }
            Self::PositionOutOfBounds { position, length } => {
                write!(
                    f,
                    "Position out of bounds: {position} (document length: {length})"
                )
            }
            Self::InvalidRange { start, end, length } => {
                write!(
                    f,
                    "Invalid range: start {start}, end {end} (document length: {length})"
                )
            }
            Self::CommandFailed { message } => write!(f, "Command execution failed: {message}"),
            Self::HistoryError { message } => write!(f, "History operation failed: {message}"),
            Self::NothingToUndo => write!(f, "Nothing to undo"),
            Self::NothingToRedo => write!(f, "Nothing to redo"),
            Self::SessionLimitExceeded { current, limit } => {
                write!(f, "Session limit exceeded: {current}/{limit} sessions")
            }
            Self::SearchIndexError { message } => write!(f, "Search index error: {message}"),
            Self::ExtensionError { extension, message } => {
                write!(f, "Extension error: {extension}: {message}")
            }
            Self::FeatureNotEnabled {
                feature,
                required_feature,
            } => {
                write!(
                    f,
                    "Feature '{feature}' requires '{required_feature}' feature flag"
                )
            }
            Self::ArenaAllocationFailed { message } => {
                write!(f, "Arena allocation failed: {message}")
            }
            Self::RopeOperationFailed { message } => write!(f, "Rope operation failed: {message}"),
            Self::EventChannelError { message } => write!(f, "Event channel error: {message}"),
            Self::ValidationError { message } => write!(f, "Validation error: {message}"),
            Self::IoError(message) => write!(f, "IO error: {message}"),
            Self::InvalidFormat(message) => write!(f, "Invalid format: {message}"),
            Self::UnsupportedFormat(format) => write!(f, "Unsupported format: {format}"),
            Self::ThreadSafetyError { message } => write!(f, "Thread safety error: {message}"),
            Self::BuilderValidationError { message } => {
                write!(f, "Builder validation error: {message}")
            }
            Self::SerializationError { message } => write!(f, "Serialization error: {message}"),
            Self::FormatLineError { message } => write!(f, "Format line error: {message}"),
        }
    }
}

/// nostd compatible Error implementation
#[cfg(not(feature = "std"))]
impl core::error::Error for EditorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_conversion_from_core() {
        let core_err = CoreError::parse("test error");
        let editor_err: EditorError = core_err.into();
        assert!(matches!(editor_err, EditorError::Core(_)));
    }

    #[test]
    fn error_recoverability() {
        assert!(EditorError::command_failed("test").is_recoverable());
        assert!(EditorError::validation("test").is_recoverable());
        assert!(!EditorError::SessionLimitExceeded {
            current: 10,
            limit: 10
        }
        .is_recoverable());
    }

    #[test]
    fn position_error_detection() {
        assert!(EditorError::InvalidPosition { line: 1, column: 1 }.is_position_error());
        assert!(EditorError::PositionOutOfBounds {
            position: 100,
            length: 50
        }
        .is_position_error());
        assert!(!EditorError::command_failed("test").is_position_error());
    }

    #[test]
    fn history_error_detection() {
        assert!(EditorError::NothingToUndo.is_history_error());
        assert!(EditorError::NothingToRedo.is_history_error());
        assert!(EditorError::HistoryError {
            message: "test".to_string()
        }
        .is_history_error());
        assert!(!EditorError::command_failed("test").is_history_error());
    }

    #[test]
    fn core_error_extraction() {
        let core_err = CoreError::parse("test");
        let editor_err = EditorError::Core(core_err.clone());
        assert_eq!(editor_err.as_core_error(), Some(&core_err));
        assert_eq!(EditorError::command_failed("test").as_core_error(), None);
    }

    #[test]
    fn new_error_types() {
        // Test builder validation error
        let builder_err = EditorError::builder_validation("Invalid field value");
        assert!(builder_err.is_recoverable());
        assert!(matches!(
            builder_err,
            EditorError::BuilderValidationError { .. }
        ));

        // Test serialization error
        let serialization_err = EditorError::serialization("Failed to serialize AST");
        assert!(serialization_err.is_recoverable());
        assert!(matches!(
            serialization_err,
            EditorError::SerializationError { .. }
        ));

        // Test format line error
        let format_err = EditorError::format_line("Invalid format specification");
        assert!(format_err.is_recoverable());
        assert!(matches!(format_err, EditorError::FormatLineError { .. }));
    }

    #[test]
    fn error_display_new_types() {
        let builder_err = EditorError::builder_validation("test");
        assert_eq!(builder_err.to_string(), "Builder validation error: test");

        let serialization_err = EditorError::serialization("test");
        assert_eq!(serialization_err.to_string(), "Serialization error: test");

        let format_err = EditorError::format_line("test");
        assert_eq!(format_err.to_string(), "Format line error: test");
    }
}
