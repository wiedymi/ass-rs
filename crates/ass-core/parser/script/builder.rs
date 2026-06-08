//! Configurable builder for [`Script`] parsing.
//!
//! Provides [`ScriptBuilder`], a fluent entry point that optionally wires an
//! extension registry of custom tag handlers and section processors into the
//! parser before producing a [`Script`].

use crate::parser::main::Parser;
use crate::Result;

#[cfg(feature = "plugins")]
use crate::plugin::ExtensionRegistry;

use super::Script;

/// Builder for configuring script parsing with optional extensions
///
/// Provides a fluent API for setting up parsing configuration including
/// extension registry for custom tag handlers and section processors.
#[derive(Debug)]
pub struct ScriptBuilder<'a> {
    /// Extension registry for custom handlers
    #[cfg(feature = "plugins")]
    registry: Option<&'a ExtensionRegistry>,
}

impl<'a> ScriptBuilder<'a> {
    /// Create a new script builder
    #[must_use]
    pub const fn new() -> Self {
        Self {
            #[cfg(feature = "plugins")]
            registry: None,
        }
    }

    /// Set the extension registry for custom tag handlers and section processors
    ///
    /// # Arguments
    /// * `registry` - Registry containing custom extensions
    #[cfg(feature = "plugins")]
    #[must_use]
    pub const fn with_registry(mut self, registry: &'a ExtensionRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Parse ASS script with configured options
    ///
    /// # Arguments
    /// * `source` - Source text to parse
    ///
    /// # Returns
    /// Parsed script with zero-copy design
    ///
    /// # Errors
    /// Returns an error if parsing fails due to malformed syntax
    pub fn parse(self, source: &'a str) -> Result<Script<'a>> {
        #[cfg(feature = "plugins")]
        let parser = Parser::new_with_registry(source, self.registry);
        #[cfg(not(feature = "plugins"))]
        let parser = Parser::new(source);

        Ok(parser.parse())
    }
}

impl Default for ScriptBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}
