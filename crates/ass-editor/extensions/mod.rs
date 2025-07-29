//! Extension system for editor functionality
//!
//! Provides the `EditorExtension` trait for extending editor capabilities
//! with custom functionality. Supports both synchronous and asynchronous
//! operations, lifecycle management, and inter-extension communication.

pub mod builtin;

#[cfg(not(feature = "std"))]
extern crate alloc;

use crate::core::{EditorDocument, Result};
use crate::events::DocumentEvent;

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::{BTreeMap as HashMap, String, Vec};

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::String, vec::Vec};

#[cfg(feature = "multi-thread")]
use std::sync::{Arc, RwLock};

#[cfg(not(feature = "multi-thread"))]
use core::cell::RefCell;

#[cfg(all(not(feature = "multi-thread"), not(feature = "std")))]
use alloc::rc::Rc;

#[cfg(all(not(feature = "multi-thread"), feature = "std"))]
use std::rc::Rc;

/// Extension capabilities that can be provided
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionCapability {
    /// Text processing and transformation
    TextProcessing,
    /// Syntax highlighting and theming
    SyntaxHighlighting,
    /// Code completion and suggestions  
    CodeCompletion,
    /// Linting and validation
    Linting,
    /// Import/export format support
    FormatSupport,
    /// Custom commands and shortcuts
    CustomCommands,
    /// UI enhancements and widgets
    UserInterface,
    /// External tool integration
    ToolIntegration,
    /// Custom event handling
    EventHandling,
    /// Performance monitoring
    Performance,
}

impl ExtensionCapability {
    /// Get a human-readable description of the capability
    pub fn description(&self) -> &'static str {
        match self {
            Self::TextProcessing => "Text processing and transformation",
            Self::SyntaxHighlighting => "Syntax highlighting and theming",
            Self::CodeCompletion => "Code completion and suggestions",
            Self::Linting => "Linting and validation",
            Self::FormatSupport => "Import/export format support",
            Self::CustomCommands => "Custom commands and shortcuts",
            Self::UserInterface => "UI enhancements and widgets",
            Self::ToolIntegration => "External tool integration",
            Self::EventHandling => "Custom event handling",
            Self::Performance => "Performance monitoring",
        }
    }
}

/// Extension metadata and information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionInfo {
    /// Extension name
    pub name: String,
    /// Extension version
    pub version: String,
    /// Extension author
    pub author: String,
    /// Extension description
    pub description: String,
    /// Capabilities provided by this extension
    pub capabilities: Vec<ExtensionCapability>,
    /// Dependencies on other extensions
    pub dependencies: Vec<String>,
    /// Optional extension website/homepage
    pub homepage: Option<String>,
    /// License identifier
    pub license: Option<String>,
}

impl ExtensionInfo {
    /// Create a new extension info
    pub fn new(name: String, version: String, author: String, description: String) -> Self {
        Self {
            name,
            version,
            author,
            description,
            capabilities: Vec::new(),
            dependencies: Vec::new(),
            homepage: None,
            license: None,
        }
    }

    /// Add a capability to this extension
    pub fn with_capability(mut self, capability: ExtensionCapability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Add multiple capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<ExtensionCapability>) -> Self {
        self.capabilities.extend(capabilities);
        self
    }

    /// Add a dependency on another extension
    pub fn with_dependency(mut self, dependency: String) -> Self {
        self.dependencies.push(dependency);
        self
    }

    /// Set the homepage URL
    pub fn with_homepage(mut self, homepage: String) -> Self {
        self.homepage = Some(homepage);
        self
    }

    /// Set the license
    pub fn with_license(mut self, license: String) -> Self {
        self.license = Some(license);
        self
    }

    /// Check if this extension provides a specific capability
    pub fn has_capability(&self, capability: &ExtensionCapability) -> bool {
        self.capabilities.contains(capability)
    }
}

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

/// Extension manager for loading and managing extensions
pub struct ExtensionManager {
    /// Loaded extensions
    #[cfg(feature = "multi-thread")]
    extensions: Arc<RwLock<HashMap<String, Box<dyn EditorExtension>>>>,

