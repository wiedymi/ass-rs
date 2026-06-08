//! Error types and result alias for plugin operations.
//!
//! Defines [`PluginError`] for registration and processing failures along with
//! the crate-local [`Result`] alias used throughout the plugin system.

use alloc::string::String;
use core::fmt;

/// Errors that can occur during plugin operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginError {
    /// Handler with same name already registered
    DuplicateHandler(String),
    /// Handler not found for given name
    HandlerNotFound(String),
    /// Plugin processing failed
    ProcessingFailed(String),
    /// Invalid plugin configuration
    InvalidConfig(String),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateHandler(name) => {
                write!(f, "Handler '{name}' already registered")
            }
            Self::HandlerNotFound(name) => {
                write!(f, "Handler '{name}' not found")
            }
            Self::ProcessingFailed(msg) => {
                write!(f, "Plugin processing failed: {msg}")
            }
            Self::InvalidConfig(msg) => {
                write!(f, "Invalid plugin configuration: {msg}")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PluginError {}

/// Result type for plugin operations
pub type Result<T> = core::result::Result<T, PluginError>;
