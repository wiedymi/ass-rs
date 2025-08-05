//! Event management commands for ASS documents
//!
//! Provides commands for splitting, merging, timing adjustments, toggling event types,
//! and effect modifications with proper validation and delta tracking.

use super::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};
use ass_core::parser::ast::{Event, EventType, Span};
use ass_core::utils::{format_ass_time, parse_ass_time};

/// Helper function to parse an ASS event line with proper comma handling
/// Returns parsed Event struct or error if parsing fails
fn parse_event_line(line: &str) -> core::result::Result<Event, EditorError> {
    // Extract event type
    let colon_pos = line
        .find(':')
        .ok_or_else(|| EditorError::command_failed("Invalid event format: missing colon"))?;
    let event_type_str = &line[..colon_pos];
    let fields_part = line[colon_pos + 1..].trim();

    let event_type = match event_type_str {
        "Dialogue" => EventType::Dialogue,
        "Comment" => EventType::Comment,
        _ => return Err(EditorError::command_failed("Unknown event type")),
    };

    // Parse fields carefully - Effect field can contain commas, so we need special handling
    // Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
    let parts: Vec<&str> = fields_part.splitn(10, ',').collect();
    if parts.len() < 10 {
        return Err(EditorError::command_failed(
            "Invalid event format: insufficient fields",
        ));
    }

    // The issue is that parts[8] (Effect) and parts[9] (Text) might be incorrectly split
    // if Effect contains commas. We need to rejoin them properly.

    // First 8 fields are safe (no commas expected)
    let layer = parts[0].trim();
    let start = parts[1].trim();
    let end = parts[2].trim();
    let style = parts[3].trim();
    let name = parts[4].trim();
    let margin_l = parts[5].trim();
    let margin_r = parts[6].trim();
    let margin_v = parts[7].trim();

    // For Effect and Text, we need to find the correct comma that separates them
    // Effect can contain commas, but Text is the final field

    // Calculate where effect+text starts in the original string
    let prefix_len = parts[0..8].iter().map(|s| s.len()).sum::<usize>() + 8; // +8 for commas
    let remaining = &fields_part[prefix_len..];

    // Find the last comma that's not inside parentheses
    let mut split_point = None;
    let chars: Vec<char> = remaining.chars().collect();
    let mut paren_depth = 0;

    for (i, &ch) in chars.iter().enumerate().rev() {
        match ch {
            ')' => paren_depth += 1,
            '(' => paren_depth -= 1,
            ',' if paren_depth == 0 => {
                split_point = Some(i);
                break;
            }
            _ => {}
        }
    }

    let (effect, text) = if let Some(split) = split_point {
        let effect_part = remaining[..split].trim();
        let text_part = remaining[split + 1..].trim();
        (effect_part, text_part)
    } else {
        // No comma found outside parentheses, treat entire remaining as effect
        (remaining.trim(), "")
    };

    Ok(Event {
        event_type,
        layer,
        start,
        end,
        style,
        name,
        margin_l,
        margin_r,
        margin_v,
        margin_t: None,
        margin_b: None,
        effect,
        text,
        span: Span::new(0, line.len(), 1, 1), // Dummy span
    })
}

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};

/// Command to split an event at a specific time
#[derive(Debug, Clone)]
pub struct SplitEventCommand {
    pub event_index: usize,
    pub split_time: String, // Time in ASS format (H:MM:SS.CC)
    pub description: Option<String>,
}

impl SplitEventCommand {
    /// Create a new event split command
    pub fn new(event_index: usize, split_time: String) -> Self {
        Self {
            event_index,
            split_time,
            description: None,
        }
    }

    /// Set a custom description for this command
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for SplitEventCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        // Parse split time to validate it
        let split_time_cs = parse_ass_time(&self.split_time).map_err(|_| {
            EditorError::command_failed(format!("Invalid time format: {}", self.split_time))
        })?;

