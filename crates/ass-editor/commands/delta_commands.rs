//! Delta-aware commands for incremental ASS editing
//!
//! Commands that use ass-core's Delta tracking for optimal performance

use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// A command that tracks deltas for incremental updates
pub trait DeltaCommand {
    /// Execute the command and return the result with delta information
    fn execute_with_delta(&self, document: &mut EditorDocument) -> Result<super::CommandResult>;

    /// Get command description for history
    fn description(&self) -> String;

    /// Check if this command can be executed incrementally
    fn supports_incremental(&self) -> bool {
        cfg!(feature = "stream")
    }
}

/// Insert text command with delta tracking
#[derive(Debug, Clone)]
pub struct IncrementalInsertCommand {
    pub position: Position,
    pub text: String,
}

impl IncrementalInsertCommand {
    /// Create a new incremental insert command
    pub fn new(position: Position, text: String) -> Self {
        Self { position, text }
    }
}

impl DeltaCommand for IncrementalInsertCommand {
    fn execute_with_delta(&self, document: &mut EditorDocument) -> Result<super::CommandResult> {
        #[cfg(feature = "stream")]
        {
            if self.supports_incremental() {
                // Use incremental parsing for optimal performance
                match document.insert_incremental(self.position, &self.text) {
                    Ok(delta) => {
                        let result = super::CommandResult {
                            success: true,
                            message: Some(format!(
                                "Inserted text at position {}",
                                self.position.offset
                            )),
                            modified_range: Some(Range::new(
                                self.position,
                                Position::new(self.position.offset + self.text.len()),
                            )),
                            new_cursor: Some(Position::new(self.position.offset + self.text.len())),
                            content_changed: true,
                            script_delta: Some(delta),
                        };
                        return Ok(result);
                    }
                    Err(_) => {
                        // Fallback to regular insert
                    }
                }
            }
        }

        // Regular insert without delta tracking
        document.insert(self.position, &self.text)?;
        let result = super::CommandResult::success_with_change(
            Range::new(
                self.position,
                Position::new(self.position.offset + self.text.len()),
            ),
            Position::new(self.position.offset + self.text.len()),
        );
        Ok(result)
    }

    fn description(&self) -> String {
        format!(
            "Insert '{}' at position {}",
            self.text, self.position.offset
        )
    }
}

/// Replace text command with delta tracking
#[derive(Debug, Clone)]
pub struct IncrementalReplaceCommand {
    pub range: Range,
    pub new_text: String,
}

impl IncrementalReplaceCommand {
    /// Create a new incremental replace command
    pub fn new(range: Range, new_text: String) -> Self {
        Self { range, new_text }
    }
}

impl DeltaCommand for IncrementalReplaceCommand {
    fn execute_with_delta(&self, document: &mut EditorDocument) -> Result<super::CommandResult> {
        #[cfg(feature = "stream")]
        {
            if self.supports_incremental() {
                // Use incremental parsing
                match document.edit_incremental(self.range, &self.new_text) {
                    Ok(delta) => {
                        let result = super::CommandResult {
                            success: true,
                            message: Some(format!(
                                "Replaced text in range {}..{}",
                                self.range.start.offset, self.range.end.offset
                            )),
                            modified_range: Some(Range::new(
                                self.range.start,
                                Position::new(self.range.start.offset + self.new_text.len()),
                            )),
                            new_cursor: Some(Position::new(
                                self.range.start.offset + self.new_text.len(),
                            )),
                            content_changed: true,
                            script_delta: Some(delta),
                        };
                        return Ok(result);
                    }
                    Err(_) => {
                        // Fallback to regular replace
                    }
                }
            }
        }

        // Regular replace
        document.replace(self.range, &self.new_text)?;
        let result = super::CommandResult::success_with_change(
            Range::new(
                self.range.start,
                Position::new(self.range.start.offset + self.new_text.len()),
            ),
            Position::new(self.range.start.offset + self.new_text.len()),
        );
        Ok(result)
    }

    fn description(&self) -> String {
        format!(
            "Replace text in range {}..{} with '{}'",
            self.range.start.offset, self.range.end.offset, self.new_text
        )
    }
}

