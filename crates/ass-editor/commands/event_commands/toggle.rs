//! Command to toggle event type between Dialogue and Comment.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

/// Command to toggle event type between Dialogue and Comment
#[derive(Debug, Clone)]
pub struct ToggleEventTypeCommand {
    pub event_indices: Vec<usize>,
    pub description: Option<String>,
}

impl ToggleEventTypeCommand {
    /// Create a new event type toggle command
    pub fn new(event_indices: Vec<usize>) -> Self {
        Self {
            event_indices,
            description: None,
        }
    }

    /// Toggle a single event
    pub fn single(event_index: usize) -> Self {
        Self::new(vec![event_index])
    }

    /// Toggle all events
    pub fn all() -> Self {
        Self::new(Vec::new()) // Empty means all events
    }

    /// Set a custom description for this command
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for ToggleEventTypeCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let mut content = document.text();
        let events_start = content
            .find("[Events]")
            .ok_or_else(|| EditorError::command_failed("Events section not found"))?;

        let events_content = &content[events_start..];
        let format_line_end = events_content
            .find("Format:")
            .and_then(|format_pos| {
                events_content[format_pos..]
                    .find('\n')
                    .map(|newline_pos| events_start + format_pos + newline_pos + 1)
            })
            .ok_or_else(|| EditorError::command_failed("Invalid events section format"))?;

        let mut changes_made = 0;
        let mut current_index = 0;
        let mut event_start = format_line_end;
        let mut total_range: Option<Range> = None;

        while event_start < content.len() {
            let line_end = content[event_start..]
                .find('\n')
                .map(|pos| event_start + pos)
                .unwrap_or(content.len());

            if event_start >= line_end {
                break;
            }

            let line = &content[event_start..line_end];

            if line.starts_with("Dialogue:") || line.starts_with("Comment:") {
                let should_toggle =
                    self.event_indices.is_empty() || self.event_indices.contains(&current_index);

                if should_toggle {
                    let new_line = if line.starts_with("Dialogue:") {
                        line.replacen("Dialogue:", "Comment:", 1)
                    } else {
                        line.replacen("Comment:", "Dialogue:", 1)
                    };

                    let range = Range::new(Position::new(event_start), Position::new(line_end));
                    document.replace(range, &new_line)?;

                    // Update content for next iteration
                    content = document.text();

                    // Track overall range
                    let change_range = Range::new(
                        Position::new(event_start),
                        Position::new(event_start + new_line.len()),
                    );
                    total_range = Some(match total_range {
                        Some(existing) => existing.union(&change_range),
                        None => change_range,
                    });

                    changes_made += 1;
                }
                current_index += 1;
            } else if line.starts_with('[') {
                break;
            }

            event_start = line_end + 1;
        }

        if changes_made > 0 {
            Ok(CommandResult::success_with_change(
                total_range.unwrap_or(Range::new(Position::new(0), Position::new(0))),
                Position::new(content.len()),
            )
            .with_message(format!("Toggled type for {changes_made} events")))
        } else {
            Ok(CommandResult::success().with_message("No events were toggled".to_string()))
        }
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Toggle event type")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.event_indices.len() * core::mem::size_of::<usize>()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}
