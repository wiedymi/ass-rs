//! Fluent builder API (`TextCommand`) and the `DocumentCommandExt` trait.

use crate::core::{EditorDocument, EditorError, Position, Range, Result};

use super::{
    CommandResult, DeleteTextCommand, EditorCommand, InsertTextCommand, ReplaceTextCommand,
};

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

/// Fluent API builder for creating and executing commands
///
/// Provides an ergonomic way to build and execute commands:
/// ```
/// use ass_editor::{EditorDocument, Position, TextCommand};
///
/// let mut doc = EditorDocument::from_content("Hello world!").unwrap();
/// let position = Position::new(5);
///
/// let result = TextCommand::new(&mut doc)
///     .at(position)
///     .insert(" beautiful")
///     .unwrap();
///
/// assert_eq!(doc.text(), "Hello beautiful world!");
/// ```
pub struct TextCommand<'a> {
    document: &'a mut EditorDocument,
    position: Option<Position>,
    range: Option<Range>,
}

impl<'a> TextCommand<'a> {
    /// Create a new text command builder
    pub fn new(document: &'a mut EditorDocument) -> Self {
        Self {
            document,
            position: None,
            range: None,
        }
    }

    /// Set the position for the operation
    #[must_use]
    pub fn at(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    /// Set the range for the operation
    #[must_use]
    pub fn range(mut self, range: Range) -> Self {
        self.range = Some(range);
        self
    }

    /// Insert text at the current position
    pub fn insert(self, text: &str) -> Result<CommandResult> {
        let position = self
            .position
            .ok_or_else(|| EditorError::command_failed("Position not set for insert operation"))?;

        let command = InsertTextCommand::new(position, text.to_string());
        command.execute(self.document)
    }

    /// Delete text in the current range
    pub fn delete(self) -> Result<CommandResult> {
        let range = self
            .range
            .ok_or_else(|| EditorError::command_failed("Range not set for delete operation"))?;

        let command = DeleteTextCommand::new(range);
        command.execute(self.document)
    }

    /// Replace text in the current range
    pub fn replace(self, new_text: &str) -> Result<CommandResult> {
        let range = self
            .range
            .ok_or_else(|| EditorError::command_failed("Range not set for replace operation"))?;

        let command = ReplaceTextCommand::new(range, new_text.to_string());
        command.execute(self.document)
    }
}

// Extension trait to add fluent command methods to EditorDocument
pub trait DocumentCommandExt {
    /// Start a fluent command chain
    fn command(&mut self) -> TextCommand<'_>;

    /// Quick insert at position
    fn insert_at(&mut self, position: Position, text: &str) -> Result<CommandResult>;

    /// Quick delete range
    fn delete_range(&mut self, range: Range) -> Result<CommandResult>;

    /// Quick replace range
    fn replace_range(&mut self, range: Range, text: &str) -> Result<CommandResult>;
}

impl DocumentCommandExt for EditorDocument {
    fn command(&mut self) -> TextCommand<'_> {
        TextCommand::new(self)
    }

    fn insert_at(&mut self, position: Position, text: &str) -> Result<CommandResult> {
        let command = InsertTextCommand::new(position, text.to_string());
        command.execute(self)
    }

    fn delete_range(&mut self, range: Range) -> Result<CommandResult> {
        let command = DeleteTextCommand::new(range);
        command.execute(self)
    }

    fn replace_range(&mut self, range: Range, text: &str) -> Result<CommandResult> {
        let command = ReplaceTextCommand::new(range, text.to_string());
        command.execute(self)
    }
}
