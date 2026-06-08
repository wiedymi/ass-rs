//! ASS-aware event edit command with delta tracking

use super::DeltaCommand;
use crate::core::{EditorDocument, Result};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String};

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
