//! Constructor helpers and classification methods for `EditorError`
//!
//! Provides ergonomic constructors for common editor error variants plus
//! predicates for inspecting errors (recoverability, position/history
//! classification, and core-error extraction).

use super::EditorError;
use ass_core::utils::errors::CoreError;
use core::fmt;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

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
