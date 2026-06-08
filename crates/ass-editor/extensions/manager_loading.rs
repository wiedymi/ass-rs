//! Extension loading, initialization, and teardown for the manager.
//!
//! Covers registering extensions, running their initialization and shutdown
//! lifecycle hooks, and unloading them along with their registered commands.

use crate::core::Result;

use super::command::ExtensionState;
use super::context::ExtensionContext;
use super::extension::EditorExtension;
use super::manager::ExtensionManager;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, format, string::ToString};

impl ExtensionManager {
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
}
