//! Command execution and state/config accessors for the manager.
//!
//! Exposes command dispatch to loaded extensions plus read/write access to
//! extension state, registered commands, configuration, and shared data.

use crate::core::Result;

use super::command::{ExtensionResult, ExtensionState};
use super::context::ExtensionContext;
use super::manager::ExtensionManager;

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

impl ExtensionManager {
    /// Execute a command from an extension
    pub fn execute_command(
        &mut self,
        command_id: &str,
        args: &HashMap<String, String>,
        context: &mut dyn ExtensionContext,
    ) -> Result<ExtensionResult> {
        let extension_name = self
            .with_inner(|inner| inner.commands.get(command_id).map(|(name, _)| name.clone()))
            .ok_or_else(|| crate::core::EditorError::CommandFailed {
                message: format!("Command '{command_id}' not found"),
            })?;

        self.with_inner_mut(|inner| {
            if let Some(extension) = inner.extensions.get_mut(&extension_name) {
                extension.execute_command(command_id, args, context)
            } else {
                Err(crate::core::EditorError::CommandFailed {
                    message: format!("Extension '{extension_name}' not found"),
                })
            }
        })
    }

    /// Get list of loaded extensions
    pub fn list_extensions(&self) -> Vec<String> {
        self.with_inner(|inner| inner.extensions.keys().cloned().collect())
    }

    /// Get extension state
    pub fn get_extension_state(&self, extension_name: &str) -> Option<ExtensionState> {
        self.with_inner(|inner| inner.extension_states.get(extension_name).copied())
    }

    /// Get all available commands
    pub fn list_commands(&self) -> Vec<String> {
        self.with_inner(|inner| inner.commands.keys().cloned().collect())
    }

    /// Get configuration value
    pub fn get_config(&self, key: &str) -> Option<String> {
        self.with_inner(|inner| inner.config.get(key).cloned())
    }

    /// Set configuration value
    pub fn set_config(&mut self, key: String, value: String) {
        self.with_inner_mut(|inner| {
            inner.config.insert(key, value);
        });
    }

    /// Get extension data
    pub fn get_extension_data(&self, extension_name: &str, key: &str) -> Option<String> {
        self.with_inner(|inner| {
            inner
                .extension_data
                .get(extension_name)
                .and_then(|data| data.get(key))
                .cloned()
        })
    }

    /// Set extension data
    pub fn set_extension_data(&mut self, extension_name: String, key: String, value: String) {
        self.with_inner_mut(|inner| {
            inner
                .extension_data
                .entry(extension_name)
                .or_default()
                .insert(key, value);
        });
    }
}
