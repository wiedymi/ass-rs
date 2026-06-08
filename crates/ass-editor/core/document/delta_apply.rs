//! Applying script deltas to the document text
//!
//! Records and applies `ScriptDeltaOwned` changes by translating section
//! additions, modifications, and removals into raw text edits, then
//! revalidating the result.

use super::EditorDocument;
use crate::commands::CommandResult;
use crate::core::errors::{EditorError, Result};
use crate::core::position::{Position, Range};
use ass_core::parser::ast::Section;
use ass_core::parser::script::ScriptDeltaOwned;
use ass_core::parser::Script;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

impl EditorDocument {
    /// Apply a script delta and record it with undo data
    pub fn apply_script_delta(&mut self, delta: ScriptDeltaOwned) -> Result<()> {
        use crate::core::history::Operation;

        // Capture undo data before applying
        let undo_data = self.capture_delta_undo_data(&delta)?;

        // Apply delta
        self.apply_script_delta_internal(delta.clone())?;

        // Record with undo data
        let operation = Operation::Delta {
            forward: delta,
            undo_data,
        };

        let result = CommandResult::success();
        self.history
            .record_operation(operation, "Apply delta".to_string(), &result);

        Ok(())
    }

    /// Apply a script delta for efficient incremental parsing (internal)
    fn apply_script_delta_internal(&mut self, delta: ScriptDeltaOwned) -> Result<()> {
        // Parse the current script to get sections
        let current_content = self.text();
        let script = Script::parse(&current_content).map_err(EditorError::from)?;

        // Apply removals first (in reverse order to maintain indices)
        let mut removed_indices = delta.removed.clone();
        removed_indices.sort_by(|a, b| b.cmp(a)); // Sort descending

        for index in removed_indices {
            if index < script.sections().len() {
                // Find the section's text range and remove it
                let section = &script.sections()[index];
                let start_offset = self.find_section_start(section)?;
                let end_offset = self.find_section_end(section)?;

                self.delete_raw(Range::new(
                    Position::new(start_offset),
                    Position::new(end_offset),
                ))?;
            }
        }

        // Apply modifications
        for (index, new_section_text) in delta.modified {
            if index < script.sections().len() {
                // Find the section's text range
                let section = &script.sections()[index];
                let start_offset = self.find_section_start(section)?;
                let end_offset = self.find_section_end(section)?;

                // Replace with new section text
                self.replace_raw(
                    Range::new(Position::new(start_offset), Position::new(end_offset)),
                    &new_section_text,
                )?;
            }
        }

        // Apply additions
        for section_text in delta.added {
            // Add new sections at the end of the document
            let end_pos = Position::new(self.len_bytes());

            // Ensure proper newline before new section
            if self.len_bytes() > 0 && !self.text().ends_with('\n') {
                self.insert_raw(end_pos, "\n")?;
            }

            self.insert_raw(Position::new(self.len_bytes()), &section_text)?;

            // Ensure trailing newline
            if !section_text.ends_with('\n') {
                self.insert_raw(Position::new(self.len_bytes()), "\n")?;
            }
        }

        // Validate the result
        let _ = Script::parse(&self.text()).map_err(EditorError::from)?;

        Ok(())
    }

    /// Find the start offset of a section in the document
    fn find_section_start(&self, section: &Section) -> Result<usize> {
        // Get the section header text
        let header = match section {
            Section::ScriptInfo(_) => "[Script Info]",
            Section::Styles(_) => "[V4+ Styles]",
            Section::Events(_) => "[Events]",
            Section::Fonts(_) => "[Fonts]",
            Section::Graphics(_) => "[Graphics]",
        };

        // Find the header in the document
        if let Some(pos) = self.text().find(header) {
            Ok(pos)
        } else {
            Err(EditorError::SectionNotFound {
                section: header.to_string(),
            })
        }
    }

    /// Find the end offset of a section in the document
    fn find_section_end(&self, section: &Section) -> Result<usize> {
        let start = self.find_section_start(section)?;
        let content = &self.text()[start..];

        // Find the next section header or end of document
        let section_headers = [
            "[Script Info]",
            "[V4+ Styles]",
            "[Events]",
            "[Fonts]",
            "[Graphics]",
        ];

        let mut end_offset = content.len();
        for header in &section_headers {
            if let Some(pos) = content.find(header) {
                if pos > 0 {
                    end_offset = end_offset.min(pos);
                }
            }
        }

        Ok(start + end_offset)
    }
}
