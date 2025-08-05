//! Command system for editor operations
//!
//! Provides a trait-based command system with fluent APIs for text editing,
//! undo/redo support, and extensible command types. All commands are atomic
//! and can be undone/redone efficiently.

pub mod delta_commands;
pub mod event_commands;
pub mod karaoke_commands;
pub mod macros;
pub mod style_commands;
pub mod tag_commands;

use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(feature = "stream")]
use ass_core::parser::ScriptDeltaOwned;

// Re-export delta commands, event commands, karaoke commands, style commands, and tag commands
pub use delta_commands::*;
pub use event_commands::*;
pub use karaoke_commands::*;
pub use style_commands::*;
pub use tag_commands::*;

#[cfg(not(feature = "std"))]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};

/// Result of executing a command
///
/// Contains the modified document and optional metadata about the operation.
/// This will be used by the history system to track changes.
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Whether the command was successfully executed
    pub success: bool,

    /// Optional message about the operation
    pub message: Option<String>,

    /// The range of text that was modified (for cursor updates)
    pub modified_range: Option<Range>,

    /// New cursor position after the command
    pub new_cursor: Option<Position>,

    /// Whether the document content was changed
    pub content_changed: bool,

    /// Script delta for incremental parsing (when available)
    #[cfg(feature = "stream")]
    pub script_delta: Option<ScriptDeltaOwned>,
}

impl CommandResult {
    /// Create a successful command result
    pub fn success() -> Self {
        Self {
            success: true,
            message: None,
            modified_range: None,
            new_cursor: None,
            content_changed: false,
            #[cfg(feature = "stream")]
            script_delta: None,
        }
    }

    /// Create a successful result with content change
    pub fn success_with_change(range: Range, cursor: Position) -> Self {
        Self {
            success: true,
            message: None,
            modified_range: Some(range),
            new_cursor: Some(cursor),
            content_changed: true,
            #[cfg(feature = "stream")]
            script_delta: None,
        }
    }

    /// Create a failed command result
    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message: Some(message),
            modified_range: None,
            new_cursor: None,
            content_changed: false,
            #[cfg(feature = "stream")]
            script_delta: None,
        }
    }

    /// Add a script delta to the result
    #[cfg(feature = "stream")]
    #[must_use]
    pub fn with_delta(mut self, delta: ScriptDeltaOwned) -> Self {
        self.script_delta = Some(delta);
        self
    }

    /// Add a message to the result
    #[must_use]
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
}

/// Trait for editor commands that can be executed and undone
///
/// All commands implement this trait to provide a consistent interface
/// for execution, undo/redo, and introspection.
///
/// # Examples
///
/// Creating a custom command:
///
/// ```
/// use ass_editor::{EditorCommand, EditorDocument, CommandResult, Result, Position, Range};
///
/// #[derive(Debug)]
/// struct UppercaseCommand {
///     description: String,
/// }
///
/// impl UppercaseCommand {
///     fn new() -> Self {
///         Self {
///             description: "Convert to uppercase".to_string(),
///         }
///     }
/// }
///
/// impl EditorCommand for UppercaseCommand {
///     fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
///         let text = document.text().to_uppercase();
///         let range = Range::new(Position::new(0), Position::new(document.len()));
///         document.replace(range, &text)?;
///         Ok(CommandResult::success().with_message("Text converted to uppercase".to_string()))
///     }
///
///     fn description(&self) -> &str {
///         &self.description
///     }
/// }
/// ```
pub trait EditorCommand: core::fmt::Debug + Send + Sync {
    /// Execute the command on the given document
    ///
    /// Returns a result indicating success/failure and metadata about
    /// the operation for undo/redo tracking.
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult>;

    /// Get a human-readable description of the command
    fn description(&self) -> &str;

    /// Check if this command modifies document content
    ///
    /// Used to determine if the document should be marked as modified
    /// and whether to save undo state.
    fn modifies_content(&self) -> bool {
        true
    }

    /// Get the estimated memory usage of this command
    ///
    /// Used for memory management in undo stacks with limited capacity.
    /// Default implementation provides a conservative estimate.
    fn memory_usage(&self) -> usize {
        64 // Conservative default estimate for command overhead
    }
}

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

/// Batch command that executes multiple commands as a single atomic operation
#[derive(Debug)]
pub struct BatchCommand {
    /// Commands to execute in order
    pub commands: Vec<Box<dyn EditorCommand>>,
    /// Description of the batch operation
    pub description: String,
}

impl BatchCommand {
    /// Create a new batch command
    ///
    /// # Examples
    ///
    /// ```
    /// use ass_editor::{BatchCommand, InsertTextCommand, DeleteTextCommand, Position, Range, EditorDocument, EditorCommand};
    ///
    /// let mut doc = EditorDocument::from_content("Hello World").unwrap();
    ///
    /// let batch = BatchCommand::new("Multiple operations".to_string())
    ///     .add_command(Box::new(InsertTextCommand::new(Position::new(5), " beautiful".to_string())))
    ///     .add_command(Box::new(DeleteTextCommand::new(Range::new(Position::new(15), Position::new(21)))));
    ///
    /// let result = batch.execute(&mut doc).unwrap();
    /// assert!(result.success);
    /// assert_eq!(doc.text(), "Hello beautiful");
    /// ```
    pub fn new(description: String) -> Self {
        Self {
            commands: Vec::new(),
            description,
        }
    }

