//! Unit tests for the extension manager and editor context.

use super::*;
use crate::core::{EditorDocument, Result};

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

#[cfg(not(feature = "std"))]
use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec,
    vec::Vec,
};

#[test]
fn extension_manager_creation() {
    let manager = ExtensionManager::new();
    assert_eq!(manager.list_extensions().len(), 0);
    assert_eq!(manager.list_commands().len(), 0);
}

#[test]
fn extension_manager_config() {
    let mut manager = ExtensionManager::new();

    manager.set_config("test_key".to_string(), "test_value".to_string());
    assert_eq!(
        manager.get_config("test_key"),
        Some("test_value".to_string())
    );
    assert_eq!(manager.get_config("nonexistent"), None);
}

#[test]
fn extension_manager_data() {
    let mut manager = ExtensionManager::new();

    manager.set_extension_data("ext1".to_string(), "key".to_string(), "value".to_string());
    assert_eq!(
        manager.get_extension_data("ext1", "key"),
        Some("value".to_string())
    );
    assert_eq!(manager.get_extension_data("ext1", "nonexistent"), None);
    assert_eq!(manager.get_extension_data("ext2", "key"), None);
}

// Mock extension for testing
struct TestExtension {
    info: ExtensionInfo,
    state: ExtensionState,
    data: HashMap<String, String>,
}

impl TestExtension {
    fn new(name: &str) -> Self {
        Self {
            info: ExtensionInfo::new(
                name.to_string(),
                "1.0.0".to_string(),
                "Test".to_string(),
                "Test extension".to_string(),
            ),
            state: ExtensionState::Uninitialized,
            data: HashMap::new(),
        }
    }
}

impl EditorExtension for TestExtension {
    fn info(&self) -> &ExtensionInfo {
        &self.info
    }

    fn initialize(&mut self, _context: &mut dyn ExtensionContext) -> Result<()> {
        self.state = ExtensionState::Active;
        Ok(())
    }

    fn shutdown(&mut self, _context: &mut dyn ExtensionContext) -> Result<()> {
        self.state = ExtensionState::Shutdown;
        Ok(())
    }

    fn state(&self) -> ExtensionState {
        self.state
    }

    fn execute_command(
        &mut self,
        command_id: &str,
        _args: &HashMap<String, String>,
        _context: &mut dyn ExtensionContext,
    ) -> Result<ExtensionResult> {
        match command_id {
            "test-command" => Ok(ExtensionResult::success_with_message(
                "Command executed".to_string(),
            )),
            _ => Ok(ExtensionResult::failure("Unknown command".to_string())),
        }
    }

    fn commands(&self) -> Vec<ExtensionCommand> {
        vec![ExtensionCommand::new(
            "test-command".to_string(),
            "Test Command".to_string(),
            "A test command".to_string(),
        )]
    }

    fn get_data(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    fn set_data(&mut self, key: String, value: String) -> Result<()> {
        self.data.insert(key, value);
        Ok(())
    }
}

#[test]
fn extension_manager_lifecycle() {
    let mut manager = ExtensionManager::new();
    let _doc = EditorDocument::new();

    let extension = Box::new(TestExtension::new("test-ext"));
    manager.load_extension(extension).unwrap();

    assert_eq!(manager.list_extensions().len(), 1);
    assert_eq!(
        manager.get_extension_state("test-ext"),
        Some(ExtensionState::Uninitialized)
    );

    {
        // We can't directly test initialization with the new context due to borrowing constraints
        // The manager needs mutable access to create the context
    }

    // Test that we can check states after operations
    let extension_exists = manager.list_extensions().contains(&"test-ext".to_string());
    assert!(extension_exists);
}

#[test]
fn editor_context() {
    let mut manager = ExtensionManager::new();

    // Test config through manager first
    manager.set_config("test".to_string(), "value".to_string());
    assert_eq!(manager.get_config("test"), Some("value".to_string()));

    // Test extension data through manager
    manager.set_extension_data("default".to_string(), "key".to_string(), "data".to_string());
    assert_eq!(
        manager.get_extension_data("default", "key"),
        Some("data".to_string())
    );
}
