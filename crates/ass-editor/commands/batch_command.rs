//! Batch command that runs multiple commands as a single atomic operation.

use crate::core::{EditorDocument, Position, Range, Result};

use super::{CommandResult, EditorCommand};

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, format, string::String, vec::Vec};

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
