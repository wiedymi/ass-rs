//! Primitive text-editing commands: insert, delete, and replace.

use crate::core::{EditorDocument, Position, Range, Result};

use super::{CommandResult, EditorCommand};

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Text insertion command
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertTextCommand {
    /// Position to insert text at
    pub position: Position,
    /// Text to insert
    pub text: String,
    /// Optional description override
    pub description: Option<String>,
}

impl InsertTextCommand {
    /// Create a new insert text command
    ///
    /// # Examples
    ///
    /// ```
    /// use ass_editor::{InsertTextCommand, EditorDocument, Position, EditorCommand};
    ///
    /// let mut doc = EditorDocument::new();
    /// let command = InsertTextCommand::new(Position::new(0), "Hello World".to_string());
    ///
    /// let result = command.execute(&mut doc).unwrap();
    /// assert!(result.success);
    /// assert_eq!(doc.text(), "Hello World");
    /// ```
    pub fn new(position: Position, text: String) -> Self {
        Self {
            position,
            text,
            description: None,
        }
    }

    /// Set a custom description for this command
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for InsertTextCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        document.insert_raw(self.position, &self.text)?;

        let end_pos = Position::new(self.position.offset + self.text.len());
        let range = Range::new(self.position, end_pos);

        Ok(CommandResult::success_with_change(range, end_pos))
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Insert text")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.text.len()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}

/// Text deletion command
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteTextCommand {
    /// Range of text to delete
    pub range: Range,
    /// Optional description override
    pub description: Option<String>,
}

impl DeleteTextCommand {
    /// Create a new delete text command
    pub fn new(range: Range) -> Self {
        Self {
            range,
            description: None,
        }
    }

    /// Set a custom description for this command
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for DeleteTextCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        document.delete_raw(self.range)?;

        let cursor_pos = self.range.start;
        let range = Range::new(self.range.start, self.range.start);

        Ok(CommandResult::success_with_change(range, cursor_pos))
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Delete text")
    }
}

/// Text replacement command
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceTextCommand {
    /// Range of text to replace
    pub range: Range,
    /// New text to insert
    pub new_text: String,
    /// Optional description override
    pub description: Option<String>,
}

impl ReplaceTextCommand {
    /// Create a new replace text command
    pub fn new(range: Range, new_text: String) -> Self {
        Self {
            range,
            new_text,
            description: None,
        }
    }

    /// Set a custom description for this command
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for ReplaceTextCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        document.replace_raw(self.range, &self.new_text)?;

        let end_pos = Position::new(self.range.start.offset + self.new_text.len());
        let range = Range::new(self.range.start, end_pos);

        Ok(CommandResult::success_with_change(range, end_pos))
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Replace text")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.new_text.len()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}