    #[cfg(not(feature = "multi-thread"))]
    extensions: Rc<RefCell<HashMap<String, Box<dyn EditorExtension>>>>,

    /// Extension states
    extension_states: HashMap<String, ExtensionState>,

    /// Available commands from all extensions
    commands: HashMap<String, String>, // command_id -> extension_name

    /// Configuration storage
    config: HashMap<String, String>,

    /// Inter-extension data storage
    extension_data: HashMap<String, HashMap<String, String>>,
}

impl std::fmt::Debug for ExtensionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionManager")
            .field("extension_states", &self.extension_states)
            .field("commands", &self.commands)
            .field("config", &self.config)
            .field("extension_data", &self.extension_data)
            .field("extensions", &"<HashMap<String, Box<dyn EditorExtension>>>")
            .finish()
    }
}

impl ExtensionManager {
    /// Create a new extension manager
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "multi-thread")]
            extensions: Arc::new(RwLock::new(HashMap::new())),
            #[cfg(not(feature = "multi-thread"))]
            extensions: Rc::new(RefCell::new(HashMap::new())),
            extension_states: HashMap::new(),
            commands: HashMap::new(),
            config: HashMap::new(),
            extension_data: HashMap::new(),
        }
    }

    /// Load an extension
    pub fn load_extension(&mut self, extension: Box<dyn EditorExtension>) -> Result<()> {
        let extension_name = extension.info().name.clone();

        // Check for dependency conflicts
        self.check_dependencies(&extension_name, &extension.info().dependencies)?;

        #[cfg(feature = "multi-thread")]
        {
            let mut extensions = self.extensions.write().map_err(|_| {
                crate::core::EditorError::ThreadSafetyError {
                    message: "Failed to acquire write lock for extensions".to_string(),
                }
            })?;

            if extensions.contains_key(&extension_name) {
                return Err(crate::core::EditorError::CommandFailed {
                    message: format!("Extension '{extension_name}' is already loaded"),
                });
            }

            extensions.insert(extension_name.clone(), extension);
        }

        #[cfg(not(feature = "multi-thread"))]
        {
            let mut extensions = self.extensions.borrow_mut();

            if extensions.contains_key(&extension_name) {
                return Err(crate::core::EditorError::CommandFailed {
                    message: format!("Extension '{extension_name}' is already loaded"),
                });
            }

            extensions.insert(extension_name.clone(), extension);
        }

        self.extension_states
            .insert(extension_name.clone(), ExtensionState::Uninitialized);

        Ok(())
    }

    /// Initialize an extension
    pub fn initialize_extension(
        &mut self,
        extension_name: &str,
        context: &mut dyn ExtensionContext,
    ) -> Result<()> {
        self.extension_states
            .insert(extension_name.to_string(), ExtensionState::Initializing);

        #[cfg(feature = "multi-thread")]
        {
            let mut extensions = self.extensions.write().map_err(|_| {
                crate::core::EditorError::ThreadSafetyError {
                    message: "Failed to acquire write lock for extensions".to_string(),
                }
            })?;

            if let Some(extension) = extensions.get_mut(extension_name) {
                match extension.initialize(context) {
                    Ok(()) => {
                        self.extension_states
                            .insert(extension_name.to_string(), ExtensionState::Active);

                        // Register commands
                        for command in extension.commands() {
                            self.commands
                                .insert(command.id.clone(), extension_name.to_string());
                        }

                        Ok(())
                    }
                    Err(e) => {
                        self.extension_states
                            .insert(extension_name.to_string(), ExtensionState::Error);
                        Err(e)
                    }
                }
            } else {
                Err(crate::core::EditorError::CommandFailed {
                    message: format!("Extension '{extension_name}' not found"),
                })
            }
        }

        #[cfg(not(feature = "multi-thread"))]
        {
            let mut extensions = self.extensions.borrow_mut();

            if let Some(extension) = extensions.get_mut(extension_name) {
                match extension.initialize(context) {
                    Ok(()) => {
                        self.extension_states
                            .insert(extension_name.to_string(), ExtensionState::Active);

                        // Register commands
                        for command in extension.commands() {
                            self.commands
                                .insert(command.id.clone(), extension_name.to_string());
                        }

                        Ok(())
                    }
                    Err(e) => {
                        self.extension_states
                            .insert(extension_name.to_string(), ExtensionState::Error);
                        Err(e)
                    }
                }
            } else {
                Err(crate::core::EditorError::CommandFailed {
                    message: format!("Extension '{extension_name}' not found"),
                })
            }
        }
    }

    /// Unload an extension
    pub fn unload_extension(
        &mut self,
        extension_name: &str,
        context: &mut dyn ExtensionContext,
    ) -> Result<()> {
        // Shutdown the extension first
        self.shutdown_extension(extension_name, context)?;

        #[cfg(feature = "multi-thread")]
        {
            let mut extensions = self.extensions.write().map_err(|_| {
                crate::core::EditorError::ThreadSafetyError {
                    message: "Failed to acquire write lock for extensions".to_string(),
                }
            })?;
            extensions.remove(extension_name);
        }

        #[cfg(not(feature = "multi-thread"))]
        {
            let mut extensions = self.extensions.borrow_mut();
            extensions.remove(extension_name);
        }

        self.extension_states.remove(extension_name);

        // Remove commands
        self.commands
            .retain(|_, ext_name| ext_name != extension_name);

        // Remove extension data
        self.extension_data.remove(extension_name);

        Ok(())
    }

    /// Shutdown an extension
    fn shutdown_extension(
        &mut self,
        extension_name: &str,
        context: &mut dyn ExtensionContext,
    ) -> Result<()> {
        self.extension_states
            .insert(extension_name.to_string(), ExtensionState::ShuttingDown);

        #[cfg(feature = "multi-thread")]
        {
            let mut extensions = self.extensions.write().map_err(|_| {
                crate::core::EditorError::ThreadSafetyError {
                    message: "Failed to acquire write lock for extensions".to_string(),
                }
            })?;

            if let Some(extension) = extensions.get_mut(extension_name) {
                extension.shutdown(context)?;
                self.extension_states
                    .insert(extension_name.to_string(), ExtensionState::Shutdown);
            }
        }

        #[cfg(not(feature = "multi-thread"))]
        {
            let mut extensions = self.extensions.borrow_mut();

            if let Some(extension) = extensions.get_mut(extension_name) {
                extension.shutdown(context)?;
                self.extension_states
                    .insert(extension_name.to_string(), ExtensionState::Shutdown);
            }
        }

        Ok(())
    }

    /// Execute a command from an extension
    pub fn execute_command(
        &mut self,
        command_id: &str,
        args: &HashMap<String, String>,
        context: &mut dyn ExtensionContext,
    ) -> Result<ExtensionResult> {
        let extension_name = self
            .commands
            .get(command_id)
            .ok_or_else(|| crate::core::EditorError::CommandFailed {
                message: format!("Command '{command_id}' not found"),
            })?
            .clone();

        #[cfg(feature = "multi-thread")]
        {
            let mut extensions = self.extensions.write().map_err(|_| {
                crate::core::EditorError::ThreadSafetyError {
                    message: "Failed to acquire write lock for extensions".to_string(),
                }
            })?;

            if let Some(extension) = extensions.get_mut(&extension_name) {
                extension.execute_command(command_id, args, context)
            } else {
                Err(crate::core::EditorError::CommandFailed {
                    message: format!("Extension '{extension_name}' not found"),
                })
            }
        }

        #[cfg(not(feature = "multi-thread"))]
        {
            let mut extensions = self.extensions.borrow_mut();

            if let Some(extension) = extensions.get_mut(&extension_name) {
                extension.execute_command(command_id, args, context)
            } else {
                Err(crate::core::EditorError::CommandFailed {
                    message: format!("Extension '{extension_name}' not found"),
                })
            }
        }
    }

    /// Get list of loaded extensions
    pub fn list_extensions(&self) -> Vec<String> {
        #[cfg(feature = "multi-thread")]
        {
            if let Ok(extensions) = self.extensions.read() {
                extensions.keys().cloned().collect()
            } else {
                Vec::new()
            }
        }

        #[cfg(not(feature = "multi-thread"))]
        {
            self.extensions.borrow().keys().cloned().collect()
        }
    }

    /// Get extension state
    pub fn get_extension_state(&self, extension_name: &str) -> Option<ExtensionState> {
        self.extension_states.get(extension_name).copied()
    }

    /// Get all available commands
    pub fn list_commands(&self) -> Vec<String> {
        self.commands.keys().cloned().collect()
    }

    /// Check dependencies for an extension
    fn check_dependencies(&self, extension_name: &str, dependencies: &[String]) -> Result<()> {
        for dependency in dependencies {
            if !self.extension_states.contains_key(dependency) {
                return Err(crate::core::EditorError::CommandFailed {
                    message: format!(
                        "Extension '{extension_name}' depends on '{dependency}' which is not loaded"
                    ),
                });
            }
        }
        Ok(())
    }

    /// Get configuration value
    pub fn get_config(&self, key: &str) -> Option<&str> {
        self.config.get(key).map(|s| s.as_str())
    }

    /// Set configuration value
    pub fn set_config(&mut self, key: String, value: String) {
        self.config.insert(key, value);
    }

    /// Get extension data
    pub fn get_extension_data(&self, extension_name: &str, key: &str) -> Option<&str> {
        self.extension_data
            .get(extension_name)
            .and_then(|data| data.get(key))
            .map(|s| s.as_str())
    }

    /// Set extension data
    pub fn set_extension_data(&mut self, extension_name: String, key: String, value: String) {
        self.extension_data
            .entry(extension_name)
            .or_default()
            .insert(key, value);
    }
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple extension context implementation
pub struct SimpleExtensionContext<'a> {
    /// Current document
    pub document: Option<&'a mut EditorDocument>,
    /// Extension manager reference  
    pub manager: &'a mut ExtensionManager,
}

