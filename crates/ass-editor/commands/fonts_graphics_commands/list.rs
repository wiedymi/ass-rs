//! Commands for listing embedded fonts and graphics in ASS documents

use crate::core::{EditorDocument, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

/// Command to list all embedded fonts in the ASS document
///
/// Returns a list of font filenames from the `[Fonts]` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListFontsCommand;

impl ListFontsCommand {
    /// Create a new list fonts command
    pub fn new() -> Self {
        Self
    }

    /// Execute and return list of font filenames
    pub fn list(&self, document: &EditorDocument) -> Result<Vec<String>> {
        let content = document.text();

        // Find [Fonts] section
        let fonts_section = content.find("[Fonts]");
        if fonts_section.is_none() {
            return Ok(Vec::new());
        }

        let section_start = fonts_section.unwrap();
        let section_end = content[section_start..]
            .find("\n[")
            .map(|pos| section_start + pos)
            .unwrap_or(content.len());

        let section_content = &content[section_start..section_end];
        let mut fonts = Vec::new();

        for line in section_content.lines() {
            if let Some(filename) = line.strip_prefix("fontname: ") {
                fonts.push(filename.trim().to_string());
            }
        }

        Ok(fonts)
    }
}

impl Default for ListFontsCommand {
    fn default() -> Self {
        Self::new()
    }
}

/// Command to list all embedded graphics in the ASS document
///
/// Returns a list of graphic filenames from the `[Graphics]` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListGraphicsCommand;

impl ListGraphicsCommand {
    /// Create a new list graphics command
    pub fn new() -> Self {
        Self
    }

    /// Execute and return list of graphic filenames
    pub fn list(&self, document: &EditorDocument) -> Result<Vec<String>> {
        let content = document.text();

        // Find [Graphics] section
        let graphics_section = content.find("[Graphics]");
        if graphics_section.is_none() {
            return Ok(Vec::new());
        }

        let section_start = graphics_section.unwrap();
        let section_end = content[section_start..]
            .find("\n[")
            .map(|pos| section_start + pos)
            .unwrap_or(content.len());

        let section_content = &content[section_start..section_end];
        let mut graphics = Vec::new();

        for line in section_content.lines() {
            if let Some(filename) = line.strip_prefix("filename: ") {
                graphics.push(filename.trim().to_string());
            }
        }

        Ok(graphics)
    }
}

impl Default for ListGraphicsCommand {
    fn default() -> Self {
        Self::new()
    }
}
