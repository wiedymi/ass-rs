//! Extension context trait and the default editor context implementation.
//!
//! `ExtensionContext` exposes editor functionality to extensions; `EditorContext`
//! is the concrete adapter that bridges extensions to the `ExtensionManager`.

use crate::core::{EditorDocument, Result};
use crate::events::DocumentEvent;

use super::command::{ExtensionCommand, MessageLevel};
use super::manager::ExtensionManager;

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(all(not(feature = "multi-thread"), feature = "std"))]
use std::collections::HashMap;

#[cfg(all(not(feature = "multi-thread"), not(feature = "std")))]
use alloc::collections::BTreeMap as HashMap;

/// Extension context providing access to editor functionality
pub trait ExtensionContext {
    /// Get the current document (if any)
    fn current_document(&self) -> Option<&EditorDocument>;

    /// Get a mutable reference to the current document
    fn current_document_mut(&mut self) -> Option<&mut EditorDocument>;

    /// Send an event to the event system
    fn send_event(&mut self, event: DocumentEvent) -> Result<()>;

    /// Get configuration value
    fn get_config(&self, key: &str) -> Option<String>;

    /// Set configuration value
    fn set_config(&mut self, key: String, value: String) -> Result<()>;

    /// Register a command with the editor
    fn register_command(&mut self, command: ExtensionCommand) -> Result<()>;

    /// Show a message to the user
    fn show_message(&mut self, message: &str, level: MessageLevel) -> Result<()>;

    /// Get data from another extension
    fn get_extension_data(&self, extension_name: &str, key: &str) -> Option<String>;

    /// Set data for inter-extension communication
    fn set_extension_data(&mut self, key: String, value: String) -> Result<()>;
}

/// Editor context providing access to editor functionality
/// Adapts to available features automatically
pub struct EditorContext<'a> {
    /// Current document (if any)
    pub document: Option<&'a mut EditorDocument>,
    /// Reference to the extension manager
    #[cfg(feature = "multi-thread")]
    pub manager: ExtensionManager,
    /// Reference to the extension manager for single-threaded builds
    #[cfg(not(feature = "multi-thread"))]
    pub manager: &'a mut ExtensionManager,
    /// Mutable state for single-threaded builds (config updates)
    #[cfg(not(feature = "multi-thread"))]
    pub manager_mut_state: alloc::rc::Rc<core::cell::RefCell<HashMap<String, String>>>,
    /// Name of the current extension
    pub extension_name: String,
}

impl ExtensionContext for EditorContext<'_> {
    fn current_document(&self) -> Option<&EditorDocument> {
        self.document.as_deref()
    }

    fn current_document_mut(&mut self) -> Option<&mut EditorDocument> {
        self.document.as_deref_mut()
    }

    fn send_event(&mut self, _event: DocumentEvent) -> Result<()> {
        // For now, we'll just log the event
        // In a real implementation, this would use the event system
        #[cfg(feature = "std")]
        {
            eprintln!("Extension {} sent event: {:?}", self.extension_name, _event);
        }
        Ok(())
    }

    fn get_config(&self, key: &str) -> Option<String> {
        #[cfg(feature = "multi-thread")]
        {
            self.manager.get_config(key)
        }
        #[cfg(not(feature = "multi-thread"))]
        {
            self.manager.get_config(key)
        }
    }

    fn set_config(&mut self, key: String, value: String) -> Result<()> {
        #[cfg(feature = "multi-thread")]
        {
            self.manager.set_config(key.clone(), value.clone());
        }
        #[cfg(not(feature = "multi-thread"))]
        {
            // In single-threaded mode, update the shared config state
            self.manager_mut_state.borrow_mut().insert(key, value);
        }
        Ok(())
    }

    fn register_command(&mut self, _command: ExtensionCommand) -> Result<()> {
        // For now, just acknowledge the command in both modes
        // In a real implementation, this would register with a command system
        #[cfg(feature = "std")]
        {
            eprintln!(
                "Extension {} registered command: {}",
                self.extension_name, _command.id
            );
        }
        Ok(())
    }

    fn show_message(&mut self, _message: &str, _level: MessageLevel) -> Result<()> {
        // Simple console output for now
        #[cfg(feature = "std")]
        {
            match _level {
                MessageLevel::Info => eprintln!("[INFO] {}: {}", self.extension_name, _message),
                MessageLevel::Warning => eprintln!("[WARN] {}: {}", self.extension_name, _message),
                MessageLevel::Error => eprintln!("[ERROR] {}: {}", self.extension_name, _message),
                MessageLevel::Success => {
                    eprintln!("[SUCCESS] {}: {}", self.extension_name, _message)
                }
            }
        }
        Ok(())
    }

    fn get_extension_data(&self, extension_name: &str, key: &str) -> Option<String> {
        self.manager.get_extension_data(extension_name, key)
    }

    fn set_extension_data(&mut self, key: String, value: String) -> Result<()> {
        self.manager
            .set_extension_data(self.extension_name.clone(), key, value);
        Ok(())
    }
}
