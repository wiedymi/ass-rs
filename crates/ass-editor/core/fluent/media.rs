//! Fluent API for managing embedded fonts and graphics.

use crate::commands::{
    AddFontCommand, AddGraphicCommand, ClearFontsCommand, ClearGraphicsCommand, EditorCommand,
    ListFontsCommand, ListGraphicsCommand, RemoveFontCommand, RemoveGraphicCommand,
};
use crate::core::{EditorDocument, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// Fluent API for managing fonts
pub struct FontsOps<'a> {
    document: &'a mut EditorDocument,
}

impl<'a> FontsOps<'a> {
    /// Create new fonts operations
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self { document }
    }

    /// Add a font from UU-encoded data
    pub fn add(self, filename: &str, data_lines: Vec<String>) -> Result<&'a mut EditorDocument> {
        let command = AddFontCommand::new(filename.to_string(), data_lines);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Add a font from binary data (will UU-encode it)
    pub fn add_binary(self, filename: &str, data: &[u8]) -> Result<&'a mut EditorDocument> {
        let command = AddFontCommand::from_binary(filename.to_string(), data);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Remove a font by filename
    pub fn remove(self, filename: &str) -> Result<&'a mut EditorDocument> {
        let command = RemoveFontCommand::new(filename.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// List all font filenames
    pub fn list(&self) -> Result<Vec<String>> {
        let command = ListFontsCommand::new();
        command.list(self.document)
    }

    /// Check if a font exists
    pub fn exists(&self, filename: &str) -> Result<bool> {
        Ok(self.list()?.contains(&filename.to_string()))
    }

    /// Clear all fonts
    pub fn clear(self) -> Result<&'a mut EditorDocument> {
        let command = ClearFontsCommand::new();
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Get count of fonts
    pub fn count(&self) -> Result<usize> {
        Ok(self.list()?.len())
    }
}

/// Fluent API for managing graphics
pub struct GraphicsOps<'a> {
    document: &'a mut EditorDocument,
}

impl<'a> GraphicsOps<'a> {
    /// Create new graphics operations
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self { document }
    }

    /// Add a graphic from UU-encoded data
    pub fn add(self, filename: &str, data_lines: Vec<String>) -> Result<&'a mut EditorDocument> {
        let command = AddGraphicCommand::new(filename.to_string(), data_lines);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Add a graphic from binary data (will UU-encode it)
    pub fn add_binary(self, filename: &str, data: &[u8]) -> Result<&'a mut EditorDocument> {
        let command = AddGraphicCommand::from_binary(filename.to_string(), data);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Remove a graphic by filename
    pub fn remove(self, filename: &str) -> Result<&'a mut EditorDocument> {
        let command = RemoveGraphicCommand::new(filename.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// List all graphic filenames
    pub fn list(&self) -> Result<Vec<String>> {
        let command = ListGraphicsCommand::new();
        command.list(self.document)
    }

    /// Check if a graphic exists
    pub fn exists(&self, filename: &str) -> Result<bool> {
        Ok(self.list()?.contains(&filename.to_string()))
    }

    /// Clear all graphics
    pub fn clear(self) -> Result<&'a mut EditorDocument> {
        let command = ClearGraphicsCommand::new();
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Get count of graphics
    pub fn count(&self) -> Result<usize> {
        Ok(self.list()?.len())
    }
}
