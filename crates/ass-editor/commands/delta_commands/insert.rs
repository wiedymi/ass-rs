//! Incremental insert command with delta tracking

use super::DeltaCommand;
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

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
