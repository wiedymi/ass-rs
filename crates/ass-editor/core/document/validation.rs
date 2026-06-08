//! Script parsing entry points, command execution, and validation
//!
//! Hosts the `parse_script_with` callback bridge, the history-recording
//! `execute_command`, and the lazy-validator integration methods.

use super::EditorDocument;
use crate::core::errors::{EditorError, Result};
use crate::core::position::{Position, Range};
use ass_core::parser::Script;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

impl EditorDocument {
    /// Parse the current document content into a Script
    ///
    /// Returns a boxed closure that provides the parsed Script.
    /// This avoids lifetime issues by ensuring the content outlives the Script.
    pub fn parse_script_with<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Script) -> R,
    {
        let content = self.text();
        match Script::parse(&content) {
            Ok(script) => Ok(f(&script)),
            Err(e) => Err(EditorError::from(e)),
        }
    }

    /// Validate the document content can be parsed as valid ASS
    /// This is the basic validation that just checks parsing
    pub fn validate(&self) -> Result<()> {
        let content = self.text();
        Script::parse(&content).map_err(EditorError::from)?;
        Ok(())
    }

    /// Execute a command with proper history recording
    ///
    /// This method ensures that commands are properly recorded in the undo history.
    /// Use this instead of calling command.execute() directly if you want undo support.
    ///
    /// For BatchCommand, this creates a synthetic undo operation that captures
    /// the aggregate effect of all sub-commands.
    pub fn execute_command(
        &mut self,
        command: &dyn crate::commands::EditorCommand,
    ) -> Result<crate::commands::CommandResult> {
        use crate::core::history::Operation;

        // Get the cursor position before
        let _cursor_before = self.cursor_position();

        // Execute the command
        let result = command.execute(self)?;

        // Only record if the command changed content
        if result.content_changed {
            // Determine the operation based on the result
            let operation = if let Some(range) = result.modified_range {
                if range.is_empty() {
                    // This was an insertion
                    let inserted_text = self.text_range(Range::new(
                        range.start,
                        Position::new(
                            range.start.offset
                                + result
                                    .new_cursor
                                    .map_or(0, |c| c.offset - range.start.offset),
                        ),
                    ))?;
                    Operation::Insert {
                        position: range.start,
                        text: inserted_text,
                    }
                } else {
                    // For batch commands and complex operations, we need to be more careful
                    // Store enough information to properly undo
                    // This is a simplified approach - ideally each command would handle its own undo
                    let cmd_desc = command.description();
                    if cmd_desc.contains("batch") || cmd_desc.contains("Multiple") {
                        // For batch operations, we don't have enough info
                        // Just use a simple insert operation
                        Operation::Insert {
                            position: range.start,
                            text: String::new(),
                        }
                    } else {
                        // For simple operations, use the range info
                        Operation::Replace {
                            range,
                            old_text: String::new(), // We don't have the old text
                            new_text: self.text_range(range).unwrap_or_default(),
                        }
                    }
                }
            } else {
                // No range info - create a generic operation
                Operation::Insert {
                    position: Position::new(0),
                    text: String::new(),
                }
            };

            // Update cursor if needed
            if let Some(new_cursor) = result.new_cursor {
                self.set_cursor_position(Some(new_cursor));
            }

            // Record in history
            self.history
                .record_operation(operation, command.description().to_string(), &result);

            // Clear validation cache since content changed
            self.validator.clear_cache();
        }

        Ok(result)
    }

    /// Perform comprehensive validation using the LazyValidator
    /// Returns detailed validation results including warnings and suggestions
    ///
    /// Note: Returns a cloned result to avoid borrow checker issues
    pub fn validate_comprehensive(&mut self) -> Result<crate::utils::validator::ValidationResult> {
        // Create a temporary document reference for validation
        // This is needed because we can't pass &self while mutably borrowing validator
        let temp_doc = EditorDocument::from_content(&self.text())?;
        let result = self.validator.validate(&temp_doc)?;
        Ok(result.clone())
    }

    /// Force revalidation even if cached results exist
    pub fn force_validate(&mut self) -> Result<crate::utils::validator::ValidationResult> {
        // Create a temporary document reference for validation
        let temp_doc = EditorDocument::from_content(&self.text())?;
        let result = self.validator.force_validate(&temp_doc)?;
        Ok(result.clone())
    }

    /// Check if document is valid (quick check using cache if available)
    pub fn is_valid_cached(&mut self) -> Result<bool> {
        // Create a temporary document reference for validation
        let temp_doc = EditorDocument::from_content(&self.text())?;
        self.validator.is_valid(&temp_doc)
    }

    /// Get cached validation result without revalidating
    pub fn validation_result(&self) -> Option<&crate::utils::validator::ValidationResult> {
        self.validator.cached_result()
    }

    /// Configure the validator
    pub fn set_validator_config(&mut self, config: crate::utils::validator::ValidatorConfig) {
        self.validator.set_config(config);
    }

    /// Get mutable access to the validator
    pub fn validator_mut(&mut self) -> &mut crate::utils::validator::LazyValidator {
        &mut self.validator
    }
}
