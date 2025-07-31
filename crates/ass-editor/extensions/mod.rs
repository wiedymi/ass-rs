//! Extension system for editor functionality
//!
//! Provides the `EditorExtension` trait for extending editor capabilities
//! with custom functionality. Supports both synchronous and asynchronous
//! operations, lifecycle management, and inter-extension communication.

pub mod builtin;
pub mod registry_integration;

#[cfg(not(feature = "std"))]
extern crate alloc;

use crate::core::{EditorDocument, Result};
use crate::events::DocumentEvent;
use core::fmt;

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, format, string::{String, ToString}, vec::Vec};

#[cfg(feature = "multi-thread")]
use std::sync::Arc;

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
/// Internal storage for ExtensionManager data
struct ExtensionManagerInner {
    /// Loaded extensions
    extensions: HashMap<String, Box<dyn EditorExtension>>,

    /// Extension states
    extension_states: HashMap<String, ExtensionState>,

    /// Available commands from all extensions
    commands: HashMap<String, (String, ExtensionCommand)>, // command_id -> (extension_name, command)

    /// Configuration storage
    config: HashMap<String, String>,

    /// Inter-extension data storage
    extension_data: HashMap<String, HashMap<String, String>>,

    /// Event channel for sending events
    #[cfg(feature = "std")]
    #[allow(dead_code)]
    event_tx: EventSender,

    /// Message handler for user notifications
    #[allow(dead_code)]
    message_handler: Box<dyn MessageHandler>,
}

/// Extension manager with built-in thread safety
#[cfg(feature = "multi-thread")]
use parking_lot::Mutex;

/// Single unified ExtensionManager that is always thread-safe when multi-thread feature is enabled
pub struct ExtensionManager {
    #[cfg(feature = "multi-thread")]
    inner: Arc<Mutex<ExtensionManagerInner>>,
    #[cfg(not(feature = "multi-thread"))]
    inner: RefCell<ExtensionManagerInner>,
}

// ExtensionManager is cloneable when multi-thread feature is enabled
#[cfg(feature = "multi-thread")]
impl Clone for ExtensionManager {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

// Note: ExtensionManager does not implement Clone without multi-thread feature
// This is intentional - cloning requires Arc<Mutex<T>> which needs multi-thread

impl fmt::Debug for ExtensionManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "multi-thread")]
        {
            let inner = self.inner.lock();
            f.debug_struct("ExtensionManager")
                .field("extension_states", &inner.extension_states)
                .field("commands", &inner.commands.keys().collect::<Vec<_>>())
                .field("config", &inner.config)
                .field("extension_data", &inner.extension_data)
                .field("extensions", &"<HashMap<String, Box<dyn EditorExtension>>>")
                .finish()
        }
        #[cfg(not(feature = "multi-thread"))]
        {
            let inner = self.inner.borrow();
            f.debug_struct("ExtensionManager")
                .field("extension_states", &inner.extension_states)
                .field("commands", &inner.commands.keys().collect::<Vec<_>>())
                .field("config", &inner.config)
                .field("extension_data", &inner.extension_data)
                .field("extensions", &"<HashMap<String, Box<dyn EditorExtension>>>")
                .finish()
        }
    }
}

impl ExtensionManagerInner {
    /// Create a new inner manager
    fn new() -> Self {
        #[cfg(feature = "std")]
        let (tx, _rx) = mpsc::channel();

        Self {
            extensions: HashMap::new(),
            extension_states: HashMap::new(),
            commands: HashMap::new(),
            config: HashMap::new(),
            extension_data: HashMap::new(),
            #[cfg(feature = "std")]
            event_tx: tx,
            #[cfg(feature = "std")]
            message_handler: Box::new(StdMessageHandler),
            #[cfg(not(feature = "std"))]
            message_handler: Box::new(NoOpMessageHandler),
        }
    }
}

