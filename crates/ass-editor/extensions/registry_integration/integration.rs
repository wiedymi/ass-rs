//! Core `RegistryIntegration` type with construction, accessors, and custom
//! handler registration.
//!
//! Holds the ass-core [`ExtensionRegistry`] together with the editor extensions
//! that provide tag handlers and section processors. Built-in registration
//! lives in the sibling `builtins` module.

use crate::core::{EditorError, Result};
use crate::extensions::EditorExtension;
use ass_core::plugin::{ExtensionRegistry, SectionProcessor, TagHandler};

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, format, string::String, vec::Vec};

/// Wrapper that connects editor extensions to ass-core's ExtensionRegistry
pub struct RegistryIntegration {
    /// The ass-core extension registry
    pub(super) registry: ExtensionRegistry,
    /// Editor extensions that provide tag handlers
    pub(super) tag_providers: Vec<Box<dyn EditorExtension>>,
    /// Editor extensions that provide section processors
    pub(super) section_providers: Vec<Box<dyn EditorExtension>>,
}

impl core::fmt::Debug for RegistryIntegration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RegistryIntegration")
            .field("registry", &self.registry)
            .field(
                "tag_providers",
                &format!("{} providers", self.tag_providers.len()),
            )
            .field(
                "section_providers",
                &format!("{} providers", self.section_providers.len()),
            )
            .finish()
    }
}

impl RegistryIntegration {
    /// Create a new registry integration
    pub fn new() -> Self {
        Self {
            registry: ExtensionRegistry::new(),
            tag_providers: Vec::new(),
            section_providers: Vec::new(),
        }
    }

    /// Get the underlying ExtensionRegistry for use in parsing
    pub fn registry(&self) -> &ExtensionRegistry {
        &self.registry
    }

    /// Get mutable access to the registry
    pub fn registry_mut(&mut self) -> &mut ExtensionRegistry {
        &mut self.registry
    }

    /// Register a custom tag handler from an editor extension
    pub fn register_custom_tag_handler(
        &mut self,
        extension_name: String,
        handler: Box<dyn TagHandler>,
    ) -> Result<()> {
        self.registry
            .register_tag_handler(handler)
            .map_err(|e| EditorError::ExtensionError {
                extension: extension_name,
                message: format!("Failed to register tag handler: {e}"),
            })
    }

    /// Register a custom section processor from an editor extension
    pub fn register_custom_section_processor(
        &mut self,
        extension_name: String,
        processor: Box<dyn SectionProcessor>,
    ) -> Result<()> {
        self.registry
            .register_section_processor(processor)
            .map_err(|e| EditorError::ExtensionError {
                extension: extension_name,
                message: format!("Failed to register section processor: {e}"),
            })
    }
}

impl Default for RegistryIntegration {
    fn default() -> Self {
        Self::new()
    }
}
