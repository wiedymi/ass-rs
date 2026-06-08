//! Central registry for ASS format extensions.
//!
//! Provides [`ExtensionRegistry`], which stores and dispatches registered
//! [`TagHandler`] and [`SectionProcessor`] implementations during parsing.

use super::{PluginError, Result, SectionProcessor, SectionResult, TagHandler, TagResult};
use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use core::fmt;

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use hashbrown::HashMap;

/// Central registry for all ASS format extensions
///
/// Manages registration and lookup of tag handlers and section processors.
/// Optimized for fast lookup during parsing with minimal memory overhead.
pub struct ExtensionRegistry {
    /// Registered tag handlers indexed by tag name
    tag_handlers: HashMap<String, Box<dyn TagHandler>>,
    /// Registered section processors indexed by section name
    section_processors: HashMap<String, Box<dyn SectionProcessor>>,
}

impl ExtensionRegistry {
    /// Create a new empty extension registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            tag_handlers: HashMap::new(),
            section_processors: HashMap::new(),
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn tag_handler_names(&self) -> Vec<&str> {
        self.tag_handlers.keys().map(String::as_str).collect()
    }

    /// Get list of registered section processor names
    #[must_use]
    pub fn section_processor_names(&self) -> Vec<&str> {
        self.section_processors.keys().map(String::as_str).collect()
    }

    /// Check if a tag handler is registered
    #[must_use]
    pub fn has_tag_handler(&self, name: &str) -> bool {
        self.tag_handlers.contains_key(name)
    }

    /// Check if a section processor is registered
    #[must_use]
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
    #[must_use]
    pub fn extension_count(&self) -> usize {
        self.tag_handlers.len() + self.section_processors.len()
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new()
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
