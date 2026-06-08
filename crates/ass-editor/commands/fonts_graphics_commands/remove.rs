//! Commands for removing embedded fonts and graphics from ASS documents

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

/// Command to remove an embedded font from the ASS document
///
/// Removes a font by filename from the `[Fonts]` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveFontCommand {
    /// Font filename to remove
    pub filename: String,
}

impl RemoveFontCommand {
    /// Create a new remove font command
    pub fn new(filename: String) -> Self {
        Self { filename }
    }
}

impl EditorCommand for RemoveFontCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text().to_string();

        // Find [Fonts] section
        let fonts_section = content
            .find("[Fonts]")
            .ok_or_else(|| EditorError::command_failed("No [Fonts] section found"))?;

        // Find the end of the Fonts section
        let section_end = content[fonts_section..]
            .find("\n[")
            .map(|pos| fonts_section + pos)
            .unwrap_or(content.len());

        // Look for the font entry
        let filename = &self.filename;
        let search_pattern = format!("fontname: {filename}");
        let section_content = &content[fonts_section..section_end];

        if let Some(font_pos) = section_content.find(&search_pattern) {
            let absolute_pos = fonts_section + font_pos;

            // Find the end of this font's data (next fontname: or section end)
            let font_end = content[absolute_pos..]
                .find("\nfontname:")
                .map(|pos| absolute_pos + pos)
                .unwrap_or(section_end);

            let range = Range::new(Position::new(absolute_pos), Position::new(font_end));
            document.delete(range)?;

            Ok(CommandResult::success_with_change(
                Range::new(Position::new(absolute_pos), Position::new(absolute_pos)),
                Position::new(absolute_pos),
            ))
        } else {
            Ok(CommandResult::success())
        }
    }

    fn description(&self) -> &str {
        "Remove font"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>() + self.filename.len()
    }
}

/// Command to remove an embedded graphic from the ASS document
///
/// Removes a graphic by filename from the `[Graphics]` section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveGraphicCommand {
    /// Graphic filename to remove
    pub filename: String,
}

impl RemoveGraphicCommand {
    /// Create a new remove graphic command
    pub fn new(filename: String) -> Self {
        Self { filename }
    }
}

impl EditorCommand for RemoveGraphicCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text().to_string();

        // Find [Graphics] section
        let graphics_section = content
            .find("[Graphics]")
            .ok_or_else(|| EditorError::command_failed("No [Graphics] section found"))?;

        // Find the end of the Graphics section
        let section_end = content[graphics_section..]
            .find("\n[")
            .map(|pos| graphics_section + pos)
            .unwrap_or(content.len());

        // Look for the graphic entry
        let filename = &self.filename;
        let search_pattern = format!("filename: {filename}");
        let section_content = &content[graphics_section..section_end];

        if let Some(graphic_pos) = section_content.find(&search_pattern) {
            let absolute_pos = graphics_section + graphic_pos;

            // Find the end of this graphic's data (next filename: or section end)
            let graphic_end = content[absolute_pos..]
                .find("\nfilename:")
                .map(|pos| absolute_pos + pos)
                .unwrap_or(section_end);

            let range = Range::new(Position::new(absolute_pos), Position::new(graphic_end));
            document.delete(range)?;

            Ok(CommandResult::success_with_change(
                Range::new(Position::new(absolute_pos), Position::new(absolute_pos)),
                Position::new(absolute_pos),
            ))
        } else {
            Ok(CommandResult::success())
        }
    }

    fn description(&self) -> &str {
        "Remove graphic"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>() + self.filename.len()
    }
}
