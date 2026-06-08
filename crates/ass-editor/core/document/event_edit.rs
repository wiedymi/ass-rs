//! Index-based structured event editing
//!
//! Implements `edit_event_by_index`, which locates an event in the raw text,
//! applies field-level modifications, and rewrites the dialogue/comment line
//! via the `build_modified_event_line_from_data` helper.

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
    /// Edit event by index with full field support
    ///
    /// Allows structured editing of specific event fields by index.
    /// Returns the modified event line for undo support.
    ///
    /// # Arguments
    ///
    /// * `index` - Zero-based index of the event to edit
    /// * `update_fn` - Function that receives the current event and returns modifications
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
    /// doc.edit_event_by_index(0, |event| {
    ///     vec![
    ///         ("text", "New dialogue text".to_string()),
    ///         ("style", "NewStyle".to_string()),
    ///     ]
    /// })?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn edit_event_by_index<F>(&mut self, index: usize, update_fn: F) -> Result<String>
    where
        F: for<'a> FnOnce(&ass_core::parser::ast::Event<'a>) -> Vec<(&'static str, String)>,
    {
        let content = self.text();
        let mut event_info = None;
        let mut event_count = 0;

        // Find the event and its location in the document
        self.parse_script_with(|script| -> Result<()> {
            for section in script.sections() {
                if let Section::Events(events) = section {
                    for event in events {
                        if event_count == index {
                            // Get the modifications from the update function
                            let modifications = update_fn(event);

                            // Build a pattern to search for this specific event
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
                                let line = content[pos..line_end].to_string();
                                (pos, line_end, line)
                            } else {
                                return Err(EditorError::ValidationError {
                                    message: "Could not find event line in document".to_string(),
                                });
                            };

                            // Store the event data we need instead of cloning
                            let event_data = (
                                event.event_type,
                                event.layer.to_string(),
                                event.start.to_string(),
                                event.end.to_string(),
                                event.style.to_string(),
                                event.name.to_string(),
                                event.margin_l.to_string(),
                                event.margin_r.to_string(),
                                event.margin_v.to_string(),
                                event.effect.to_string(),
                                event.text.to_string(),
                            );
                            event_info = Some((event_data, event_line, modifications));
                            return Ok(());
                        }
                        event_count += 1;
                    }
                }
            }
            Ok(())
        })??;

        if let Some((event_data, (line_start, line_end, original_line), modifications)) = event_info
        {
            // Build the new event line with modifications
            let new_line = self.build_modified_event_line_from_data(
                event_data,
                &original_line,
                modifications,
            )?;

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
