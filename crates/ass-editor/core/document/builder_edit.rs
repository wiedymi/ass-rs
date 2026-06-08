//! Builder-driven structured event editing
//!
//! Implements `edit_event_with_builder`, which pre-populates an
//! `EventBuilder` from an existing event, applies the caller's fluent
//! modifications, and rewrites the line honoring the section format.

use super::EditorDocument;
use crate::core::errors::{EditorError, Result};
use crate::core::position::{Position, Range};
use ass_core::parser::ast::Section;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

impl EditorDocument {
    /// Edit event using a builder for structured modifications
    ///
    /// Allows editing events using the EventBuilder fluent API. The builder
    /// is pre-populated with the current event's values, allowing selective
    /// field updates.
    ///
    /// # Arguments
    ///
    /// * `index` - Zero-based index of the event to edit
    /// * `builder_fn` - Function that receives a pre-populated EventBuilder
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ass_editor::core::EditorDocument;
    /// # let content = r#"[Script Info]
    /// # Title: Test
    /// #
    /// # [Events]
    /// # Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
    /// # Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Original text"#;
    /// # let mut doc = EditorDocument::from_content(content).unwrap();
    /// # use ass_editor::core::builders::EventBuilder;
    /// doc.edit_event_with_builder(0, |builder| {
    ///     builder
    ///         .text("New dialogue text")
    ///         .style("NewStyle")
    ///         .end_time("0:00:10.00")
    /// })?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn edit_event_with_builder<F>(&mut self, index: usize, builder_fn: F) -> Result<String>
    where
        F: for<'a> FnOnce(
            crate::core::builders::EventBuilder,
        ) -> crate::core::builders::EventBuilder,
    {
        use crate::core::builders::EventBuilder;

        let content = self.text();
        let mut event_info = None;
        let mut event_count = 0;
        let mut format_line = None;

        // Find the event and extract format line
        self.parse_script_with(|script| -> Result<()> {
            for section in script.sections() {
                if let Section::Events(events) = section {
                    // Get format line if available
                    if format_line.is_none() {
                        // Find Events section header and format line in raw text
                        if let Some(events_pos) = content.find("[Events]") {
                            let after_header = &content[events_pos + 8..];
                            if let Some(format_pos) = after_header.find("Format:") {
                                let format_start = events_pos + 8 + format_pos + 7; // Skip "Format:"
                                if let Some(format_end) = content[format_start..].find('\n') {
                                    let format_str =
                                        content[format_start..format_start + format_end].trim();
                                    let fields: Vec<&str> =
                                        format_str.split(',').map(str::trim).collect();
                                    format_line = Some(fields);
                                }
                            }
                        }
                    }

                    for event in events {
                        if event_count == index {
                            // Create a builder pre-populated with current values
                            let mut builder = match event.event_type {
                                ass_core::parser::ast::EventType::Dialogue => {
                                    EventBuilder::dialogue()
                                }
                                ass_core::parser::ast::EventType::Comment => {
                                    EventBuilder::comment()
                                }
                                _ => EventBuilder::new(),
                            };

                            // Pre-populate builder with current event values
                            builder = builder
                                .layer(event.layer.parse::<u32>().unwrap_or(0))
                                .start_time(event.start)
                                .end_time(event.end)
                                .style(event.style)
                                .speaker(event.name)
                                .margin_left(event.margin_l.parse::<u32>().unwrap_or(0))
                                .margin_right(event.margin_r.parse::<u32>().unwrap_or(0))
                                .margin_vertical(event.margin_v.parse::<u32>().unwrap_or(0))
                                .effect(event.effect)
                                .text(event.text);

                            if let Some(margin_t) = event.margin_t {
                                builder = builder.margin_top(margin_t.parse::<u32>().unwrap_or(0));
                            }
                            if let Some(margin_b) = event.margin_b {
                                builder =
                                    builder.margin_bottom(margin_b.parse::<u32>().unwrap_or(0));
                            }

                            // Apply user modifications
                            let modified_builder = builder_fn(builder);

                            // Build the new event line
                            let new_line = if let Some(ref format_fields) = format_line {
                                modified_builder.build_with_format(format_fields)?
                            } else {
                                modified_builder.build()?
                            };

                            // Find the event line in the raw text
                            let event_type_str = event.event_type.as_str();
                            let pattern = format!(
                                "{}: {},{},{}",
                                event_type_str, event.layer, event.start, event.end
                            );

                            let event_line = if let Some(pos) = content.find(&pattern) {
                                let line_end = content[pos..]
                                    .find('\n')
                                    .map(|n| pos + n)
                                    .unwrap_or(content.len());
                                (pos, line_end)
                            } else {
                                return Err(EditorError::ValidationError {
                                    message: "Could not find event line in document".to_string(),
                                });
                            };

                            event_info = Some((event_line, new_line));
                            return Ok(());
                        }
                        event_count += 1;
                    }
                }
            }
            Ok(())
        })??;

        if let Some(((line_start, line_end), new_line)) = event_info {
            // Replace the line in the document
            let range = Range::new(Position::new(line_start), Position::new(line_end));
            self.replace(range, &new_line)?;

            Ok(new_line)
        } else {
            Err(EditorError::InvalidRange {
                start: index,
                end: index + 1,
                length: event_count,
            })
        }
    }
}
