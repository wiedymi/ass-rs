//! Capturing undo data for script deltas
//!
//! Snapshots the pre-edit text of sections that a delta will remove or
//! modify so the change can later be reversed. Exposed as `pub(super)` for
//! the apply and incremental-edit submodules.

use super::EditorDocument;
use crate::core::errors::{EditorError, Result};
use ass_core::parser::ast::Section;
use ass_core::parser::script::ScriptDeltaOwned;

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::Vec,
};

impl EditorDocument {
    /// Capture undo data before applying a delta
    pub(super) fn capture_delta_undo_data(
        &self,
        delta: &ScriptDeltaOwned,
    ) -> Result<crate::core::history::DeltaUndoData> {
        let mut removed_sections = Vec::new();
        let mut modified_sections = Vec::new();

        // First, get the current content for section extraction
        let content = self.text();

        // For incremental edits, use simplified undo data to avoid expensive full parsing
        // Only do detailed section analysis for larger operations
        #[cfg(feature = "stream")]
        let is_small_edit = delta.added.len() + delta.removed.len() + delta.modified.len() <= 2;
        #[cfg(not(feature = "stream"))]
        let is_small_edit = false;

        if is_small_edit {
            // For small incremental edits, skip expensive undo data capture
            // The basic text-based undo in edit_incremental will handle this
        } else {
            // For larger operations, do full undo data capture
            let _ = self.parse_script_with(|script| {
                // Capture removed sections
                for &index in &delta.removed {
                    if let Some(section) = script.sections().get(index) {
                        match self.extract_section_text(&content, section) {
                            Ok(section_text) => removed_sections.push((index, section_text)),
                            Err(_) => {
                                // If we can't extract the section text, store a placeholder
                                removed_sections.push((index, String::new()));
                            }
                        }
                    }
                }

                // Capture original state of modified sections
                for (index, _) in &delta.modified {
                    if let Some(section) = script.sections().get(*index) {
                        match self.extract_section_text(&content, section) {
                            Ok(section_text) => modified_sections.push((*index, section_text)),
                            Err(_) => {
                                // If we can't extract the section text, store a placeholder
                                modified_sections.push((*index, String::new()));
                            }
                        }
                    }
                }

                Ok::<(), EditorError>(())
            })?;
        }

        Ok(crate::core::history::DeltaUndoData {
            removed_sections,
            modified_sections,
        })
    }

    /// Extract section text from content
    fn extract_section_text(&self, content: &str, section: &Section) -> Result<String> {
        let header = match section {
            Section::ScriptInfo(_) => "[Script Info]",
            Section::Styles(_) => "[V4+ Styles]",
            Section::Events(_) => "[Events]",
            Section::Fonts(_) => "[Fonts]",
            Section::Graphics(_) => "[Graphics]",
        };

        // Find the header in the content
        let start = content
            .find(header)
            .ok_or_else(|| EditorError::SectionNotFound {
                section: header.to_string(),
            })?;

        // Find the next section header or end of document
        let section_headers = [
            "[Script Info]",
            "[V4+ Styles]",
            "[Events]",
            "[Fonts]",
            "[Graphics]",
        ];

        let end = content[start + header.len()..]
            .find(|_c: char| {
                for sh in &section_headers {
                    if content[start + header.len()..].starts_with(sh) {
                        return true;
                    }
                }
                false
            })
            .map(|pos| start + header.len() + pos)
            .unwrap_or(content.len());

        Ok(content[start..end].to_string())
    }
}
