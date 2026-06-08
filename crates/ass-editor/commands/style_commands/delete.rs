//! Style deletion command for ASS documents.
//!
//! Provides [`DeleteStyleCommand`] to remove a named style line from the
//! styles section.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

/// Command to delete a style
#[derive(Debug, Clone)]
pub struct DeleteStyleCommand {
    pub style_name: String,
    pub description: Option<String>,
}

impl DeleteStyleCommand {
    /// Create a new style deletion command
    pub fn new(style_name: String) -> Self {
        Self {
            style_name,
            description: None,
        }
    }

    /// Set custom description
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for DeleteStyleCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text();
        let style_pattern = format!("Style: {}", self.style_name);

        if let Some(style_start) = content.find(&style_pattern) {
            // Find the complete line including the newline
            let line_start = content[..style_start]
                .rfind('\n')
                .map(|pos| pos + 1)
                .unwrap_or(0);
            let line_end = content[style_start..]
                .find('\n')
                .map(|pos| style_start + pos + 1) // Include the newline
                .unwrap_or(content.len());

            let range = Range::new(Position::new(line_start), Position::new(line_end));
            document.delete(range)?;

            Ok(CommandResult::success_with_change(
                Range::new(Position::new(line_start), Position::new(line_start)),
                Position::new(line_start),
            )
            .with_message(format!("Deleted style '{}'", self.style_name)))
        } else {
            Err(EditorError::command_failed(format!(
                "Style '{}' not found",
                self.style_name
            )))
        }
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Delete style")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.style_name.len()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}
