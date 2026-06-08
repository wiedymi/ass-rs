//! In-place line updates and format mutation.
//!
//! Implements [`Script::update_line_at_offset`], which locates the section and
//! line at a byte offset and replaces it while recording change-tracking
//! deltas, alongside the styles/events format setters.

use alloc::{boxed::Box, vec::Vec};

use crate::parser::ast::Section;
use crate::parser::errors::ParseError;

use super::types::{Change, LineContent};
use super::Script;

impl<'a> Script<'a> {
    /// Update a line in the script at the given byte offset
    ///
    /// Finds the section containing the offset and updates the appropriate line.
    /// Returns the old line content if successful.
    ///
    /// # Arguments
    ///
    /// * `offset` - Byte offset of the line to update
    /// * `new_line` - New line content
    /// * `line_number` - Line number for error reporting
    ///
    /// # Returns
    ///
    /// The old line content if successful, or error if update failed
    ///
    /// # Errors
    ///
    /// Returns error if offset is invalid or line cannot be parsed
    pub fn update_line_at_offset(
        &mut self,
        offset: usize,
        new_line: &'a str,
        line_number: u32,
    ) -> core::result::Result<LineContent<'a>, ParseError> {
        // Find which section contains this offset
        let section_index = self
            .sections
            .iter()
            .position(|s| {
                s.span()
                    .is_some_and(|span| span.start <= offset && offset < span.end)
            })
            .ok_or(ParseError::SectionNotFound)?;

        // Parse the new line to determine its type
        let (_, new_content) = self.parse_line_auto(new_line, line_number)?;

        // Update the appropriate section
        let result = match (&mut self.sections[section_index], new_content.clone()) {
            (Section::Styles(styles), LineContent::Style(new_style)) => {
                // Find the style at this offset
                styles
                    .iter()
                    .position(|s| s.span.start <= offset && offset < s.span.end)
                    .map_or(Err(ParseError::IndexOutOfBounds), |style_index| {
                        let old_style = styles[style_index].clone();
                        styles[style_index] = *new_style;
                        Ok(LineContent::Style(Box::new(old_style)))
                    })
            }
            (Section::Events(events), LineContent::Event(new_event)) => {
                // Find the event at this offset
                events
                    .iter()
                    .position(|e| e.span.start <= offset && offset < e.span.end)
                    .map_or(Err(ParseError::IndexOutOfBounds), |event_index| {
                        let old_event = events[event_index].clone();
                        events[event_index] = *new_event;
                        Ok(LineContent::Event(Box::new(old_event)))
                    })
            }
            (Section::ScriptInfo(info), LineContent::Field(key, value)) => {
                // Find and update the field
                if let Some(field_index) = info.fields.iter().position(|(k, _)| *k == key) {
                    let old_value = info.fields[field_index].1;
                    info.fields[field_index] = (key, value);
                    Ok(LineContent::Field(key, old_value))
                } else {
                    // Add new field if not found
                    info.fields.push((key, value));
                    // Record as addition
                    self.change_tracker.record(Change::Added {
                        offset,
                        content: LineContent::Field(key, value),
                        line_number,
                    });
                    Ok(LineContent::Field(key, ""))
                }
            }
            _ => Err(ParseError::InvalidFieldFormat {
                line: line_number as usize,
            }),
        };

        // Record change if successful
        if let Ok(old_content) = &result {
            if !matches!(old_content, LineContent::Field(_, "")) {
                // This was a modification, not an addition
                self.change_tracker.record(Change::Modified {
                    offset,
                    old_content: old_content.clone(),
                    new_content,
                    line_number,
                });
            }
        }

        result
    }

    /// Update format for styles section
    pub fn set_styles_format(&mut self, format: Vec<&'a str>) {
        self.styles_format = Some(format);
    }

    /// Update format for events section
    pub fn set_events_format(&mut self, format: Vec<&'a str>) {
        self.events_format = Some(format);
    }
}