        // Find the event to split
        let content = document.text();
        let events_start = content
            .find("[Events]")
            .ok_or_else(|| EditorError::command_failed("Events section not found"))?;

        // Skip to first event after format line
        let events_content = &content[events_start..];
        let format_line_end = events_content
            .find("Format:")
            .and_then(|format_pos| {
                events_content[format_pos..]
                    .find('\n')
                    .map(|newline_pos| events_start + format_pos + newline_pos + 1)
            })
            .ok_or_else(|| EditorError::command_failed("Invalid events section format"))?;

        // Find the event at the specified index
        let mut current_index = 0;
        let mut event_start = format_line_end;

        while event_start < content.len() {
            let line_end = content[event_start..]
                .find('\n')
                .map(|pos| event_start + pos)
                .unwrap_or(content.len());

            if event_start >= line_end {
                break;
            }

            let line = &content[event_start..line_end];

            // Check if this is an event line
            if line.starts_with("Dialogue:") || line.starts_with("Comment:") {
                if current_index == self.event_index {
                    // Found the event to split - parse using ass-core's parser
                    let event = parse_event_line(line)?;

                    // Validate split time is within event bounds
                    let start_time_cs = event
                        .start_time_cs()
                        .map_err(|_| EditorError::command_failed("Invalid start time in event"))?;
                    let end_time_cs = event
                        .end_time_cs()
                        .map_err(|_| EditorError::command_failed("Invalid end time in event"))?;

                    if split_time_cs <= start_time_cs || split_time_cs >= end_time_cs {
                        return Err(EditorError::command_failed(
                            "Split time must be between event start and end times",
                        ));
                    }

                    // Create two new events
                    let event_type_str = match event.event_type {
                        EventType::Dialogue => "Dialogue",
                        EventType::Comment => "Comment",
                        _ => "Dialogue", // Default fallback
                    };
                    let first_event = format!(
                        "{}: {},{},{},{},{},{},{},{},{},{}",
                        event_type_str,
                        event.layer,
                        event.start,
                        self.split_time,
                        event.style,
                        event.name,
                        event.margin_l,
                        event.margin_r,
                        event.margin_v,
                        event.effect,
                        event.text
                    );

                    let second_event = format!(
                        "{}: {},{},{},{},{},{},{},{},{},{}",
                        event_type_str,
                        event.layer,
                        self.split_time,
                        event.end,
                        event.style,
                        event.name,
                        event.margin_l,
                        event.margin_r,
                        event.margin_v,
                        event.effect,
                        event.text
                    );

                    // Replace the original event with the two new events
                    let replacement = format!("{first_event}\n{second_event}");
                    let range = Range::new(Position::new(event_start), Position::new(line_end));
                    document.replace(range, &replacement)?;

                    let end_pos = Position::new(event_start + replacement.len());
                    return Ok(CommandResult::success_with_change(
                        Range::new(Position::new(event_start), end_pos),
                        end_pos,
                    )
                    .with_message(format!(
                        "Split event {} at time {}",
                        self.event_index, self.split_time
                    )));
                }
                current_index += 1;
            } else if line.starts_with('[') {
                // Stop at next section
                break;
            }

            event_start = line_end + 1;
        }

        Err(EditorError::command_failed(format!(
            "Event index {} not found",
            self.event_index
        )))
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Split event")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.split_time.len()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}

/// Command to merge two consecutive events
#[derive(Debug, Clone)]
pub struct MergeEventsCommand {
    pub first_event_index: usize,
    pub second_event_index: usize,
    pub merge_text_separator: String, // Text to put between merged texts
    pub description: Option<String>,
}

impl MergeEventsCommand {
    /// Create a new event merge command
    pub fn new(first_event_index: usize, second_event_index: usize) -> Self {
        Self {
            first_event_index,
            second_event_index,
            merge_text_separator: " ".to_string(), // Default space separator
            description: None,
        }
    }

