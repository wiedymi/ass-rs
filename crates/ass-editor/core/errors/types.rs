//! `EditorError` enum definition, `Result` alias, and `CoreError` conversion
//!
//! Defines the main `EditorError` enum that wraps `CoreError` from ass-core
//! and adds editor-specific error cases, along with the editor `Result` type
//! alias and the `From<CoreError>` conversion.

use ass_core::utils::errors::CoreError;

#[cfg(feature = "std")]
use thiserror::Error;

#[cfg(not(feature = "std"))]
use alloc::string::String;

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

/// Result type alias for editor operations
pub type Result<T> = core::result::Result<T, EditorError>;

/// Implement `From<CoreError>` for automatic conversion
impl From<CoreError> for EditorError {
    fn from(err: CoreError) -> Self {
        Self::Core(err)
    }
}