/// ASS-aware event edit command with delta tracking
#[derive(Debug, Clone)]
pub struct IncrementalEventEditCommand {
    pub old_text: String,
    pub new_text: String,
}

impl IncrementalEventEditCommand {
    /// Create a new incremental event edit command
    pub fn new(old_text: String, new_text: String) -> Self {
        Self { old_text, new_text }
    }
}

impl DeltaCommand for IncrementalEventEditCommand {
    fn execute_with_delta(&self, document: &mut EditorDocument) -> Result<super::CommandResult> {
        #[cfg(feature = "stream")]
        {
            if self.supports_incremental() {
                // Use ASS-aware incremental editing
                match document.edit_event_incremental(&self.old_text, &self.new_text) {
                    Ok(delta) => {
                        let result = super::CommandResult {
                            success: true,
                            message: Some(format!(
                                "Edited event: '{}' → '{}'",
                                self.old_text, self.new_text
                            )),
                            modified_range: None, // Would need to track position
                            new_cursor: None,
                            content_changed: true,
                            script_delta: Some(delta),
                        };
                        return Ok(result);
                    }
                    Err(_) => {
                        // Fallback to regular event edit
                    }
                }
            }
        }

        // Regular event edit
        document.edit_event_text(&self.old_text, &self.new_text)?;
        let result = super::CommandResult {
            success: true,
            message: Some(format!(
                "Edited event: '{}' → '{}'",
                self.old_text, self.new_text
            )),
            modified_range: None,
            new_cursor: None,
            content_changed: true,
            #[cfg(feature = "stream")]
            script_delta: None,
        };
        Ok(result)
    }

    fn description(&self) -> String {
        format!("Edit event: '{}' → '{}'", self.old_text, self.new_text)
    }
}

/// Batch command that executes multiple delta commands efficiently
pub struct DeltaBatchCommand {
    commands: Vec<Box<dyn DeltaCommand>>,
}

impl DeltaBatchCommand {
    /// Create a new batch command
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Add a command to the batch
    pub fn add_command<T: DeltaCommand + 'static>(mut self, command: T) -> Self {
        self.commands.push(Box::new(command));
        self
    }

    /// Execute all commands in the batch
    pub fn execute_batch(
        &self,
        document: &mut EditorDocument,
    ) -> Result<Vec<super::CommandResult>> {
        let mut results = Vec::new();

        for command in &self.commands {
            let result = command.execute_with_delta(document)?;
            results.push(result);
        }

        Ok(results)
    }
}

impl Default for DeltaBatchCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EditorDocument;

    #[test]
    fn test_incremental_insert_command() {
        let mut doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();

        let command = IncrementalInsertCommand::new(
            Position::new(doc.len_bytes()),
            "\nAuthor: Test Author".to_string(),
        );

        let result = command.execute_with_delta(&mut doc).unwrap();
        assert!(result.success);
        assert!(result.content_changed);
        assert!(doc.text().contains("Author: Test Author"));
    }

    #[test]
    fn test_incremental_event_edit_command() {
        let content = r#"[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:05.00,0:00:10.00,Default,John,0,0,0,,Hello, world!"#;

        let mut doc = EditorDocument::from_content(content).unwrap();

        let command = IncrementalEventEditCommand::new(
            "Hello, world!".to_string(),
            "Hello, ASS-RS!".to_string(),
        );

        let result = command.execute_with_delta(&mut doc).unwrap();
        assert!(result.success);
        assert!(doc.text().contains("Hello, ASS-RS!"));
    }

    #[test]
    fn test_delta_batch_command() {
        let mut doc = EditorDocument::from_content("[Script Info]\nTitle: Test").unwrap();

        let batch = DeltaBatchCommand::new()
            .add_command(IncrementalInsertCommand::new(
                Position::new(doc.len_bytes()),
                "\nAuthor: Test".to_string(),
            ))
            .add_command(IncrementalInsertCommand::new(
                Position::new(doc.len_bytes() + "\nAuthor: Test".len()),
                "\nVersion: 1.0".to_string(),
            ));

        let results = batch.execute_batch(&mut doc).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.success));
        assert!(doc.text().contains("Author: Test"));
        assert!(doc.text().contains("Version: 1.0"));
    }
}