    /// Set the text separator for merged text
    pub fn with_separator(mut self, separator: String) -> Self {
        self.merge_text_separator = separator;
        self
    }

    /// Set a custom description for this command
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for MergeEventsCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        if self.first_event_index >= self.second_event_index {
            return Err(EditorError::command_failed(
                "First event index must be less than second event index",
            ));
        }

        let content = document.text();
        let events_start = content
            .find("[Events]")
            .ok_or_else(|| EditorError::command_failed("Events section not found"))?;

        // Skip to first event after format line
        let events_content = &content[events_start..];
        let format_line_end = events_content
            .find("Format:")
            .and_then(|format_pos| {
                events_content[format_pos..]
                    .find('\n')
                    .map(|newline_pos| events_start + format_pos + newline_pos + 1)
            })
            .ok_or_else(|| EditorError::command_failed("Invalid events section format"))?;

        // Find both events
        let mut events = Vec::new();
        let mut current_index = 0;
        let mut event_start = format_line_end;

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
                if current_index == self.first_event_index
                    || current_index == self.second_event_index
                {
                    events.push((current_index, event_start, line_end, line.to_string()));
                }
                current_index += 1;
            } else if line.starts_with('[') {
                break;
            }

            event_start = line_end + 1;
        }

        if events.len() != 2 {
            return Err(EditorError::command_failed(
                "Could not find both events to merge",
            ));
        }

        // Parse both events using ass-core's parser
        let first_event_line = &events[0].3;
        let second_event_line = &events[1].3;

        let first_event = parse_event_line(first_event_line)?;
        let second_event = parse_event_line(second_event_line)?;

        // Use first event's properties, second event's end time, merged text
        let merged_text = format!(
            "{}{}{}",
            first_event.text, self.merge_text_separator, second_event.text
        );

        // Create merged event
        let event_type_str = match first_event.event_type {
            EventType::Dialogue => "Dialogue",
            EventType::Comment => "Comment",
            _ => "Dialogue", // Default fallback
        };
        let merged_event = format!(
            "{}: {},{},{},{},{},{},{},{},{},{}",
            event_type_str,
            first_event.layer,
            first_event.start,
            second_event.end,
            first_event.style,
            first_event.name,
            first_event.margin_l,
            first_event.margin_r,
            first_event.margin_v,
            first_event.effect,
            merged_text
        );

        // Replace both events with the merged one
        let first_start = events[0].1;
        let second_end = events[1].2 + 1; // Include newline
        let range = Range::new(Position::new(first_start), Position::new(second_end));
        let replacement = format!("{merged_event}\n");

        document.replace(range, &replacement)?;

        let end_pos = Position::new(first_start + replacement.len());
        Ok(CommandResult::success_with_change(
            Range::new(Position::new(first_start), end_pos),
            end_pos,
        )
        .with_message(format!(
            "Merged events {} and {}",
            self.first_event_index, self.second_event_index
        )))
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Merge events")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.merge_text_separator.len()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}

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

/// Command to modify event effects
#[derive(Debug, Clone)]
pub struct EventEffectCommand {
    pub event_indices: Vec<usize>,
    pub effect: String,
    pub operation: EffectOperation,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectOperation {
    Set,     // Replace current effect
    Append,  // Add to existing effect
    Prepend, // Add before existing effect
    Clear,   // Remove all effects
}

impl EventEffectCommand {
    /// Create a new effect command
    pub fn new(event_indices: Vec<usize>, effect: String, operation: EffectOperation) -> Self {
        Self {
            event_indices,
            effect,
            operation,
            description: None,
        }
    }

    /// Set effect for specific events
    pub fn set_effect(event_indices: Vec<usize>, effect: String) -> Self {
        Self::new(event_indices, effect, EffectOperation::Set)
    }

