//! Style editing command for ASS documents.
//!
//! Provides [`EditStyleCommand`] to update individual fields of an existing
//! style, mapping field names to columns via the section format line.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

/// Command to edit an existing style
#[derive(Debug, Clone)]
pub struct EditStyleCommand {
    pub style_name: String,
    pub field_updates: HashMap<String, String>,
    pub description: Option<String>,
}

impl EditStyleCommand {
    /// Create a new style edit command
    pub fn new(style_name: String) -> Self {
        Self {
            style_name,
            field_updates: HashMap::new(),
            description: None,
        }
    }

    /// Set a field value
    pub fn set_field(mut self, field: &str, value: String) -> Self {
        self.field_updates.insert(field.to_string(), value);
        self
    }

    /// Set font name
    pub fn set_font(self, font: &str) -> Self {
        self.set_field("Fontname", font.to_string())
    }

    /// Set font size
    pub fn set_size(self, size: u32) -> Self {
        self.set_field("Fontsize", size.to_string())
    }

    /// Set primary color
    pub fn set_color(self, color: &str) -> Self {
        self.set_field("PrimaryColour", color.to_string())
    }

    /// Set bold
    pub fn set_bold(self, bold: bool) -> Self {
        self.set_field("Bold", if bold { "-1" } else { "0" }.to_string())
    }

    /// Set italic
    pub fn set_italic(self, italic: bool) -> Self {
        self.set_field("Italic", if italic { "-1" } else { "0" }.to_string())
    }

    /// Set alignment
    pub fn set_alignment(self, alignment: u32) -> Self {
        self.set_field("Alignment", alignment.to_string())
    }

    /// Set custom description
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for EditStyleCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text();
        let style_pattern = format!("Style: {}", self.style_name);

        if let Some(style_start) = content.find(&style_pattern) {
            // Find the end of the style line
            let line_start = content[..style_start]
                .rfind('\n')
                .map(|pos| pos + 1)
                .unwrap_or(0);
            let line_end = content[style_start..]
                .find('\n')
                .map(|pos| style_start + pos)
                .unwrap_or(content.len());

            let style_line = &content[line_start..line_end];
            let fields: Vec<&str> = style_line.split(',').collect();

            if fields.len() < 2 {
                return Err(EditorError::command_failed("Invalid style format"));
            }

            // Find format line to determine field order
            let styles_section_start = content[..line_start]
                .rfind("[V4+ Styles]")
                .or_else(|| content[..line_start].rfind("[V4 Styles]"))
                .or_else(|| content[..line_start].rfind("[Styles]"))
                .ok_or_else(|| EditorError::command_failed("Could not find styles section"))?;

            let format_line_start = content[styles_section_start..]
                .find("Format:")
                .map(|pos| styles_section_start + pos)
                .ok_or_else(|| EditorError::command_failed("Could not find format line"))?;

            let format_line_end = content[format_line_start..]
                .find('\n')
                .map(|pos| format_line_start + pos)
                .unwrap_or(content.len());

            let format_line = &content[format_line_start..format_line_end];
            let format_fields: Vec<&str> = format_line
                .strip_prefix("Format: ")
                .unwrap_or(format_line)
                .split(", ")
                .collect();

            // Build updated style line
            let mut updated_fields = fields
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<String>>();

            for (field_name, new_value) in &self.field_updates {
                if let Some(field_index) = format_fields.iter().position(|f| f == field_name) {
                    if field_index < updated_fields.len() {
                        updated_fields[field_index] = new_value.clone();
                    }
                }
            }

            let new_style_line = updated_fields.join(",");
            let range = Range::new(Position::new(line_start), Position::new(line_end));

            document.replace(range, &new_style_line)?;

            let end_pos = Position::new(line_start + new_style_line.len());
            Ok(CommandResult::success_with_change(
                Range::new(Position::new(line_start), end_pos),
                end_pos,
            )
            .with_message(format!("Updated style '{}'", self.style_name)))
        } else {
            Err(EditorError::command_failed(format!(
                "Style '{}' not found",
                self.style_name
            )))
        }
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Edit style")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.style_name.len()
            + self
                .field_updates
                .iter()
                .map(|(k, v)| k.len() + v.len())
                .sum::<usize>()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}
