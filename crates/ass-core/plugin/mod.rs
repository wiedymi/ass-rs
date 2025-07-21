//! Plugin system for extending ASS parsing and rendering capabilities.
//!
//! This module provides a trait-based extension system allowing custom tag handlers,
//! section processors, and rendering backends to be registered at runtime. Designed
//! for zero-allocation hot paths and efficient lookup via optimized hash maps.
//!
//! ## Architecture
//!
//! - **TagHandler**: Process custom override tags (e.g., `{\custom}`)
//! - **SectionProcessor**: Handle non-standard sections (e.g., `[Aegisub Project]`)
//! - **ExtensionRegistry**: Central registry for all extensions
//!
//! ## Example
//!
//! ```rust
//! use ass_core::plugin::{ExtensionRegistry, TagHandler, TagResult};
//!
//! struct CustomColorTag;
//!
//! impl TagHandler for CustomColorTag {
//!     fn name(&self) -> &'static str { "customcolor" }
//!
//!     fn process(&self, args: &str) -> TagResult {
//!         // Custom color processing logic
//!         TagResult::Processed
//!     }
//! }
//!
//! let mut registry = ExtensionRegistry::new();
//! registry.register_tag_handler(Box::new(CustomColorTag));
//! ```

use alloc::{boxed::Box, string::String, vec::Vec};
use core::fmt;

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use hashbrown::HashMap;

/// Result of tag processing operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagResult {
    /// Tag was successfully processed
    Processed,
    /// Tag was ignored (not handled by this processor)
    Ignored,
    /// Tag processing failed with error message
    Failed(String),
}

/// Result of section processing operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionResult {
    /// Section was successfully processed
    Processed,
    /// Section was ignored (not handled by this processor)
    Ignored,
    /// Section processing failed with error message
    Failed(String),
}

/// Trait for handling custom ASS override tags
///
/// Implementors can process custom tags that extend standard ASS functionality.
/// Tag handlers are called during parsing when unknown tags are encountered.
pub trait TagHandler: Send + Sync {
    /// Unique name identifier for this tag handler
    fn name(&self) -> &'static str;

    /// Process a tag with its arguments
    ///
    /// # Arguments
    /// * `args` - Raw tag arguments as string slice
    ///
    /// # Returns
    /// * `TagResult::Processed` - Tag was handled successfully
    /// * `TagResult::Ignored` - Tag not recognized by this handler
    /// * `TagResult::Failed` - Error occurred during processing
    fn process(&self, args: &str) -> TagResult;

    /// Optional validation of tag arguments during parsing
    fn validate(&self, args: &str) -> bool {
        !args.is_empty()
    }
}

/// Trait for handling custom ASS sections
///
/// Implementors can process non-standard sections that extend ASS functionality.
/// Section processors are called when unknown section headers are encountered.
pub trait SectionProcessor: Send + Sync {
    /// Unique name identifier for this section processor
    fn name(&self) -> &'static str;

    /// Process section header and content lines
    ///
    /// # Arguments
    /// * `header` - Section header (e.g., "Aegisub Project")
    /// * `lines` - All lines belonging to this section
    ///
    /// # Returns
    /// * `SectionResult::Processed` - Section was handled successfully
    /// * `SectionResult::Ignored` - Section not recognized by this processor
    /// * `SectionResult::Failed` - Error occurred during processing
    fn process(&self, header: &str, lines: &[&str]) -> SectionResult;

