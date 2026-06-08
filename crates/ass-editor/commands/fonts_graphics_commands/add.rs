//! Commands for adding embedded fonts and graphics to ASS documents

use super::uuencode_data;
use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

/// Command to add an embedded font to the ASS document
///
/// Fonts are embedded using UU-encoding in the `[Fonts]` section.
/// This command supports both pre-encoded data and raw binary data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddFontCommand {
    /// Font filename
    pub filename: String,
    /// UU-encoded font data lines
    pub data_lines: Vec<String>,
}

impl AddFontCommand {
    /// Create a new add font command
    pub fn new(filename: String, data_lines: Vec<String>) -> Self {
        Self {
            filename,
            data_lines,
        }
    }

    /// Create from raw binary data (will UU-encode it)
    pub fn from_binary(filename: String, data: &[u8]) -> Self {
        let encoded = uuencode_data(&filename, data);
        Self {
            filename,
            data_lines: encoded,
        }
    }
}

impl EditorCommand for AddFontCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text().to_string();

        // Find or create [Fonts] section
        let fonts_section = content.find("[Fonts]");

        let insert_pos = if let Some(section_start) = fonts_section {
            // Find the end of the Fonts section
            content[section_start..]
                .find("\n[")
                .map(|pos| section_start + pos)
                .unwrap_or(content.len())
        } else {
            // Create new Fonts section at the end
            let mut insert_pos = content.len();

            // Add newline if needed
            if !content.ends_with('\n') {
                document.insert_raw(Position::new(insert_pos), "\n")?;
                insert_pos += 1;
            }

            // Add section header
            document.insert_raw(Position::new(insert_pos), "[Fonts]\n")?;
            insert_pos + "[Fonts]\n".len()
        };

        // Build font entry
        let filename = &self.filename;
        let mut font_entry = format!("fontname: {filename}\n");
        for line in &self.data_lines {
            font_entry.push_str(line);
            font_entry.push('\n');
        }

        // Insert font data
        document.insert_raw(Position::new(insert_pos), &font_entry)?;

        Ok(CommandResult::success_with_change(
            Range::new(
                Position::new(insert_pos),
                Position::new(insert_pos + font_entry.len()),
            ),
            Position::new(insert_pos + font_entry.len()),
        ))
    }

    fn description(&self) -> &str {
        "Add font"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.filename.len()
            + self.data_lines.iter().map(|l| l.len()).sum::<usize>()
    }
}

/// Command to add an embedded graphic to the ASS document
///
/// Graphics are embedded using UU-encoding in the `[Graphics]` section.
/// This command supports both pre-encoded data and raw binary data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddGraphicCommand {
    /// Graphic filename
    pub filename: String,
    /// UU-encoded graphic data lines
    pub data_lines: Vec<String>,
}

impl AddGraphicCommand {
    /// Create a new add graphic command
    pub fn new(filename: String, data_lines: Vec<String>) -> Self {
        Self {
            filename,
            data_lines,
        }
    }

    /// Create from raw binary data (will UU-encode it)
    pub fn from_binary(filename: String, data: &[u8]) -> Self {
        let encoded = uuencode_data(&filename, data);
        Self {
            filename,
            data_lines: encoded,
        }
    }
}

impl EditorCommand for AddGraphicCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text().to_string();

        // Find or create [Graphics] section
        let graphics_section = content.find("[Graphics]");

        let insert_pos = if let Some(section_start) = graphics_section {
            // Find the end of the Graphics section
            content[section_start..]
                .find("\n[")
                .map(|pos| section_start + pos)
                .unwrap_or(content.len())
        } else {
            // Create new Graphics section at the end
            let mut insert_pos = content.len();

            // Add newline if needed
            if !content.ends_with('\n') {
                document.insert_raw(Position::new(insert_pos), "\n")?;
                insert_pos += 1;
            }

            // Add section header
            document.insert_raw(Position::new(insert_pos), "[Graphics]\n")?;
            insert_pos + "[Graphics]\n".len()
        };

        // Build graphic entry
        let filename = &self.filename;
        let mut graphic_entry = format!("filename: {filename}\n");
        for line in &self.data_lines {
            graphic_entry.push_str(line);
            graphic_entry.push('\n');
        }

        // Insert graphic data
        document.insert_raw(Position::new(insert_pos), &graphic_entry)?;

        Ok(CommandResult::success_with_change(
            Range::new(
                Position::new(insert_pos),
                Position::new(insert_pos + graphic_entry.len()),
            ),
            Position::new(insert_pos + graphic_entry.len()),
        ))
    }

    fn description(&self) -> &str {
        "Add graphic"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.filename.len()
            + self.data_lines.iter().map(|l| l.len()).sum::<usize>()
    }
}
