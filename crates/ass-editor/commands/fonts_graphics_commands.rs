//! Fonts and Graphics management commands for ASS documents
//!
//! Provides commands for managing embedded fonts and graphics in the
//! `[Fonts]` and `[Graphics]` sections using UU-encoding.

use super::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

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

/// Helper function to UU-encode binary data
fn uuencode_data(filename: &str, data: &[u8]) -> Vec<String> {
    let mut lines = Vec::new();

    // Add UU-encode header
    lines.push(format!("begin 644 {filename}"));

    // Process data in 45-byte chunks (standard UU-encoding)
    for chunk in data.chunks(45) {
        let mut encoded = String::new();

        // Length character
        encoded.push((32 + chunk.len()) as u8 as char);

        // Encode groups of 3 bytes into 4 characters
        for group in chunk.chunks(3) {
            let mut bytes = [0u8; 3];
            for (i, &byte) in group.iter().enumerate() {
                bytes[i] = byte;
            }

            // Convert 3 bytes to 4 UU-encoded characters
            encoded.push((32 + (bytes[0] >> 2)) as char);
            encoded.push((32 + (((bytes[0] & 0x03) << 4) | (bytes[1] >> 4))) as char);

            if group.len() > 1 {
                encoded.push((32 + (((bytes[1] & 0x0F) << 2) | (bytes[2] >> 6))) as char);
            } else {
                encoded.push((32 + ((bytes[1] & 0x0F) << 2)) as char);
            }

            if group.len() > 2 {
                encoded.push((32 + (bytes[2] & 0x3F)) as char);
            } else if group.len() > 1 {
                encoded.push(' ');
            }
        }

        lines.push(encoded);
    }

    // Add UU-encode footer
    lines.push("`".to_string());
    lines.push("end".to_string());

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EditorDocument;

    #[cfg(not(feature = "std"))]
    use alloc::vec;

    const TEST_CONTENT: &str = r#"[Script Info]
Title: Test

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#;

    #[test]
    fn test_add_font() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = AddFontCommand::new(
            "custom.ttf".to_string(),
            vec![
                "begin 644 custom.ttf".to_string(),
                "M1234...".to_string(),
                "end".to_string(),
            ],
        );
        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);
        assert!(doc.text().contains("[Fonts]"));
        assert!(doc.text().contains("fontname: custom.ttf"));
    }

    #[test]
    fn test_remove_font() {
        let mut doc = EditorDocument::from_content(
            "[Script Info]\n[Fonts]\nfontname: test.ttf\nbegin 644 test.ttf\nM123\nend\n",
        )
        .unwrap();

        let command = RemoveFontCommand::new("test.ttf".to_string());
        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);
        assert!(!doc.text().contains("fontname: test.ttf"));
    }

    #[test]
    fn test_list_fonts() {
        let doc = EditorDocument::from_content(
            "[Fonts]\nfontname: font1.ttf\ndata\nfontname: font2.otf\ndata\n",
        )
        .unwrap();

        let command = ListFontsCommand::new();
        let fonts = command.list(&doc).unwrap();

        assert_eq!(fonts.len(), 2);
        assert_eq!(fonts[0], "font1.ttf");
        assert_eq!(fonts[1], "font2.otf");
    }

    #[test]
    fn test_uuencode() {
        let data = b"Hello World!";
        let encoded = uuencode_data("test.txt", data);

        assert_eq!(encoded[0], "begin 644 test.txt");
        assert_eq!(encoded[encoded.len() - 1], "end");
        assert_eq!(encoded[encoded.len() - 2], "`");
    }
}
