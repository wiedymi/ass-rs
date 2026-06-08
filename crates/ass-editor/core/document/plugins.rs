//! Extension-registry integration for the document
//!
//! Lets a document own an `ExtensionRegistry`, parse with custom handlers,
//! and register custom tag handlers and section processors.

use super::EditorDocument;
use crate::core::errors::{EditorError, Result};

#[cfg(feature = "std")]
use std::sync::Arc;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

impl EditorDocument {
    /// Initialize the extension registry with built-in handlers
    pub fn initialize_registry(&mut self) -> Result<()> {
        use crate::extensions::registry_integration::RegistryIntegration;

        let mut integration = RegistryIntegration::new();

        // Register all built-in extensions using the function from the builtin module
        crate::extensions::builtin::register_builtin_extensions(&mut integration)?;

        self.registry_integration = Some(Arc::new(integration));
        Ok(())
    }

    /// Get the extension registry for use in parsing
    pub fn registry(&self) -> Option<&ass_core::plugin::ExtensionRegistry> {
        self.registry_integration
            .as_ref()
            .map(|integration| integration.registry())
    }

    /// Parse the document content with extension support and process it with a callback
    ///
    /// Since Script<'a> requires the source text to outlive it, this method uses a callback
    /// pattern to process the script while the content is still in scope.
    pub fn parse_with_extensions<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&ass_core::parser::Script) -> R,
    {
        let content = self.text();

        if let Some(integration) = &self.registry_integration {
            // Parse with the extension registry using the builder pattern
            let script = ass_core::parser::Script::builder()
                .with_registry(integration.registry())
                .parse(&content)
                .map_err(EditorError::Core)?;
            Ok(f(&script))
        } else {
            // No registry, parse normally
            let script = ass_core::parser::Script::parse(&content).map_err(EditorError::Core)?;
            Ok(f(&script))
        }
    }

    /// Register a custom tag handler
    pub fn register_tag_handler(
        &mut self,
        extension_name: String,
        handler: Box<dyn ass_core::plugin::TagHandler>,
    ) -> Result<()> {
        if self.registry_integration.is_none() {
            self.initialize_registry()?;
        }

        let registry_ref =
            self.registry_integration
                .as_mut()
                .ok_or_else(|| EditorError::ExtensionError {
                    extension: extension_name.clone(),
                    message: "Registry integration not available".to_string(),
                })?;

        if let Some(integration) = Arc::get_mut(registry_ref) {
            integration.register_custom_tag_handler(extension_name, handler)
        } else {
            Err(EditorError::ExtensionError {
                extension: extension_name,
                message: "Cannot modify shared registry integration".to_string(),
            })
        }
    }

    /// Register a custom section processor
    pub fn register_section_processor(
        &mut self,
        extension_name: String,
        processor: Box<dyn ass_core::plugin::SectionProcessor>,
    ) -> Result<()> {
        if self.registry_integration.is_none() {
            self.initialize_registry()?;
        }

        let registry_ref =
            self.registry_integration
                .as_mut()
                .ok_or_else(|| EditorError::ExtensionError {
                    extension: extension_name.clone(),
                    message: "Registry integration not available".to_string(),
                })?;

        if let Some(integration) = Arc::get_mut(registry_ref) {
            integration.register_custom_section_processor(extension_name, processor)
        } else {
            Err(EditorError::ExtensionError {
                extension: extension_name,
                message: "Cannot modify shared registry integration".to_string(),
            })
        }
    }
}