    /// Clear effects for specific events  
    pub fn clear_effect(event_indices: Vec<usize>) -> Self {
        Self::new(event_indices, String::new(), EffectOperation::Clear)
    }

    /// Append effect to specific events
    pub fn append_effect(event_indices: Vec<usize>, effect: String) -> Self {
        Self::new(event_indices, effect, EffectOperation::Append)
    }

    /// Set a custom description for this command
    #[must_use]
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
}

impl EditorCommand for EventEffectCommand {
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
                let should_modify =
                    self.event_indices.is_empty() || self.event_indices.contains(&current_index);

                if should_modify {
                    // Parse event line using helper function
                    if let Ok(event) = parse_event_line(line) {
                        let new_effect = match self.operation {
                            EffectOperation::Set => self.effect.clone(),
                            EffectOperation::Clear => String::new(),
                            EffectOperation::Append => {
                                if event.effect.is_empty() {
                                    self.effect.clone()
                                } else {
                                    format!("{} {}", event.effect, self.effect)
                                }
                            }
                            EffectOperation::Prepend => {
                                if event.effect.is_empty() {
                                    self.effect.clone()
                                } else {
                                    format!("{} {}", self.effect, event.effect)
                                }
                            }
                        };

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
                            event.start,
                            event.end,
                            event.style,
                            event.name,
                            event.margin_l,
                            event.margin_r,
                            event.margin_v,
                            new_effect,
                            event.text
                        );

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
                }
                current_index += 1;
            } else if line.starts_with('[') {
                break;
            }

