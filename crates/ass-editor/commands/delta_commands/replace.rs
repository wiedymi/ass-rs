//! Incremental replace command with delta tracking

use super::DeltaCommand;
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

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
