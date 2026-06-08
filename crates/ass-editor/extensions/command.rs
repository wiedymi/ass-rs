//! Extension lifecycle state, commands, and result types.
//!
//! Defines the runtime state machine for extensions, the command descriptors
//! they expose, user-facing message levels, and command execution results.

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// Extension lifecycle state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionState {
    /// Extension is uninitialized
    Uninitialized,
    /// Extension is being initialized
    Initializing,
    /// Extension is active and running
    Active,
    /// Extension is paused/suspended
    Paused,
    /// Extension encountered an error
    Error,
    /// Extension is being shut down
    ShuttingDown,
    /// Extension has been shut down
    Shutdown,
}

impl ExtensionState {
    /// Check if the extension is in an active state
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Check if the extension can be used
    pub fn is_usable(&self) -> bool {
        matches!(self, Self::Active | Self::Paused)
    }

    /// Check if the extension is in an error state
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }
}

/// Extension command that can be executed
#[derive(Debug, Clone)]
pub struct ExtensionCommand {
    /// Command identifier
    pub id: String,
    /// Human-readable command name
    pub name: String,
    /// Command description
    pub description: String,
    /// Keyboard shortcut (if any)
    pub shortcut: Option<String>,
    /// Command category for organization
    pub category: String,
    /// Whether the command requires a document
    pub requires_document: bool,
}

impl ExtensionCommand {
    /// Create a new extension command
    pub fn new(id: String, name: String, description: String) -> Self {
        Self {
            id,
            name,
            description,
            shortcut: None,
            category: "General".to_string(),
            requires_document: true,
        }
    }

    /// Set the keyboard shortcut
    pub fn with_shortcut(mut self, shortcut: String) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Set the command category
    pub fn with_category(mut self, category: String) -> Self {
        self.category = category;
        self
    }

    /// Set whether the command requires a document
    pub fn requires_document(mut self, requires: bool) -> Self {
        self.requires_document = requires;
        self
    }
}

/// Message levels for user notifications
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    /// Informational message
    Info,
    /// Warning message
    Warning,
    /// Error message
    Error,
    /// Success message
    Success,
}

/// Result of extension command execution
#[derive(Debug, Clone)]
pub struct ExtensionResult {
    /// Whether the command succeeded
    pub success: bool,
    /// Optional result message
    pub message: Option<String>,
    /// Optional result data
    pub data: HashMap<String, String>,
}

impl ExtensionResult {
    /// Create a successful result
    pub fn success() -> Self {
        Self {
            success: true,
            message: None,
            data: HashMap::new(),
        }
    }

    /// Create a successful result with message
    pub fn success_with_message(message: String) -> Self {
        Self {
            success: true,
            message: Some(message),
            data: HashMap::new(),
        }
    }

    /// Create a failure result
    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message: Some(message),
            data: HashMap::new(),
        }
    }

    /// Add data to the result
    pub fn with_data(mut self, key: String, value: String) -> Self {
        self.data.insert(key, value);
        self
    }
}