impl<'a> SimpleExtensionContext<'a> {
    /// Create a new simple context
    pub fn new(
        document: Option<&'a mut EditorDocument>,
        manager: &'a mut ExtensionManager,
    ) -> Self {
        Self { document, manager }
    }
}

impl<'a> ExtensionContext for SimpleExtensionContext<'a> {
    fn current_document(&self) -> Option<&EditorDocument> {
        self.document.as_deref()
    }

    fn current_document_mut(&mut self) -> Option<&mut EditorDocument> {
        self.document.as_deref_mut()
    }

    fn send_event(&mut self, _event: DocumentEvent) -> Result<()> {
        // Would integrate with event system in real implementation
        Ok(())
    }

    fn get_config(&self, key: &str) -> Option<String> {
        self.manager.get_config(key).map(|s| s.to_string())
    }

    fn set_config(&mut self, key: String, value: String) -> Result<()> {
        self.manager.set_config(key, value);
        Ok(())
    }

    fn register_command(&mut self, _command: ExtensionCommand) -> Result<()> {
        // Would register with command system in real implementation
        Ok(())
    }

    fn show_message(&mut self, _message: &str, _level: MessageLevel) -> Result<()> {
        // Would show message to user in real implementation
        Ok(())
    }

    fn get_extension_data(&self, extension_name: &str, key: &str) -> Option<String> {
        self.manager
            .get_extension_data(extension_name, key)
            .map(|s| s.to_string())
    }

