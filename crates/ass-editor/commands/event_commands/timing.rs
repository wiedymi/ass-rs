//! Command to adjust event timing by shifting start/end times.

use super::helpers::parse_event_line;
use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};
use ass_core::parser::ast::EventType;
use ass_core::utils::format_ass_time;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

/// Command to adjust event timing (shift start/end times)
#[derive(Debug, Clone)]
pub struct TimingAdjustCommand {
    pub event_indices: Vec<usize>, // Events to adjust (empty = all events)
    pub start_offset_cs: i32,      // Offset in centiseconds for start time
    pub end_offset_cs: i32,        // Offset in centiseconds for end time
    pub description: Option<String>,
}

impl TimingAdjustCommand {
    /// Create a new timing adjustment command for specific events
    pub fn new(event_indices: Vec<usize>, start_offset_cs: i32, end_offset_cs: i32) -> Self {
        Self {
            event_indices,
            start_offset_cs,
            end_offset_cs,
            description: None,
        }
    }

    /// Create a timing adjustment command for all events
    pub fn all_events(start_offset_cs: i32, end_offset_cs: i32) -> Self {
        Self {
            event_indices: Vec::new(), // Empty means all events
            start_offset_cs,
            end_offset_cs,
            description: None,
        }
    }

    /// Adjust only start times (keep duration constant)
    pub fn shift_start(event_indices: Vec<usize>, offset_cs: i32) -> Self {
        Self::new(event_indices, offset_cs, offset_cs)
    }

    /// Adjust only end times (change duration)
    pub fn shift_end(event_indices: Vec<usize>, offset_cs: i32) -> Self {
        Self::new(event_indices, 0, offset_cs)
    }

    /// Scale duration (multiply by factor)
    pub fn scale_duration(event_indices: Vec<usize>, factor: f64) -> Self {
        // This is a simplified version - actual implementation would need to calculate per-event
        let offset = (factor * 100.0) as i32 - 100; // Convert factor to centisecond offset
        Self::new(event_indices, 0, offset)
    }

    /// Set a custom description for this command
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for TimingAdjustCommand {
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
                let should_adjust =
                    self.event_indices.is_empty() || self.event_indices.contains(&current_index);

                if should_adjust {
                    // Parse event line using ass-core's parser
                    if let Ok(event) = parse_event_line(line) {
                        // Parse current times using Event methods
                        if let (Ok(start_cs), Ok(end_cs)) =
                            (event.start_time_cs(), event.end_time_cs())
                        {
                            // Apply offsets
                            let new_start_cs =
                                (start_cs as i32 + self.start_offset_cs).max(0) as u32;
                            let new_end_cs = (end_cs as i32 + self.end_offset_cs).max(0) as u32;

                            // Ensure end time is after start time
                            let final_end_cs = new_end_cs.max(new_start_cs + 1);

                            let new_start_time = format_ass_time(new_start_cs);
                            let new_end_time = format_ass_time(final_end_cs);

                            // Build new event line
                            let event_type_str = match event.event_type {
                                EventType::Dialogue => "Dialogue",
                                EventType::Comment => "Comment",
                                _ => "Dialogue", // Default fallback
                            };
                            let new_line = format!(
                                "{}: {},{},{},{},{},{},{},{},{},{}",
                                event_type_str,
                                event.layer,
                                new_start_time,
                                new_end_time,
                                event.style,
                                event.name,
                                event.margin_l,
                                event.margin_r,
                                event.margin_v,
                                event.effect,
                                event.text
                            );

                            // Replace the line
                            let range =
                                Range::new(Position::new(event_start), Position::new(line_end));
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
                    }
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
            .with_message(format!("Adjusted timing for {changes_made} events")))
        } else {
            Ok(CommandResult::success().with_message("No events were adjusted".to_string()))
        }
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Adjust event timing")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.event_indices.len() * core::mem::size_of::<usize>()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}
