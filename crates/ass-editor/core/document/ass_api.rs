//! ASS-aware query helpers and simple field/event editing
//!
//! Convenience methods that parse the document on demand to count or inspect
//! sections, find event text, and read or update script-info fields without
//! the caller managing the parser directly.

use super::EditorDocument;
use crate::core::errors::Result;
use crate::core::position::{Position, Range};
use ass_core::parser::ast::Section;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

impl EditorDocument {
    // === ASS-Aware APIs ===

    /// Get number of events without manual parsing
    pub fn events_count(&self) -> Result<usize> {
        self.parse_script_with(|script| {
            let mut count = 0;
            for section in script.sections() {
                if let Section::Events(events) = section {
                    count += events.len();
                }
            }
            count
        })
    }

    /// Get number of styles without manual parsing
    pub fn styles_count(&self) -> Result<usize> {
        self.parse_script_with(|script| {
            let mut count = 0;
            for section in script.sections() {
                if let Section::Styles(styles) = section {
                    count += styles.len();
                }
            }
            count
        })
    }

    /// Get script info field names
    pub fn script_info_fields(&self) -> Result<Vec<String>> {
        self.parse_script_with(|script| {
            let mut fields = Vec::new();
            for section in script.sections() {
                if let Section::ScriptInfo(info) = section {
                    for (key, _) in &info.fields {
                        fields.push(key.to_string());
                    }
                }
            }
            fields
        })
    }

    /// Get number of sections
    pub fn sections_count(&self) -> Result<usize> {
        self.parse_script_with(|script| script.sections().len())
    }

    /// Check if document has events section
    pub fn has_events(&self) -> Result<bool> {
        self.parse_script_with(|script| {
            script
                .sections()
                .iter()
                .any(|section| matches!(section, Section::Events(_)))
        })
    }

    /// Check if document has styles section
    pub fn has_styles(&self) -> Result<bool> {
        self.parse_script_with(|script| {
            script
                .sections()
                .iter()
                .any(|section| matches!(section, Section::Styles(_)))
        })
    }

    /// Get event text by line pattern (simplified search)
    pub fn find_event_text(&self, pattern: &str) -> Result<Vec<String>> {
        self.parse_script_with(|script| {
            let mut matches = Vec::new();
            for section in script.sections() {
                if let Section::Events(events) = section {
                    for event in events {
                        if event.text.contains(pattern) {
                            matches.push(event.text.to_string());
                        }
                    }
                }
            }
            matches
        })
    }

    /// Edit an event by finding and replacing text (simplified ASS-aware editing)
    pub fn edit_event_text(&mut self, old_text: &str, new_text: &str) -> Result<()> {
        let content = self.text();

        if let Some(pos) = content.find(old_text) {
            let range = Range::new(Position::new(pos), Position::new(pos + old_text.len()));
            self.replace(range, new_text)?;
        }

        Ok(())
    }

    /// Get script info field value by key
    pub fn get_script_info_field(&self, key: &str) -> Result<Option<String>> {
        self.parse_script_with(|script| {
            script.sections().iter().find_map(|section| {
                if let Section::ScriptInfo(info) = section {
                    info.fields
                        .iter()
                        .find(|(k, _)| *k == key)
                        .map(|(_, v)| v.to_string())
                } else {
                    None
                }
            })
        })
    }

    /// Set script info field (ASS-aware editing)
    pub fn set_script_info_field(&mut self, key: &str, value: &str) -> Result<()> {
        // Simplified implementation - find and replace the field
        let content = self.text();
        let field_pattern = format!("{key}:");

        if let Some(pos) = content.find(&field_pattern) {
            // Find end of line
            let line_start = pos;
            let line_end = content[pos..].find('\n').map_or(content.len(), |n| pos + n);

            let range = Range::new(Position::new(line_start), Position::new(line_end));

            let new_line = format!("{key}: {value}");
            self.replace(range, &new_line)?;
        }

        Ok(())
    }
}
