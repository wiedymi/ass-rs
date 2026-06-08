//! Core extension trait and user-message handlers.
//!
//! Defines the `EditorExtension` trait that all extensions implement along
//! with the `MessageHandler` trait and its std / `no_std` implementations.

use crate::core::Result;
use crate::events::DocumentEvent;

use super::command::{ExtensionCommand, ExtensionResult, ExtensionState, MessageLevel};
use super::context::ExtensionContext;
use super::info::ExtensionInfo;

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap as HashMap, string::String, vec::Vec};

/// Main extension trait that extensions must implement
pub trait EditorExtension: Send + Sync {
    /// Get extension metadata
    fn info(&self) -> &ExtensionInfo;

    /// Initialize the extension
    fn initialize(&mut self, context: &mut dyn ExtensionContext) -> Result<()>;

    /// Shutdown the extension
    fn shutdown(&mut self, context: &mut dyn ExtensionContext) -> Result<()>;

    /// Get the current state of the extension
    fn state(&self) -> ExtensionState;

    /// Execute a command provided by this extension
    fn execute_command(
        &mut self,
        command_id: &str,
        args: &HashMap<String, String>,
        context: &mut dyn ExtensionContext,
    ) -> Result<ExtensionResult>;

    /// Get commands provided by this extension
    fn commands(&self) -> Vec<ExtensionCommand> {
        Vec::new()
    }

    /// Handle a document event (optional)
    fn handle_event(
        &mut self,
        _event: &DocumentEvent,
        _context: &mut dyn ExtensionContext,
    ) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }

    /// Get configuration schema (optional)
    fn config_schema(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// Validate configuration (optional)
    fn validate_config(&self, _config: &HashMap<String, String>) -> Result<()> {
        Ok(())
    }

    /// Pause the extension (optional)
    fn pause(&mut self) -> Result<()> {
        Ok(())
    }

    /// Resume the extension (optional)
    fn resume(&mut self) -> Result<()> {
        Ok(())
    }

    /// Get extension-specific data (optional)
    fn get_data(&self, _key: &str) -> Option<String> {
        None
    }

    /// Set extension-specific data (optional)
    fn set_data(&mut self, _key: String, _value: String) -> Result<()> {
        Ok(())
    }
}

/// Message handler trait for showing messages to users
pub trait MessageHandler: Send + Sync {
    /// Show a message to the user
    fn show(&mut self, message: &str, level: MessageLevel) -> Result<()>;
}

/// Default message handler implementation for std environments
#[cfg(feature = "std")]
pub struct StdMessageHandler;

#[cfg(feature = "std")]
impl MessageHandler for StdMessageHandler {
    fn show(&mut self, message: &str, level: MessageLevel) -> Result<()> {
        match level {
            MessageLevel::Error => eprintln!("[ERROR] {message}"),
            MessageLevel::Warning => eprintln!("[WARN] {message}"),
            MessageLevel::Info => println!("[INFO] {message}"),
            MessageLevel::Success => println!("[SUCCESS] {message}"),
        }
        Ok(())
    }
}

/// No-op message handler for no_std environments
#[cfg(not(feature = "std"))]
pub struct NoOpMessageHandler;

#[cfg(not(feature = "std"))]
impl MessageHandler for NoOpMessageHandler {
    fn show(&mut self, _message: &str, _level: MessageLevel) -> Result<()> {
        Ok(())
    }
}
