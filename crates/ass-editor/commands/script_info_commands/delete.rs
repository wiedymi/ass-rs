//! Command to remove a property from the `[Script Info]` section.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

/// Command to delete a script info property from the ASS document
///
/// Removes a specific property from the `[Script Info]` section.
/// Does not remove the section itself even if it becomes empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteScriptInfoCommand {
    /// Property name to delete
    pub property: String,
}

impl DeleteScriptInfoCommand {
    /// Create a new delete script info command
    pub fn new(property: String) -> Self {
        Self { property }
    }
}

impl EditorCommand for DeleteScriptInfoCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text().to_string();

        // Find [Script Info] section
        let script_info_start = content
            .find("[Script Info]")
            .ok_or_else(|| EditorError::command_failed("No [Script Info] section found"))?;

        // Find the end of Script Info section
        let script_info_end = content[script_info_start..]
            .find("\n[")
            .map(|pos| script_info_start + pos)
            .unwrap_or(content.len());

        // Look for the property
        let property_pattern = format!("{}: ", self.property);
        let search_range = &content[script_info_start..script_info_end];

        if let Some(prop_pos) = search_range.find(&property_pattern) {
            let absolute_pos = script_info_start + prop_pos;
            let line_end = content[absolute_pos..]
                .find('\n')
                .map(|pos| absolute_pos + pos + 1) // Include newline
                .unwrap_or(content.len());

            let range = Range::new(Position::new(absolute_pos), Position::new(line_end));
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
        "Delete script info property"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>() + self.property.len()
    }
}
