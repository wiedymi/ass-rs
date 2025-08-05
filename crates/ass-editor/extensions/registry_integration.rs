//! Integration with ass-core's ExtensionRegistry
//!
//! This module provides the glue between ass-editor's extension system and
//! ass-core's plugin system, allowing editor extensions to register tag handlers
//! and section processors that will be used during ASS parsing.

use crate::core::{EditorError, Result};
use crate::extensions::EditorExtension;
use ass_core::plugin::{ExtensionRegistry, SectionProcessor, SectionResult, TagHandler, TagResult};

#[cfg(not(feature = "std"))]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};

/// Wrapper that connects editor extensions to ass-core's ExtensionRegistry
pub struct RegistryIntegration {
    /// The ass-core extension registry
    registry: ExtensionRegistry,
    /// Editor extensions that provide tag handlers
    tag_providers: Vec<Box<dyn EditorExtension>>,
    /// Editor extensions that provide section processors
    section_providers: Vec<Box<dyn EditorExtension>>,
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

    /// Register all built-in tag handlers from ass-core
    pub fn register_builtin_handlers(&mut self) -> Result<()> {
        use ass_core::plugin::tags::*;

        // Register formatting handlers
        for handler in formatting::create_formatting_handlers() {
            self.registry.register_tag_handler(handler).map_err(|e| {
                EditorError::ExtensionError {
                    extension: "builtin".to_string(),
                    message: format!("Failed to register formatting handler: {e}"),
                }
            })?;
        }

        // Register special character handlers
        for handler in special::create_special_handlers() {
            self.registry.register_tag_handler(handler).map_err(|e| {
                EditorError::ExtensionError {
                    extension: "builtin".to_string(),
                    message: format!("Failed to register special handler: {e}"),
                }
            })?;
        }

        // Register font handlers
        for handler in font::create_font_handlers() {
            self.registry.register_tag_handler(handler).map_err(|e| {
                EditorError::ExtensionError {
                    extension: "builtin".to_string(),
                    message: format!("Failed to register font handler: {e}"),
                }
            })?;
        }

        // Register advanced handlers
        for handler in advanced::create_advanced_handlers() {
            self.registry.register_tag_handler(handler).map_err(|e| {
                EditorError::ExtensionError {
                    extension: "builtin".to_string(),
                    message: format!("Failed to register advanced handler: {e}"),
                }
            })?;
        }

        // Register alignment handlers
        for handler in alignment::create_alignment_handlers() {
            self.registry.register_tag_handler(handler).map_err(|e| {
                EditorError::ExtensionError {
                    extension: "builtin".to_string(),
                    message: format!("Failed to register alignment handler: {e}"),
                }
            })?;
        }

        // Register karaoke handlers
        for handler in karaoke::create_karaoke_handlers() {
            self.registry.register_tag_handler(handler).map_err(|e| {
                EditorError::ExtensionError {
                    extension: "builtin".to_string(),
                    message: format!("Failed to register karaoke handler: {e}"),
                }
            })?;
        }

        // Register position handlers
        for handler in position::create_position_handlers() {
            self.registry.register_tag_handler(handler).map_err(|e| {
                EditorError::ExtensionError {
                    extension: "builtin".to_string(),
                    message: format!("Failed to register position handler: {e}"),
                }
            })?;
        }

        // Register color handlers
        for handler in color::create_color_handlers() {
            self.registry.register_tag_handler(handler).map_err(|e| {
                EditorError::ExtensionError {
                    extension: "builtin".to_string(),
                    message: format!("Failed to register color handler: {e}"),
                }
            })?;
        }

        // Register transform handlers
        for handler in transform::create_transform_handlers() {
            self.registry.register_tag_handler(handler).map_err(|e| {
                EditorError::ExtensionError {
                    extension: "builtin".to_string(),
                    message: format!("Failed to register transform handler: {e}"),
                }
            })?;
        }

        // Register animation handlers
        for handler in animation::create_animation_handlers() {
            self.registry.register_tag_handler(handler).map_err(|e| {
                EditorError::ExtensionError {
                    extension: "builtin".to_string(),
                    message: format!("Failed to register animation handler: {e}"),
                }
            })?;
        }

        // Register clipping handlers
        for handler in clipping::create_clipping_handlers() {
            self.registry.register_tag_handler(handler).map_err(|e| {
                EditorError::ExtensionError {
                    extension: "builtin".to_string(),
                    message: format!("Failed to register clipping handler: {e}"),
                }
            })?;
        }

        // Register misc handlers
        for handler in misc::create_misc_handlers() {
            self.registry.register_tag_handler(handler).map_err(|e| {
                EditorError::ExtensionError {
                    extension: "builtin".to_string(),
                    message: format!("Failed to register misc handler: {e}"),
                }
            })?;
        }

        Ok(())
    }

