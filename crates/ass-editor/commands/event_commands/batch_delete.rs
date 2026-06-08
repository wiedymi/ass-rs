//! Command to delete multiple events from an ASS document.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

/// Command to delete multiple events from the ASS document
///
/// Removes multiple events (Dialogue or Comment) at the specified indices from the `[Events]` section.
/// Indices are automatically sorted and processed in reverse order to maintain correctness.
/// All indices are 0-based and include both Dialogue and Comment events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchDeleteEventsCommand {
    /// Indices of events to delete (will be sorted and deduplicated)
    pub event_indices: Vec<usize>,
}

impl BatchDeleteEventsCommand {
    /// Create a new batch delete events command
    pub fn new(event_indices: Vec<usize>) -> Self {
        Self { event_indices }
    }
}

impl EditorCommand for BatchDeleteEventsCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        if self.event_indices.is_empty() {
            return Ok(CommandResult::success());
        }

        // Sort indices in descending order to delete from end to start
        // This prevents index shifting issues
        let mut sorted_indices = self.event_indices.clone();
        sorted_indices.sort_unstable_by(|a, b| b.cmp(a));
        sorted_indices.dedup();

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

        // Collect all event positions
        let mut event_positions = Vec::new();
        let mut current_index = 0;
        let mut event_start = format_line_end;

        while event_start < content.len() {
            let line_end = content[event_start..]
                .find('\n')
                .map(|pos| event_start + pos + 1) // Include newline
                .unwrap_or(content.len());

            if event_start >= line_end {
                break;
            }

            let line = &content[event_start..line_end.saturating_sub(1)];

            if line.starts_with("Dialogue:") || line.starts_with("Comment:") {
                event_positions.push((current_index, event_start, line_end));
                current_index += 1;
            }

            event_start = line_end;
        }

        // Delete events in reverse order to avoid index shifting
        let mut total_deleted = 0;
        let mut first_delete_pos = Position::new(content.len());

        for index in &sorted_indices {
            if let Some((_, start, end)) = event_positions.iter().find(|(idx, _, _)| idx == index) {
                let range = Range::new(Position::new(*start), Position::new(*end));
                document.delete(range)?;
                total_deleted += 1;

                // Track the earliest deletion position
                if range.start.offset < first_delete_pos.offset {
                    first_delete_pos = range.start;
                }
            }
        }

        if total_deleted > 0 {
            Ok(CommandResult::success_with_change(
                Range::new(first_delete_pos, first_delete_pos),
                first_delete_pos,
            ))
        } else {
            Ok(CommandResult::success())
        }
    }

    fn description(&self) -> &str {
        "Delete multiple events"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>() + self.event_indices.len() * core::mem::size_of::<usize>()
    }
}
