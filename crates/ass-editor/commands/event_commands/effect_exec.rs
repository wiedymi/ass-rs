//! `EditorCommand` implementation for `EventEffectCommand`.

use super::effect::{EffectOperation, EventEffectCommand};
use super::helpers::parse_event_line;
use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};
use ass_core::parser::ast::EventType;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

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