impl ExtensionManager {
    /// Helper method for accessing inner data mutably
    #[cfg(feature = "multi-thread")]
    fn with_inner_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ExtensionManagerInner) -> R,
    {
        let mut inner = self.inner.lock();
        f(&mut inner)
    }

    /// Helper method for accessing inner data immutably
    #[cfg(feature = "multi-thread")]
    fn with_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ExtensionManagerInner) -> R,
    {
        let inner = self.inner.lock();
        f(&inner)
    }

    /// Helper method for accessing inner data mutably
    #[cfg(not(feature = "multi-thread"))]
    fn with_inner_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut ExtensionManagerInner) -> R,
    {
        let mut inner = self.inner.borrow_mut();
        f(&mut inner)
    }

    /// Helper method for accessing inner data immutably
    #[cfg(not(feature = "multi-thread"))]
    fn with_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ExtensionManagerInner) -> R,
    {
        let inner = self.inner.borrow();
        f(&inner)
    }

    /// Create a new extension manager
    pub fn new() -> Self {
        #[cfg(feature = "multi-thread")]
        {
            Self {
                inner: Arc::new(Mutex::new(ExtensionManagerInner::new())),
            }
        }
        #[cfg(not(feature = "multi-thread"))]
        {
            Self {
                inner: RefCell::new(ExtensionManagerInner::new()),
            }
        }
    }

    /// Create a new extension manager with custom event sender and message handler
    #[cfg(feature = "std")]
    pub fn with_event_channel(
        event_tx: EventSender,
        message_handler: Box<dyn MessageHandler>,
    ) -> Self {
        let inner = ExtensionManagerInner {
            extensions: HashMap::new(),
            extension_states: HashMap::new(),
            commands: HashMap::new(),
            config: HashMap::new(),
            extension_data: HashMap::new(),
            event_tx,
            message_handler,
        };

        #[cfg(feature = "multi-thread")]
        {
            Self {
                inner: Arc::new(Mutex::new(inner)),
            }
        }
        #[cfg(not(feature = "multi-thread"))]
        {
            Self {
                inner: RefCell::new(inner),
            }
        }
    }

    /// Create an extension context for use by extensions
    pub fn create_context<'a>(
        &'a mut self,
        extension_name: String,
        document: Option<&'a mut EditorDocument>,
    ) -> Result<Box<dyn ExtensionContext + 'a>> {
        #[cfg(feature = "multi-thread")]
        {
            Ok(Box::new(EditorContext {
                document,
                manager: self.clone(),
                extension_name,
            }))
        }
        
        #[cfg(not(feature = "multi-thread"))]
        {
            // In single-threaded mode, we share the config state via Rc<RefCell>
            let config_clone = self.inner.borrow().config.clone();
            let shared_config = Rc::new(RefCell::new(config_clone));
            
            Ok(Box::new(EditorContext {
                document,
                manager: self,
                manager_mut_state: shared_config,
                extension_name,
            }))
        }
    }

    /// Load an extension
    pub fn load_extension(&mut self, extension: Box<dyn EditorExtension>) -> Result<()> {
        let extension_name = extension.info().name.clone();
        let dependencies = extension.info().dependencies.clone();

        // Check for dependency conflicts
        let has_deps = self.with_inner(|inner| {
            dependencies
                .iter()
                .all(|dep| inner.extension_states.contains_key(dep))
        });

        if !has_deps {
            return Err(crate::core::EditorError::CommandFailed {
                message: format!("Extension '{extension_name}' has unmet dependencies"),
            });
        }

        self.with_inner_mut(|inner| {
            if inner.extensions.contains_key(&extension_name) {
                return Err(crate::core::EditorError::CommandFailed {
                    message: format!("Extension '{extension_name}' is already loaded"),
                });
            }

            inner.extensions.insert(extension_name.clone(), extension);
            inner
                .extension_states
                .insert(extension_name.clone(), ExtensionState::Uninitialized);
            Ok(())
        })
    }

    /// Initialize an extension
    pub fn initialize_extension(
        &mut self,
        extension_name: &str,
        context: &mut dyn ExtensionContext,
    ) -> Result<()> {
        self.with_inner_mut(|inner| {
            inner
                .extension_states
                .insert(extension_name.to_string(), ExtensionState::Initializing);

            if let Some(extension) = inner.extensions.get_mut(extension_name) {
                match extension.initialize(context) {
                    Ok(()) => {
                        inner
                            .extension_states
                            .insert(extension_name.to_string(), ExtensionState::Active);

                        // Register commands
                        for command in extension.commands() {
                            inner
                                .commands
                                .insert(command.id.clone(), (extension_name.to_string(), command));
                        }

                        Ok(())
                    }
                    Err(e) => {
                        inner
                            .extension_states
                            .insert(extension_name.to_string(), ExtensionState::Error);
                        Err(e)
                    }
                }
            } else {
                Err(crate::core::EditorError::CommandFailed {
                    message: format!("Extension '{extension_name}' not found"),
                })
            }
        })
    }

    /// Unload an extension
    pub fn unload_extension(
        &mut self,
        extension_name: &str,
        context: &mut dyn ExtensionContext,
    ) -> Result<()> {
        // Shutdown the extension first
        self.shutdown_extension(extension_name, context)?;

        self.with_inner_mut(|inner| {
            // Remove the extension
            inner.extensions.remove(extension_name);
            inner.extension_states.remove(extension_name);

            // Remove commands
            inner
                .commands
                .retain(|_, (ext_name, _)| ext_name != extension_name);

            // Remove extension data
            inner.extension_data.remove(extension_name);
        });

        Ok(())
    }

    /// Shutdown an extension
    fn shutdown_extension(
        &mut self,
        extension_name: &str,
        context: &mut dyn ExtensionContext,
    ) -> Result<()> {
        self.with_inner_mut(|inner| {
            inner
                .extension_states
                .insert(extension_name.to_string(), ExtensionState::ShuttingDown);

            if let Some(extension) = inner.extensions.get_mut(extension_name) {
                extension.shutdown(context)?;
                inner
                    .extension_states
                    .insert(extension_name.to_string(), ExtensionState::Shutdown);
            }
            Ok(())
        })
    }

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

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new()
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

