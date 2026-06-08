//! Command to set or update a property in the `[Script Info]` section.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

/// Command to set a script info property in the ASS document
///
/// Sets or updates properties like Title, Author, PlayResX, PlayResY, etc.
/// in the `[Script Info]` section. Creates the section if it doesn't exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetScriptInfoCommand {
    /// Property name (e.g., "Title", "Author", "PlayResX")
    pub property: String,
    /// New value for the property
    pub value: String,
}

impl SetScriptInfoCommand {
    /// Create a new set script info command
    pub fn new(property: String, value: String) -> Self {
        Self { property, value }
    }
}

impl EditorCommand for SetScriptInfoCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text().to_string();

        // Find [Script Info] section
        let script_info_start = content
            .find("[Script Info]")
            .ok_or_else(|| EditorError::command_failed("No [Script Info] section found"))?;

        // Find the end of Script Info section (next section or end of file)
        let script_info_end = content[script_info_start..]
            .find("\n[")
            .map(|pos| script_info_start + pos)
            .unwrap_or(content.len());

        // Check if property already exists
        let property_pattern = format!("{}: ", self.property);
        let search_range = &content[script_info_start..script_info_end];

        if let Some(prop_pos) = search_range.find(&property_pattern) {
            // Property exists, update it
            let absolute_pos = script_info_start + prop_pos;
            let line_start = absolute_pos;
            let line_end = content[absolute_pos..]
                .find('\n')
                .map(|pos| absolute_pos + pos)
                .unwrap_or(content.len());

            let new_line = format!("{}: {}", self.property, self.value);
            let range = Range::new(Position::new(line_start), Position::new(line_end));

            document.replace(range, &new_line)?;

            Ok(CommandResult::success_with_change(
                Range::new(
                    Position::new(line_start),
                    Position::new(line_start + new_line.len()),
                ),
                Position::new(line_start + new_line.len()),
            ))
        } else {
            // Property doesn't exist, add it after [Script Info]
            let insert_pos = content[script_info_start..]
                .find('\n')
                .map(|pos| script_info_start + pos + 1)
                .unwrap_or(script_info_start + 13); // Length of "[Script Info]"

            let new_line = format!("{}: {}\n", self.property, self.value);
            document.insert_raw(Position::new(insert_pos), &new_line)?;

            Ok(CommandResult::success_with_change(
                Range::new(
                    Position::new(insert_pos),
                    Position::new(insert_pos + new_line.len()),
                ),
                Position::new(insert_pos + new_line.len()),
            ))
        }
    }

    fn description(&self) -> &str {
        "Set script info property"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>() + self.property.len() + self.value.len()
    }
}
