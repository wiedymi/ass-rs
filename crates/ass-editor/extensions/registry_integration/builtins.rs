//! Built-in tag handler and section processor registration for
//! [`RegistryIntegration`].
//!
//! Wires the handler factories and section processors shipped by ass-core into
//! the wrapped `ExtensionRegistry`.

use super::RegistryIntegration;
use crate::core::{EditorError, Result};

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, format, string::ToString};

impl RegistryIntegration {
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
}
