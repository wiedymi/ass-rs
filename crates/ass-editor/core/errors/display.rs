//! `no_std` `Display` and `Error` implementations for `EditorError`
//!
//! When the `std` feature is disabled, thiserror's derive is unavailable, so
//! these manual implementations provide equivalent message formatting and the
//! `core::error::Error` marker.

#[cfg(not(feature = "std"))]
use super::EditorError;
#[cfg(not(feature = "std"))]
use core::fmt;

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
            Self::SectionNotFound { section } => write!(f, "Section not found: {section}"),
        }
    }
}

/// nostd compatible Error implementation
#[cfg(not(feature = "std"))]
impl core::error::Error for EditorError {}