    /// Optional validation of section format
    fn validate(&self, header: &str, lines: &[&str]) -> bool {
        !header.is_empty() && !lines.is_empty()
    }
}

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
                write!(f, "Handler '{}' already registered", name)
            }
            Self::HandlerNotFound(name) => {
                write!(f, "Handler '{}' not found", name)
            }
            Self::ProcessingFailed(msg) => {
                write!(f, "Plugin processing failed: {}", msg)
            }
            Self::InvalidConfig(msg) => {
                write!(f, "Invalid plugin configuration: {}", msg)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for PluginError {}

/// Central registry for all ASS format extensions
///
/// Manages registration and lookup of tag handlers and section processors.
/// Optimized for fast lookup during parsing with minimal memory overhead.
#[derive(Default)]
pub struct ExtensionRegistry {
    tag_handlers: HashMap<String, Box<dyn TagHandler>>,
    section_processors: HashMap<String, Box<dyn SectionProcessor>>,
}

impl ExtensionRegistry {
    /// Create a new empty extension registry
    pub fn new() -> Self {
        Self {
            tag_handlers: HashMap::default(),
            section_processors: HashMap::default(),
        }
    }

    /// Register a new tag handler
    ///
    /// # Arguments
    /// * `handler` - Boxed tag handler implementation
    ///
    /// # Errors
    /// Returns `PluginError::DuplicateHandler` if handler name already exists
    pub fn register_tag_handler(&mut self, handler: Box<dyn TagHandler>) -> Result<()> {
        let name = handler.name().to_string();

        if self.tag_handlers.contains_key(&name) {
            return Err(PluginError::DuplicateHandler(name));
        }

        self.tag_handlers.insert(name, handler);
        Ok(())
    }

    /// Register a new section processor
    ///
    /// # Arguments
    /// * `processor` - Boxed section processor implementation
    ///
    /// # Errors
    /// Returns `PluginError::DuplicateHandler` if processor name already exists
    pub fn register_section_processor(
        &mut self,
        processor: Box<dyn SectionProcessor>,
    ) -> Result<()> {
        let name = processor.name().to_string();

        if self.section_processors.contains_key(&name) {
            return Err(PluginError::DuplicateHandler(name));
        }

        self.section_processors.insert(name, processor);
        Ok(())
    }

    /// Process a tag using registered handlers
    ///
    /// # Arguments
    /// * `tag_name` - Name of the tag to process
    /// * `args` - Tag arguments as string slice
    ///
    /// # Returns
    /// * `Some(TagResult)` - If a handler was found and executed
    /// * `None` - If no handler was registered for this tag
    pub fn process_tag(&self, tag_name: &str, args: &str) -> Option<TagResult> {
        self.tag_handlers
            .get(tag_name)
            .map(|handler| handler.process(args))
    }

    /// Process a section using registered processors
    ///
    /// # Arguments
    /// * `section_name` - Name of the section to process
    /// * `header` - Section header line
    /// * `lines` - All lines in the section
    ///
    /// # Returns
    /// * `Some(SectionResult)` - If a processor was found and executed
    /// * `None` - If no processor was registered for this section
    pub fn process_section(
        &self,
        section_name: &str,
        header: &str,
        lines: &[&str],
    ) -> Option<SectionResult> {
        self.section_processors
            .get(section_name)
            .map(|processor| processor.process(header, lines))
    }

    /// Get list of registered tag handler names
    pub fn tag_handler_names(&self) -> Vec<&str> {
        self.tag_handlers.keys().map(|s| s.as_str()).collect()
    }

    /// Get list of registered section processor names
    pub fn section_processor_names(&self) -> Vec<&str> {
        self.section_processors.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a tag handler is registered
    pub fn has_tag_handler(&self, name: &str) -> bool {
        self.tag_handlers.contains_key(name)
    }

    /// Check if a section processor is registered
    pub fn has_section_processor(&self, name: &str) -> bool {
        self.section_processors.contains_key(name)
    }

    /// Remove a tag handler by name
    ///
    /// # Returns
    /// * `Some(handler)` - If handler was found and removed
    /// * `None` - If no handler with that name was registered
    pub fn remove_tag_handler(&mut self, name: &str) -> Option<Box<dyn TagHandler>> {
        self.tag_handlers.remove(name)
    }

    /// Remove a section processor by name
    ///
    /// # Returns
    /// * `Some(processor)` - If processor was found and removed
    /// * `None` - If no processor with that name was registered
    pub fn remove_section_processor(&mut self, name: &str) -> Option<Box<dyn SectionProcessor>> {
        self.section_processors.remove(name)
    }

    /// Clear all registered handlers and processors
    pub fn clear(&mut self) {
        self.tag_handlers.clear();
        self.section_processors.clear();
    }

    /// Get total number of registered extensions
    pub fn extension_count(&self) -> usize {
        self.tag_handlers.len() + self.section_processors.len()
    }
}

impl fmt::Debug for ExtensionRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtensionRegistry")
            .field("tag_handlers", &self.tag_handler_names())
            .field("section_processors", &self.section_processor_names())
            .finish()
    }
}

/// Result type for plugin operations
pub type Result<T> = core::result::Result<T, PluginError>;