            event_start = line_end + 1;
        }

        if changes_made > 0 {
            let operation_name = match self.operation {
                EffectOperation::Set => "set",
                EffectOperation::Clear => "cleared",
                EffectOperation::Append => "appended",
                EffectOperation::Prepend => "prepended",
            };

            Ok(CommandResult::success_with_change(
                total_range.unwrap_or(Range::new(Position::new(0), Position::new(0))),
                Position::new(content.len()),
            )
            .with_message(format!("Effect {operation_name} for {changes_made} events")))
        } else {
            Ok(CommandResult::success().with_message("No events were modified".to_string()))
        }
    }

    fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("Modify event effect")
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.event_indices.len() * core::mem::size_of::<usize>()
            + self.effect.len()
            + self.description.as_ref().map_or(0, |d| d.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::string::ToString;
    #[cfg(not(feature = "std"))]
    use alloc::vec;
    use crate::core::EditorDocument;
    const TEST_CONTENT: &str = r#"[Script Info]
Title: Event Commands Test

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,Speaker,0,0,0,,First event
Dialogue: 0,0:00:05.00,0:00:10.00,Default,Speaker,0,0,0,,Second event
Comment: 0,0:00:10.00,0:00:15.00,Default,Speaker,0,0,0,,Third event
"#;

    #[test]
    fn test_split_event_command() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = SplitEventCommand::new(0, "0:00:03.00".to_string());
        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);

        // Should now have 4 events total (1 split into 2)
        let events_count = doc
            .text()
            .lines()
            .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
            .count();
        assert_eq!(events_count, 4);

        // Check split times
        assert!(doc.text().contains("0:00:01.00,0:00:03.00"));
        assert!(doc.text().contains("0:00:03.00,0:00:05.00"));
    }

    #[test]
    fn test_merge_events_command() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = MergeEventsCommand::new(0, 1).with_separator(" | ".to_string());
        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);

        // Should now have 2 events total (2 merged into 1)
        let events_count = doc
            .text()
            .lines()
            .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
            .count();
        assert_eq!(events_count, 2);

        // Check merged text and timing
        assert!(doc.text().contains("First event | Second event"));
        assert!(doc.text().contains("0:00:01.00,0:00:10.00")); // Start of first, end of second
    }

    #[test]
    fn test_timing_adjust_command() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Shift all events forward by 2 seconds (200 centiseconds)
        let command = TimingAdjustCommand::all_events(200, 200);
        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);

        // Check that times were adjusted
        assert!(doc.text().contains("0:00:03.00,0:00:07.00")); // First event shifted
        assert!(doc.text().contains("0:00:07.00,0:00:12.00")); // Second event shifted
        assert!(doc.text().contains("0:00:12.00,0:00:17.00")); // Third event shifted
    }

    #[test]
    fn test_toggle_event_type_command() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = ToggleEventTypeCommand::single(0);
        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);

        // First event should now be Comment, others unchanged
        let text = doc.text();
        let lines: Vec<&str> = text.lines().collect();
        let event_lines: Vec<&str> = lines
            .iter()
            .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
            .copied()
            .collect();

        assert_eq!(event_lines.len(), 3);
        assert!(event_lines[0].starts_with("Comment:")); // Was Dialogue, now Comment
        assert!(event_lines[1].starts_with("Dialogue:")); // Unchanged
        assert!(event_lines[2].starts_with("Comment:")); // Unchanged
    }

    #[test]
    fn test_event_effect_command() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        let command = EventEffectCommand::set_effect(vec![0, 1], "Fade(255,0)".to_string());
        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);
        assert!(result.content_changed);

        // Check that effects were set for first two events
        let content = doc.text();
        let lines: Vec<&str> = content.lines().collect();
        let event_lines: Vec<&str> = lines
            .iter()
            .filter(|line| line.starts_with("Dialogue:") || line.starts_with("Comment:"))
            .copied()
            .collect();

        assert!(event_lines[0].contains("Fade(255,0)"));
        assert!(event_lines[1].contains("Fade(255,0)"));
        assert!(!event_lines[2].contains("Fade(255,0)")); // Third event unchanged
    }

    #[test]
    fn test_split_event_invalid_time() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Try to split outside event bounds
        let command = SplitEventCommand::new(0, "0:00:00.50".to_string()); // Before event start
        let result = command.execute(&mut doc);

        assert!(result.is_err());
    }

    #[test]
    fn test_merge_events_invalid_indices() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Try to merge with invalid order
        let command = MergeEventsCommand::new(1, 0); // Second before first
        let result = command.execute(&mut doc);

        assert!(result.is_err());
    }

    #[test]
    fn test_timing_adjust_with_specific_events() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // Adjust only first event
        let command = TimingAdjustCommand::new(vec![0], 100, 100); // +1 second
        let result = command.execute(&mut doc).unwrap();

        assert!(result.success);

        // Only first event should be changed
        assert!(doc.text().contains("0:00:02.00,0:00:06.00")); // First event adjusted
        assert!(doc.text().contains("0:00:05.00,0:00:10.00")); // Second event unchanged
    }

    #[test]
    fn test_effect_operations() {
        let mut doc = EditorDocument::from_content(TEST_CONTENT).unwrap();

        // First set an effect
        let set_cmd = EventEffectCommand::set_effect(vec![0], "Fade(255,0)".to_string());
        set_cmd.execute(&mut doc).unwrap();

        // Then append to it
        let append_cmd = EventEffectCommand::append_effect(vec![0], "Move(100,200)".to_string());
        append_cmd.execute(&mut doc).unwrap();

        // Check that both effects are present
        // println!("Document after append: {}", doc.text());
        assert!(doc.text().contains("Fade(255,0) Move(100,200)"));

        // Clear the effect
        let clear_cmd = EventEffectCommand::clear_effect(vec![0]);
        clear_cmd.execute(&mut doc).unwrap();

        // Check that effect field is empty (has the right number of commas)
        let text = doc.text();
        let lines: Vec<&str> = text.lines().collect();
        let first_event = lines
            .iter()
            .find(|line| line.starts_with("Dialogue:"))
            .unwrap();
        let parts: Vec<&str> = first_event.split(',').collect();
        assert_eq!(parts[8].trim(), ""); // Effect field should be empty
    }
}
