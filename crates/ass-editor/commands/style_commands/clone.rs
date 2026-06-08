//! Style cloning command for ASS documents.
//!
//! Provides [`CloneStyleCommand`] to duplicate an existing style under a new
//! name, inserting the clone after the source style line.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

/// Command to clone an existing style with a new name
#[derive(Debug, Clone)]
pub struct CloneStyleCommand {
    pub source_style: String,
    pub target_style: String,
    pub description: Option<String>,
}

impl CloneStyleCommand {
    /// Create a new style cloning command
    pub fn new(source_style: String, target_style: String) -> Self {
        Self {
            source_style,
            target_style,
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

impl EditorCommand for CloneStyleCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text();
        let source_pattern = format!("Style: {}", self.source_style);

        // Check if target style already exists
        let target_pattern = format!("Style: {}", self.target_style);
        if content.contains(&target_pattern) {
            return Err(EditorError::command_failed(format!(
                "Style '{}' already exists",
                self.target_style
            )));
        }

        if let Some(source_start) = content.find(&source_pattern) {
            // Find the complete source style line
            let line_start = content[..source_start]
                .rfind('\n')
                .map(|pos| pos + 1)
                .unwrap_or(0);
            let line_end = content[source_start..]
                .find('\n')
                .map(|pos| source_start + pos)
                .unwrap_or(content.len());

            let source_line = &content[line_start..line_end];

            // Replace the style name in the cloned line
            let cloned_line = source_line.replace(
                &format!("Style: {}", self.source_style),
                &format!("Style: {}", self.target_style),
            );

            // Find where to insert the new style (after the source style)
            let insert_pos = line_end;
            let insert_text = format!("\n{cloned_line}");

            document.insert(Position::new(insert_pos), &insert_text)?;

            let end_pos = Position::new(insert_pos + insert_text.len());
            Ok(CommandResult::success_with_change(
                Range::new(Position::new(insert_pos), end_pos),
                end_pos,
            )
            .with_message(format!(
                "Cloned style '{}' to '{}'",
                self.source_style, self.target_style
            )))
        } else {
            Err(EditorError::command_failed(format!(
                "Source style '{}' not found",
                self.source_style
            )))
        }
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Clone style")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.source_style.len()
            + self.target_style.len()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}
