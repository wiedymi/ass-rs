//! Fluent API for managing Script Info properties.

use crate::commands::{
    DeleteScriptInfoCommand, EditorCommand, GetAllScriptInfoCommand, GetScriptInfoCommand,
    SetScriptInfoCommand,
};
use crate::core::errors::EditorError;
use crate::core::{EditorDocument, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// Fluent API for managing Script Info properties
pub struct ScriptInfoOps<'a> {
    document: &'a mut EditorDocument,
}

impl<'a> ScriptInfoOps<'a> {
    /// Create new script info operations
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self { document }
    }

    /// Set a script info property
    pub fn set(self, property: &str, value: &str) -> Result<&'a mut EditorDocument> {
        let command = SetScriptInfoCommand::new(property.to_string(), value.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Get a script info property value
    pub fn get(&self, property: &str) -> Result<Option<String>> {
        let command = GetScriptInfoCommand::new(property.to_string());
        command.get_value(self.document)
    }

    /// Delete a script info property
    pub fn delete(self, property: &str) -> Result<&'a mut EditorDocument> {
        let command = DeleteScriptInfoCommand::new(property.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Get all script info properties
    pub fn all(&self) -> Result<Vec<(String, String)>> {
        let command = GetAllScriptInfoCommand::new();
        command.get_all(self.document)
    }

    /// Set the title
    pub fn title(self, title: &str) -> Result<&'a mut EditorDocument> {
        self.set("Title", title)
    }

    /// Get the title
    pub fn get_title(&self) -> Result<Option<String>> {
        self.get("Title")
    }

    /// Set the author
    pub fn author(self, author: &str) -> Result<&'a mut EditorDocument> {
        self.set("Original Script", author)
    }

    /// Get the author
    pub fn get_author(&self) -> Result<Option<String>> {
        self.get("Original Script")
    }

    /// Set the resolution
    pub fn resolution(self, width: u32, height: u32) -> Result<&'a mut EditorDocument> {
        let command1 = SetScriptInfoCommand::new("PlayResX".to_string(), width.to_string());
        let command2 = SetScriptInfoCommand::new("PlayResY".to_string(), height.to_string());
        command1.execute(self.document)?;
        command2.execute(self.document)?;
        Ok(self.document)
    }

    /// Get the resolution
    pub fn get_resolution(&self) -> Result<Option<(u32, u32)>> {
        let width_cmd = GetScriptInfoCommand::new("PlayResX".to_string());
        let height_cmd = GetScriptInfoCommand::new("PlayResY".to_string());

        let width = width_cmd.get_value(self.document)?;
        let height = height_cmd.get_value(self.document)?;

        match (width, height) {
            (Some(w), Some(h)) => {
                let width_val = w
                    .parse::<u32>()
                    .map_err(|_| EditorError::command_failed("Invalid PlayResX value"))?;
                let height_val = h
                    .parse::<u32>()
                    .map_err(|_| EditorError::command_failed("Invalid PlayResY value"))?;
                Ok(Some((width_val, height_val)))
            }
            _ => Ok(None),
        }
    }

    /// Set the wrap style
    pub fn wrap_style(self, style: u8) -> Result<&'a mut EditorDocument> {
        self.set("WrapStyle", &style.to_string())
    }

    /// Get the wrap style
    pub fn get_wrap_style(&self) -> Result<Option<u8>> {
        self.get("WrapStyle")?
            .map(|s| {
                s.parse::<u8>()
                    .map_err(|_| EditorError::command_failed("Invalid WrapStyle value"))
            })
            .transpose()
    }

    /// Set scaled border and shadow
    pub fn scaled_border_and_shadow(self, scaled: bool) -> Result<&'a mut EditorDocument> {
        let value = if scaled { "yes" } else { "no" };
        self.set("ScaledBorderAndShadow", value)
    }

    /// Get scaled border and shadow setting
    pub fn get_scaled_border_and_shadow(&self) -> Result<Option<bool>> {
        Ok(self
            .get("ScaledBorderAndShadow")?
            .map(|s| s.to_lowercase() == "yes" || s == "1"))
    }
}
