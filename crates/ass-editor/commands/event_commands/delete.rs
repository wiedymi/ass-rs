//! Command to delete a single event from an ASS document.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{format, string::ToString};

/// Command to delete a single event from the ASS document
///
/// Removes an event (Dialogue or Comment) at the specified index from the `[Events]` section.
/// The index is 0-based and includes both Dialogue and Comment events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteEventCommand {
    /// Index of the event to delete
    pub event_index: usize,
}

impl DeleteEventCommand {
    /// Create a new delete event command
    pub fn new(event_index: usize) -> Self {
        Self { event_index }
    }
}

impl EditorCommand for DeleteEventCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let content = document.text().to_string();

        // Find [Events] section
        let events_start = content
            .find("[Events]")
            .ok_or_else(|| EditorError::command_failed("No [Events] section found"))?;

        // Find Format line end
        let format_line_end = content[events_start..]
            .find("Format:")
            .and_then(|format_pos| {
                content[events_start + format_pos..]
                    .find('\n')
                    .map(|newline_pos| events_start + format_pos + newline_pos + 1)
            })
            .ok_or_else(|| EditorError::command_failed("Invalid events section format"))?;

        let mut current_index = 0;
        let mut event_start = format_line_end;
        let mut delete_range: Option<Range> = None;

        while event_start < content.len() {
            let line_end = content[event_start..]
                .find('\n')
                .map(|pos| event_start + pos + 1) // Include newline
                .unwrap_or(content.len());

            if event_start >= line_end {
                break;
            }

            let line = &content[event_start..line_end.saturating_sub(1)]; // Check without newline

            if line.starts_with("Dialogue:") || line.starts_with("Comment:") {
                if current_index == self.event_index {
                    delete_range = Some(Range::new(
                        Position::new(event_start),
                        Position::new(line_end),
                    ));
                    break;
                }
                current_index += 1;
            }

            event_start = line_end;
        }

        if let Some(range) = delete_range {
            document.delete(range)?;
            Ok(CommandResult::success_with_change(
                Range::new(range.start, range.start),
                range.start,
            ))
        } else {
            Err(EditorError::command_failed(format!(
                "Event index {} not found",
                self.event_index
            )))
        }
    }

    fn description(&self) -> &str {
        "Delete event"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
    }
}
