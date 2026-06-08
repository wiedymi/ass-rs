//! Wrap command for ASS override tags
//!
//! Provides [`WrapTagCommand`] for surrounding a text range with an opening and
//! closing override tag, auto-generating a closing tag when none is supplied.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

/// Wrap text range with ASS override tags
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrapTagCommand {
    /// Range of text to wrap
    pub range: Range,
    /// Opening tag (e.g., "\\b1")
    pub opening_tag: String,
    /// Closing tag (e.g., "\\b0"). If None, uses reset tag
    pub closing_tag: Option<String>,
    /// Whether to merge with existing override blocks
    pub merge_overrides: bool,
}

impl WrapTagCommand {
    /// Create a new wrap tag command
    pub fn new(range: Range, opening_tag: String) -> Self {
        Self {
            range,
            opening_tag,
            closing_tag: None,
            merge_overrides: true,
        }
    }

    /// Set explicit closing tag
    #[must_use]
    pub fn closing_tag(mut self, closing_tag: String) -> Self {
        self.closing_tag = Some(closing_tag);
        self
    }

    /// Don't merge with existing override blocks
    #[must_use]
    pub fn no_merge(mut self) -> Self {
        self.merge_overrides = false;
        self
    }

    /// Generate appropriate closing tag
    fn get_closing_tag(&self) -> String {
        if let Some(ref closing) = self.closing_tag {
            return closing.clone();
        }

        // Auto-generate closing tag based on opening tag
        let tag = self.opening_tag.trim_start_matches('\\');

        if tag.starts_with('b') {
            "\\b0".to_string()
        } else if tag.starts_with('i') {
            "\\i0".to_string()
        } else if tag.starts_with('u') {
            "\\u0".to_string()
        } else if tag.starts_with('s') {
            "\\s0".to_string()
        } else if tag.starts_with("c&") || tag.starts_with("1c&") {
            "\\c".to_string() // Reset to default color
        } else {
            "\\r".to_string() // Generic reset
        }
    }
}

impl EditorCommand for WrapTagCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let closing_tag = self.get_closing_tag();

        // Insert closing tag first (so positions don't shift)
        let closing_formatted = format!("{{{closing_tag}}}");
        document.insert_raw(self.range.end, &closing_formatted)?;

        // Insert opening tag
        let opening_formatted = format!("{{{}}}", self.opening_tag);
        document.insert_raw(self.range.start, &opening_formatted)?;

        let total_length = opening_formatted.len()
            + (self.range.end.offset - self.range.start.offset)
            + closing_formatted.len();
        let end_pos = Position::new(self.range.start.offset + total_length);
        let range = Range::new(self.range.start, end_pos);

        Ok(CommandResult::success_with_change(range, end_pos))
    }

    fn description(&self) -> &str {
        "Wrap text with ASS tags"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.opening_tag.len()
            + self.closing_tag.as_ref().map_or(0, |t| t.len())
    }
}