    /// Add a command to the batch
    pub fn add_command(mut self, command: Box<dyn EditorCommand>) -> Self {
        self.commands.push(command);
        self
    }

    /// Add multiple commands to the batch
    pub fn add_commands(mut self, commands: Vec<Box<dyn EditorCommand>>) -> Self {
        self.commands.extend(commands);
        self
    }
}

impl EditorCommand for BatchCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let mut overall_result = CommandResult::success();
        let mut first_range: Option<Range> = None;
        let mut last_cursor: Option<Position> = None;

        for command in &self.commands {
            let result = command.execute(document)?;

            if !result.success {
                return Ok(CommandResult::failure(format!(
                    "Batch command failed at: {}",
                    command.description()
                )));
            }

            // Track the overall range of changes
            if let Some(range) = result.modified_range {
                first_range = Some(match first_range {
                    Some(existing) => existing.union(&range),
                    None => range,
                });
            }

            if result.new_cursor.is_some() {
                last_cursor = result.new_cursor;
            }

            if result.content_changed {
                overall_result.content_changed = true;
            }
        }

        overall_result.modified_range = first_range;
        overall_result.new_cursor = last_cursor;

        Ok(overall_result)
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.description.len()
            + self
                .commands
                .iter()
                .map(|c| c.memory_usage())
                .sum::<usize>()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EditorDocument;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;
    #[cfg(not(feature = "std"))]
    #[test]
    fn insert_command_execution() {
        let mut doc = EditorDocument::new();
        let command = InsertTextCommand::new(Position::new(0), "Hello".to_string());

        let result = command.execute(&mut doc).unwrap();
        assert!(result.success);
        assert!(result.content_changed);
        assert_eq!(doc.text(), "Hello");
    }

    #[test]
    fn delete_command_execution() {
        let mut doc = EditorDocument::from_content("Hello World").unwrap();
        let range = Range::new(Position::new(6), Position::new(11));
        let command = DeleteTextCommand::new(range);

        let result = command.execute(&mut doc).unwrap();
        assert!(result.success);
        assert!(result.content_changed);
        assert_eq!(doc.text(), "Hello ");
    }

    #[test]
    fn replace_command_execution() {
        let mut doc = EditorDocument::from_content("Hello World").unwrap();
        let range = Range::new(Position::new(6), Position::new(11));
        let command = ReplaceTextCommand::new(range, "Rust".to_string());

        let result = command.execute(&mut doc).unwrap();
        assert!(result.success);
        assert!(result.content_changed);
        assert_eq!(doc.text(), "Hello Rust");
    }

    #[test]
    fn batch_command_execution() {
        let mut doc = EditorDocument::from_content("Hello").unwrap();

        let batch = BatchCommand::new("Insert and replace".to_string())
            .add_command(Box::new(InsertTextCommand::new(
                Position::new(5),
                " World".to_string(),
            )))
            .add_command(Box::new(ReplaceTextCommand::new(
                Range::new(Position::new(0), Position::new(5)),
                "Hi".to_string(),
            )));

        let result = batch.execute(&mut doc).unwrap();
        assert!(result.success);
        assert!(result.content_changed);
        assert_eq!(doc.text(), "Hi World");
    }

    #[test]
    fn fluent_api_usage() {
        let mut doc = EditorDocument::new();

        // Test fluent insertion
        let result = doc.command().at(Position::new(0)).insert("Hello").unwrap();

        assert!(result.success);
        assert_eq!(doc.text(), "Hello");

        // Test fluent replacement
        let range = Range::new(Position::new(0), Position::new(5));
        let result = doc.command().range(range).replace("Hi").unwrap();

        assert!(result.success);
        assert_eq!(doc.text(), "Hi");
    }

    #[test]
    fn document_extension_methods() {
        let mut doc = EditorDocument::new();

        // Test insert_at
        doc.insert_at(Position::new(0), "Hello").unwrap();
        assert_eq!(doc.text(), "Hello");

        // Test replace_range
        let range = Range::new(Position::new(0), Position::new(5));
        doc.replace_range(range, "Hi").unwrap();
        assert_eq!(doc.text(), "Hi");

        // Test delete_range
        let range = Range::new(Position::new(0), Position::new(2));
        doc.delete_range(range).unwrap();
        assert_eq!(doc.text(), "");
    }

    #[test]
    fn command_memory_usage() {
        let insert_cmd = InsertTextCommand::new(Position::new(0), "Hello".to_string());
        let usage = insert_cmd.memory_usage();

        // Should account for the struct size plus string length
        assert!(usage >= core::mem::size_of::<InsertTextCommand>() + 5);
    }
}
