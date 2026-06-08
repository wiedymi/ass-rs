//! Full reparse and cache-update routines for [`IncrementalParser`].
//!
//! Implements [`IncrementalParser::full_reparse`], which rebuilds the cached
//! script and computes a delta against the previous parse, plus the private
//! cache update helper that keeps cached content consistent with applied edits.

use super::IncrementalParser;
use crate::core::errors::EditorError;
use crate::core::{Range, Result};
use ass_core::parser::{script::ScriptDeltaOwned, Script};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, string::ToString, vec::Vec};

impl IncrementalParser {
    /// Force a full reparse of the document
    pub fn full_reparse(&mut self, content: &str) -> Result<ScriptDeltaOwned> {
        // Parse the entire document
        let new_script = Script::parse(content).map_err(EditorError::from)?;

        // If we had a previous script, calculate delta
        let delta = if let Some(cached_content) = &self.cached_script {
            let old_script = Script::parse(cached_content).map_err(EditorError::from)?;

            // Calculate sections that changed
            let delta = ass_core::parser::calculate_delta(&old_script, &new_script);

            // Convert to owned format
            let mut owned_delta = ScriptDeltaOwned {
                added: Vec::new(),
                modified: Vec::new(),
                removed: Vec::new(),
                new_issues: new_script.issues().to_vec(),
            };

            // Convert added sections
            for section in delta.added {
                owned_delta.added.push(format!("{section:?}"));
            }

            // Convert modified sections
            for (idx, section) in delta.modified {
                owned_delta.modified.push((idx, format!("{section:?}")));
            }

            // Copy removed indices
            owned_delta.removed = delta.removed;

            owned_delta
        } else {
            // First parse - everything is "added"
            ScriptDeltaOwned {
                added: new_script
                    .sections()
                    .iter()
                    .map(|s| format!("{s:?}"))
                    .collect(),
                modified: Vec::new(),
                removed: Vec::new(),
                new_issues: new_script.issues().to_vec(),
            }
        };

        // Update cache
        self.cached_script = Some(content.to_string());
        self.pending_changes.clear();
        self.bytes_changed = 0;

        Ok(delta)
    }

    /// Update the cached script content with a change
    pub(super) fn update_cached_script(&mut self, range: Range, new_text: &str) -> Result<()> {
        if let Some(cached) = &mut self.cached_script {
            // Validate boundaries
            if range.start.offset > cached.len() || range.end.offset > cached.len() {
                return Err(EditorError::InvalidRange {
                    start: range.start.offset,
                    end: range.end.offset,
                    length: cached.len(),
                });
            }

            // Ensure we're on valid UTF-8 boundaries
            if !cached.is_char_boundary(range.start.offset)
                || !cached.is_char_boundary(range.end.offset)
            {
                return Err(EditorError::command_failed(
                    "Cache update range is not on valid UTF-8 character boundaries",
                ));
            }

            // Build the new content
            let mut result = String::with_capacity(
                cached.len() - (range.end.offset - range.start.offset) + new_text.len(),
            );

            // Copy content before the change
            result.push_str(&cached[..range.start.offset]);

            // Insert new text
            result.push_str(new_text);

            // Copy content after the change
            if range.end.offset < cached.len() {
                result.push_str(&cached[range.end.offset..]);
            }

            *cached = result;
        }

        Ok(())
    }
}
