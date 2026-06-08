//! Insert command for ASS override tags
//!
//! Provides [`InsertTagCommand`] for inserting an override tag at a position,
//! with optional auto-wrapping in override brackets `{}`.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

/// Insert ASS override tag at specified position
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertTagCommand {
    /// Position to insert tag at
    pub position: Position,
    /// Tag to insert (e.g., "\\b1", "\\c&H00FF00&")
    pub tag: String,
    /// Whether to wrap in override brackets {} if not present
    pub auto_wrap: bool,
}

impl InsertTagCommand {
    /// Create a new insert tag command
    pub fn new(position: Position, tag: String) -> Self {
        Self {
            position,
            tag,
            auto_wrap: true,
        }
    }

    /// Disable automatic wrapping in override brackets
    #[must_use]
    pub fn no_auto_wrap(mut self) -> Self {
        self.auto_wrap = false;
        self
    }

    /// Validate and format tag for insertion
    fn format_tag(&self) -> Result<String> {
        let tag = self.tag.trim();

        // Validate tag format - should start with backslash
        if !tag.starts_with('\\') {
            return Err(EditorError::command_failed(format!(
                "ASS override tag must start with backslash: '{tag}'"
            )));
        }

        // Check if already wrapped in override brackets
        if tag.starts_with('{') && tag.ends_with('}') {
            return Ok(tag.to_string());
        }

        // Auto-wrap if enabled
        if self.auto_wrap {
            Ok(format!("{{{tag}}}"))
        } else {
            Ok(tag.to_string())
        }
    }
}

impl EditorCommand for InsertTagCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let formatted_tag = self.format_tag()?;
        document.insert_raw(self.position, &formatted_tag)?;

        let end_pos = Position::new(self.position.offset + formatted_tag.len());
        let range = Range::new(self.position, end_pos);

        Ok(CommandResult::success_with_change(range, end_pos))
    }

    fn description(&self) -> &str {
        "Insert ASS tag"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>() + self.tag.len()
    }
}