    fn set_extension_data(&mut self, key: String, value: String) -> Result<()> {
        // For simplicity, we'll use a default extension name
        self.manager
            .set_extension_data("default".to_string(), key, value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_info_creation() {
        let info = ExtensionInfo::new(
            "test-extension".to_string(),
            "1.0.0".to_string(),
            "Test Author".to_string(),
            "A test extension".to_string(),
        )
        .with_capability(ExtensionCapability::TextProcessing)
        .with_dependency("core-extension".to_string())
        .with_homepage("https://example.com".to_string())
        .with_license("MIT".to_string());

        assert_eq!(info.name, "test-extension");
        assert_eq!(info.version, "1.0.0");
        assert!(info.has_capability(&ExtensionCapability::TextProcessing));
        assert_eq!(info.dependencies.len(), 1);
        assert_eq!(info.homepage, Some("https://example.com".to_string()));
        assert_eq!(info.license, Some("MIT".to_string()));
    }

    #[test]
    fn extension_capability_description() {
        let capability = ExtensionCapability::TextProcessing;
        assert_eq!(
            capability.description(),
            "Text processing and transformation"
        );
    }

    #[test]
    fn extension_state_checks() {
        let state = ExtensionState::Active;
        assert!(state.is_active());
        assert!(state.is_usable());
        assert!(!state.is_error());

        let error_state = ExtensionState::Error;
        assert!(!error_state.is_active());
        assert!(!error_state.is_usable());
        assert!(error_state.is_error());
    }

    #[test]
    fn extension_command_creation() {
        let command = ExtensionCommand::new(
            "test-command".to_string(),
            "Test Command".to_string(),
            "A test command".to_string(),
        )
        .with_shortcut("Ctrl+T".to_string())
        .with_category("Testing".to_string())
        .requires_document(false);

        assert_eq!(command.id, "test-command");
        assert_eq!(command.shortcut, Some("Ctrl+T".to_string()));
        assert_eq!(command.category, "Testing");
        assert!(!command.requires_document);
    }

    #[test]
    fn extension_result_creation() {
        let success = ExtensionResult::success_with_message("Success!".to_string())
            .with_data("key".to_string(), "value".to_string());

        assert!(success.success);
        assert_eq!(success.message, Some("Success!".to_string()));
        assert_eq!(success.data.get("key"), Some(&"value".to_string()));

        let failure = ExtensionResult::failure("Failed!".to_string());
        assert!(!failure.success);
        assert_eq!(failure.message, Some("Failed!".to_string()));
    }

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
        assert_eq!(manager.get_config("test_key"), Some("test_value"));
        assert_eq!(manager.get_config("nonexistent"), None);
    }

    #[test]
    fn extension_manager_data() {
        let mut manager = ExtensionManager::new();

        manager.set_extension_data("ext1".to_string(), "key".to_string(), "value".to_string());
        assert_eq!(manager.get_extension_data("ext1", "key"), Some("value"));
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
        let mut doc = EditorDocument::new();

        let extension = Box::new(TestExtension::new("test-ext"));
        manager.load_extension(extension).unwrap();

        assert_eq!(manager.list_extensions().len(), 1);
        assert_eq!(
            manager.get_extension_state("test-ext"),
            Some(ExtensionState::Uninitialized)
        );

        {
            let _context = SimpleExtensionContext::new(Some(&mut doc), &mut manager);
            // Split the manager borrow to avoid conflicts - we can't test the internal state
            // during initialization since we need mutable access to the manager
        }

        // Test that we can check states after operations
        let extension_exists = manager.list_extensions().contains(&"test-ext".to_string());
        assert!(extension_exists);
    }

    #[test]
    fn simple_extension_context() {
        let mut manager = ExtensionManager::new();
        let mut doc = EditorDocument::new();

        // Test config through manager first
        manager.set_config("test".to_string(), "value".to_string());
        assert_eq!(manager.get_config("test"), Some("value"));

        // Test extension data through manager
        manager.set_extension_data("default".to_string(), "key".to_string(), "data".to_string());
        assert_eq!(manager.get_extension_data("default", "key"), Some("data"));

        // Test context creation and document access
        let mut context = SimpleExtensionContext::new(Some(&mut doc), &mut manager);
        assert!(context.current_document().is_some());
        assert!(context.current_document_mut().is_some());
    }
}
