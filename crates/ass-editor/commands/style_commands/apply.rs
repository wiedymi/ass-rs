//! Style application command for ASS documents.
//!
//! Provides [`ApplyStyleCommand`] to retarget events from one style to
//! another, with an optional text filter on event lines.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

/// Command to apply a style to events (change all events using one style to another)
#[derive(Debug, Clone)]
pub struct ApplyStyleCommand {
    pub old_style: String,
    pub new_style: String,
    pub event_filter: Option<String>, // Optional filter for event text
    pub description: Option<String>,
}

impl ApplyStyleCommand {
    /// Create a new style application command
    pub fn new(old_style: String, new_style: String) -> Self {
        Self {
            old_style,
            new_style,
            event_filter: None,
            description: None,
        }
    }

    /// Only apply to events containing specific text
    pub fn with_filter(mut self, filter: String) -> Self {
        self.event_filter = Some(filter);
        self
    }

    /// Set custom description
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for ApplyStyleCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let mut content = document.text();
        let mut changes_made = 0;
        let mut total_range: Option<Range> = None;

        // Find events section
        let events_start = content
            .find("[Events]")
            .ok_or_else(|| EditorError::command_failed("Events section not found"))?;

        // Skip format line
        let events_content_start = content[events_start..]
            .find('\n')
            .map(|pos| events_start + pos + 1)
            .ok_or_else(|| EditorError::command_failed("Invalid events section format"))?;

        // Skip format line again (Format: ...)
        let first_event_start = content[events_content_start..]
            .find('\n')
            .map(|pos| events_content_start + pos + 1)
            .unwrap_or(events_content_start);

        let mut search_pos = first_event_start;

        while search_pos < content.len() {
            // Find next event line
            let line_start = search_pos;
            let line_end = content[line_start..]
                .find('\n')
                .map(|pos| line_start + pos)
                .unwrap_or(content.len());

            if line_start >= line_end {
                break;
            }

            let line = &content[line_start..line_end];

            // Check if this is an event line
            if line.starts_with("Dialogue:") || line.starts_with("Comment:") {
                let parts: Vec<&str> = line.split(',').collect();

                // Check if this event uses the old style (typically 4th field after event type)
                if parts.len() > 4 && parts[3].trim() == self.old_style {
                    // Apply filter if specified
                    let should_update = if let Some(ref filter) = self.event_filter {
                        line.contains(filter)
                    } else {
                        true
                    };

                    if should_update {
                        // Replace the old style with new style
                        let updated_line = line.replace(
                            &format!(",{},", self.old_style),
                            &format!(",{},", self.new_style),
                        );

                        // Update the document
                        let range = Range::new(Position::new(line_start), Position::new(line_end));
                        document.replace(range, &updated_line)?;

                        // Update content for next iteration (this is inefficient but correct)
                        content = document.text();

                        // Track overall range
                        let change_range = Range::new(
                            Position::new(line_start),
                            Position::new(line_start + updated_line.len()),
                        );
                        total_range = Some(match total_range {
                            Some(existing) => existing.union(&change_range),
                            None => change_range,
                        });

                        changes_made += 1;
                    }
                }
            } else if line.starts_with('[') && line != "[Events]" {
                // Stop at next section
                break;
            }

            search_pos = line_end + 1;
        }

        if changes_made > 0 {
            Ok(CommandResult::success_with_change(
                total_range.unwrap_or(Range::new(Position::new(0), Position::new(0))),
                Position::new(content.len()),
            )
            .with_message(format!(
                "Applied style '{}' to {} events",
                self.new_style, changes_made
            )))
        } else {
            Ok(CommandResult::success()
                .with_message("No events found matching the criteria".to_string()))
        }
    }

    fn description(&self) -> &str {
        self.description
            .as_deref()
            .unwrap_or("Apply style to events")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.old_style.len()
            + self.new_style.len()
            + self.event_filter.as_ref().map_or(0, |f| f.len())
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}