    /// Register built-in section processors from ass-core
    pub fn register_builtin_sections(&mut self) -> Result<()> {
        use ass_core::plugin::sections::*;

        // Register Aegisub section processor
        self.registry
            .register_section_processor(Box::new(aegisub::AegisubProjectProcessor))
            .map_err(|e| EditorError::ExtensionError {
                extension: "builtin".to_string(),
                message: format!("Failed to register Aegisub section processor: {e}"),
            })?;

        Ok(())
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

/// Adapter that allows editor extensions to provide tag handlers
#[allow(dead_code)]
pub struct EditorTagHandlerAdapter {
    extension_name: String,
    tag_name: String,
    extension: Box<dyn EditorExtension>,
}

impl EditorTagHandlerAdapter {
    /// Create a new tag handler adapter
    pub fn new(
        extension_name: String,
        tag_name: String,
        extension: Box<dyn EditorExtension>,
    ) -> Self {
        Self {
            extension_name,
            tag_name,
            extension,
        }
    }
}

impl TagHandler for EditorTagHandlerAdapter {
    fn name(&self) -> &'static str {
        // This is a limitation - we need to leak the string to get a 'static lifetime
        Box::leak(self.tag_name.clone().into_boxed_str())
    }

    fn process(&self, _args: &str) -> TagResult {
        // Extensions process tags through their command system
        // This is a simplified implementation
        TagResult::Processed
    }

    fn validate(&self, args: &str) -> bool {
        !args.is_empty()
    }
}

/// Adapter that allows editor extensions to provide section processors
#[allow(dead_code)]
pub struct EditorSectionProcessorAdapter {
    extension_name: String,
    section_name: String,
    extension: Box<dyn EditorExtension>,
}

impl EditorSectionProcessorAdapter {
    /// Create a new section processor adapter
    pub fn new(
        extension_name: String,
        section_name: String,
        extension: Box<dyn EditorExtension>,
    ) -> Self {
        Self {
            extension_name,
            section_name,
            extension,
        }
    }
}

impl SectionProcessor for EditorSectionProcessorAdapter {
    fn name(&self) -> &'static str {
        // This is a limitation - we need to leak the string to get a 'static lifetime
        Box::leak(self.section_name.clone().into_boxed_str())
    }

    fn process(&self, _header: &str, _lines: &[&str]) -> SectionResult {
        // Extensions process sections through their command system
        // This is a simplified implementation
        SectionResult::Processed
    }

    fn validate(&self, header: &str, lines: &[&str]) -> bool {
        !header.is_empty() && !lines.is_empty()
    }
}

impl Default for RegistryIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;

    #[test]
    fn test_registry_integration_creation() {
        let integration = RegistryIntegration::new();
        assert!(integration.tag_providers.is_empty());
        assert!(integration.section_providers.is_empty());
    }

    #[test]
    fn test_register_builtin_handlers() {
        let mut integration = RegistryIntegration::new();

        // Should successfully register all built-in handlers
        assert!(integration.register_builtin_handlers().is_ok());
    }

    #[test]
    fn test_register_builtin_sections() {
        let mut integration = RegistryIntegration::new();

        // Should successfully register built-in section processors
        assert!(integration.register_builtin_sections().is_ok());
    }

    #[test]
    fn test_custom_tag_handler_registration() {
        use ass_core::plugin::{TagHandler, TagResult};

        struct TestTagHandler;
        impl TagHandler for TestTagHandler {
            fn name(&self) -> &'static str {
                "test"
            }

            fn process(&self, _args: &str) -> TagResult {
                TagResult::Processed
            }

            fn validate(&self, _args: &str) -> bool {
                true
            }
        }

        let mut integration = RegistryIntegration::new();
        let handler = Box::new(TestTagHandler);

        assert!(integration
            .register_custom_tag_handler("test-extension".to_string(), handler)
            .is_ok());
    }

    #[test]
    fn test_custom_section_processor_registration() {
        use ass_core::plugin::{SectionProcessor, SectionResult};

        struct TestSectionProcessor;
        impl SectionProcessor for TestSectionProcessor {
            fn name(&self) -> &'static str {
                "TestSection"
            }

            fn process(&self, _header: &str, _lines: &[&str]) -> SectionResult {
                SectionResult::Processed
            }

            fn validate(&self, _header: &str, _lines: &[&str]) -> bool {
                true
            }
        }

        let mut integration = RegistryIntegration::new();
        let processor = Box::new(TestSectionProcessor);

        assert!(integration
            .register_custom_section_processor("test-extension".to_string(), processor)
            .is_ok());
    }

    #[test]
    fn test_registry_access() {
        let integration = RegistryIntegration::new();

        // Should be able to access the registry
        let _registry = integration.registry();

        // Mutable access
        let mut integration = RegistryIntegration::new();
        let _registry_mut = integration.registry_mut();
    }

    #[test]
    fn test_full_integration() {
        let mut integration = RegistryIntegration::new();

        // Register all built-ins
        assert!(integration.register_builtin_handlers().is_ok());
        assert!(integration.register_builtin_sections().is_ok());

        // The registry should now have many handlers registered
        // We can't easily test the exact count, but we can verify it worked
        let registry = integration.registry();

        // Use the registry to parse some ASS content with tags
        let test_content = "[Script Info]\nTitle: Test\n\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,{\\b1}Bold{\\b0} text";

        // Parse with the registry
        let result = ass_core::parser::Script::builder()
            .with_registry(registry)
            .parse(test_content);

        assert!(result.is_ok());
    }
}
