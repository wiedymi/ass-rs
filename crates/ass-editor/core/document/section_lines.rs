//! Append/replace helpers for event and style section lines
//!
//! Text-level helpers that insert new dialogue or style lines, creating the
//! relevant section (with a default format line) when it does not yet exist.

use super::EditorDocument;
use crate::core::errors::Result;
use crate::core::position::{Position, Range};

#[cfg(not(feature = "std"))]
use alloc::format;

impl EditorDocument {
    /// Add event line to document
    pub fn add_event_line(&mut self, event_line: &str) -> Result<()> {
        let content = self.text();
        if let Some(events_pos) = content.find("[Events]") {
            // Find end of format line and add after it
            let format_start = content[events_pos..].find("Format:").unwrap_or(0) + events_pos;
            let line_end = content[format_start..].find('\n').unwrap_or(0) + format_start + 1;

            let insert_pos = Position::new(line_end);
            self.insert(insert_pos, &format!("{event_line}\n"))
        } else {
            // Add Events section if it doesn't exist
            let content_len = self.len_bytes();
            let events_section = format!("\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n{event_line}\n");
            self.insert(Position::new(content_len), &events_section)
        }
    }

    /// Edit style line
    pub fn edit_style_line(&mut self, style_name: &str, new_style_line: &str) -> Result<()> {
        let content = self.text();
        let pattern = format!("Style: {style_name},");

        if let Some(pos) = content.find(&pattern) {
            // Find end of line
            let line_end = content[pos..].find('\n').map_or(content.len(), |n| pos + n);
            let range = Range::new(Position::new(pos), Position::new(line_end));
            self.replace(range, new_style_line)
        } else {
            // Add style if it doesn't exist
            self.add_style_line(new_style_line)
        }
    }

    /// Add style line to document
    pub fn add_style_line(&mut self, style_line: &str) -> Result<()> {
        let content = self.text();
        if let Some(styles_pos) = content
            .find("[V4+ Styles]")
            .or_else(|| content.find("[V4 Styles]"))
        {
            // Find end of format line and add after it
            let format_start = content[styles_pos..].find("Format:").unwrap_or(0) + styles_pos;
            let line_end = content[format_start..].find('\n').unwrap_or(0) + format_start + 1;

            let insert_pos = Position::new(line_end);
            self.insert(insert_pos, &format!("{style_line}\n"))
        } else {
            // Add Styles section if it doesn't exist
            let script_info_end = content.find("\n[Events]").unwrap_or(content.len());
            let styles_section = format!("\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n{style_line}\n");
            self.insert(Position::new(script_info_end), &styles_section)
        }
    }
}