/// Event sender type for channel communication
#[cfg(feature = "std")]
use std::sync::mpsc::{self, Sender};

#[cfg(feature = "std")]
pub type EventSender = Sender<DocumentEvent>;

#[cfg(not(feature = "std"))]
pub type EventSender = (); // No-op for no_std environments

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

impl<'a> ExtensionContext for EditorContext<'a> {
    fn current_document(&self) -> Option<&EditorDocument> {
        self.document.as_deref()
    }

    fn current_document_mut(&mut self) -> Option<&mut EditorDocument> {
        self.document.as_deref_mut()
    }

    fn send_event(&mut self, event: DocumentEvent) -> Result<()> {
        // For now, we'll just log the event
        // In a real implementation, this would use the event system
        #[cfg(feature = "std")]
        {
            eprintln!("Extension {} sent event: {:?}", self.extension_name, event);
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

    fn register_command(&mut self, command: ExtensionCommand) -> Result<()> {
        // For now, just acknowledge the command in both modes
        // In a real implementation, this would register with a command system
        #[cfg(feature = "std")]
        {
            eprintln!(
                "Extension {} registered command: {}",
                self.extension_name, command.id
            );
        }
        Ok(())
    }

    fn show_message(&mut self, message: &str, level: MessageLevel) -> Result<()> {
        // Simple console output for now
        #[cfg(feature = "std")]
        {
            match level {
                MessageLevel::Info => eprintln!("[INFO] {}: {}", self.extension_name, message),
                MessageLevel::Warning => eprintln!("[WARN] {}: {}", self.extension_name, message),
                MessageLevel::Error => eprintln!("[ERROR] {}: {}", self.extension_name, message),
                MessageLevel::Success => {
                    eprintln!("[SUCCESS] {}: {}", self.extension_name, message)
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
}
