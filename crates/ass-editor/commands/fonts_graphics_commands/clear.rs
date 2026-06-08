//! Commands for clearing embedded fonts and graphics sections in ASS documents

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

/// Command to clear all embedded fonts from the ASS document
///
/// Removes the entire `[Fonts]` section if it exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearFontsCommand;

impl ClearFontsCommand {
    /// Create a new clear fonts command
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClearFontsCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorCommand for ClearFontsCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text().to_string();

        // Find [Fonts] section
        if let Some(fonts_section) = content.find("[Fonts]") {
            let header_end = content[fonts_section..]
                .find('\n')
                .map(|pos| fonts_section + pos + 1)
                .unwrap_or(fonts_section + "[Fonts]".len());

            let section_end = content[fonts_section..]
                .find("\n[")
                .map(|pos| fonts_section + pos)
                .unwrap_or(content.len());

            if section_end > header_end {
                let range = Range::new(Position::new(header_end), Position::new(section_end));
                document.delete(range)?;

                return Ok(CommandResult::success_with_change(
                    Range::new(Position::new(header_end), Position::new(header_end)),
                    Position::new(header_end),
                ));
            }
        }

        Ok(CommandResult::success())
    }

    fn description(&self) -> &str {
        "Clear all fonts"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
    }
}

/// Command to clear all embedded graphics from the ASS document
///
/// Removes the entire `[Graphics]` section if it exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearGraphicsCommand;

impl ClearGraphicsCommand {
    /// Create a new clear graphics command
    pub fn new() -> Self {
        Self
    }
}

impl Default for ClearGraphicsCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorCommand for ClearGraphicsCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text().to_string();

        // Find [Graphics] section
        if let Some(graphics_section) = content.find("[Graphics]") {
            let header_end = content[graphics_section..]
                .find('\n')
                .map(|pos| graphics_section + pos + 1)
                .unwrap_or(graphics_section + "[Graphics]".len());

            let section_end = content[graphics_section..]
                .find("\n[")
                .map(|pos| graphics_section + pos)
                .unwrap_or(content.len());

            if section_end > header_end {
                let range = Range::new(Position::new(header_end), Position::new(section_end));
                document.delete(range)?;

                return Ok(CommandResult::success_with_change(
                    Range::new(Position::new(header_end), Position::new(header_end)),
                    Position::new(header_end),
                ));
            }
        }

        Ok(CommandResult::success())
    }

    fn description(&self) -> &str {
        "Clear all graphics"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
    }
}
