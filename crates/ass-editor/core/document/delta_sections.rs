//! Section-level insert/replace/remove helpers for delta undo/redo
//!
//! Locate sections by header in the raw text and splice them in or out.
//! The insert/replace/remove entry points are `pub(super)` so the undo/redo
//! submodule can reverse delta operations.

use super::EditorDocument;
use crate::core::errors::{EditorError, Result};
use crate::core::position::{Position, Range};
use ass_core::parser::ast::Section;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

impl EditorDocument {
    // === Delta undo/redo helper methods ===

    /// Insert a section at a specific index
    pub(super) fn insert_section_at(&mut self, index: usize, section_text: &str) -> Result<()> {
        // Get the current sections count
        let section_count = self.parse_script_with(|script| script.sections().len())?;

        // If index is beyond current sections, append to end
        if index >= section_count {
            let end_pos = Position::new(self.len_bytes());

            // Ensure proper newline before new section
            if self.len_bytes() > 0 && !self.text().ends_with('\n') {
                self.insert_raw(end_pos, "\n")?;
            }

            self.insert_raw(Position::new(self.len_bytes()), section_text)?;

            // Ensure trailing newline
            if !section_text.ends_with('\n') {
                self.insert_raw(Position::new(self.len_bytes()), "\n")?;
            }

            return Ok(());
        }

        // Find the position where to insert the new section
        let content = self.text();
        let insert_pos = self.parse_script_with(|script| -> Result<usize> {
            if let Some(section) = script.sections().get(index) {
                // Find the start of this section to insert before it
                let header = match section {
                    Section::ScriptInfo(_) => "[Script Info]",
                    Section::Styles(_) => "[V4+ Styles]",
                    Section::Events(_) => "[Events]",
                    Section::Fonts(_) => "[Fonts]",
                    Section::Graphics(_) => "[Graphics]",
                };

                if let Some(pos) = content.find(header) {
                    Ok(pos)
                } else {
                    Err(EditorError::SectionNotFound {
                        section: header.to_string(),
                    })
                }
            } else {
                // Append to end if index is out of bounds
                Ok(content.len())
            }
        })??;

        // Insert the section at the found position
        let mut text_to_insert = section_text.to_string();

        // Ensure section ends with newline
        if !text_to_insert.ends_with('\n') {
            text_to_insert.push('\n');
        }

        // Add extra newline if needed to separate from next section
        if insert_pos < content.len() {
            text_to_insert.push('\n');
        }

        self.insert_raw(Position::new(insert_pos), &text_to_insert)?;

        Ok(())
    }

    /// Replace a section at a specific index
    pub(super) fn replace_section(&mut self, index: usize, new_text: &str) -> Result<()> {
        // Parse to find the section and get its boundaries
        // Get the section to replace
        let content = self.text();
        let section_info: Result<Option<&str>> = self.parse_script_with(|script| {
            if let Some(section) = script.sections().get(index) {
                let header = match section {
                    Section::ScriptInfo(_) => "[Script Info]",
                    Section::Styles(_) => "[V4+ Styles]",
                    Section::Events(_) => "[Events]",
                    Section::Fonts(_) => "[Fonts]",
                    Section::Graphics(_) => "[Graphics]",
                };
                Ok(Some(header))
            } else {
                Ok(None)
            }
        })?;

        if let Some(header) = section_info? {
            let start = self.find_section_start_by_header(&content, header)?;
            let end = self.find_section_end_from_start(&content, start)?;

            self.replace_raw(
                Range::new(Position::new(start), Position::new(end)),
                new_text,
            )?;
        }

        Ok(())
    }

    /// Remove the last section
    pub(super) fn remove_last_section(&mut self) -> Result<()> {
        // Parse to find the last section and get its boundaries
        // Get the last section
        let content = self.text();
        let section_info: Result<Option<&str>> = self.parse_script_with(|script| {
            if let Some(section) = script.sections().last() {
                let header = match section {
                    Section::ScriptInfo(_) => "[Script Info]",
                    Section::Styles(_) => "[V4+ Styles]",
                    Section::Events(_) => "[Events]",
                    Section::Fonts(_) => "[Fonts]",
                    Section::Graphics(_) => "[Graphics]",
                };
                Ok(Some(header))
            } else {
                Ok(None)
            }
        })?;

        if let Some(header) = section_info? {
            let start = self.find_section_start_by_header(&content, header)?;
            let end = self.find_section_end_from_start(&content, start)?;

            self.delete_raw(Range::new(Position::new(start), Position::new(end)))?;
        }

        Ok(())
    }

    /// Find section start by header
    fn find_section_start_by_header(&self, content: &str, header: &str) -> Result<usize> {
        content
            .find(header)
            .ok_or_else(|| EditorError::SectionNotFound {
                section: header.to_string(),
            })
    }

    /// Find section end from start position
    fn find_section_end_from_start(&self, content: &str, start: usize) -> Result<usize> {
        let section_headers = [
            "[Script Info]",
            "[V4+ Styles]",
            "[Events]",
            "[Fonts]",
            "[Graphics]",
        ];

        // Find the next section header after start
        let mut end = content.len();
        for header in &section_headers {
            if let Some(pos) = content[start + 1..].find(header) {
                let actual_pos = start + 1 + pos;
                if actual_pos < end {
                    end = actual_pos;
                }
            }
        }

        Ok(end)
    }
}
