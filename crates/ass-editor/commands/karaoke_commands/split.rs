//! Split existing karaoke timing into separate timed segments.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

/// Split karaoke timing at specific points
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitKaraokeCommand {
    /// Range containing karaoke to split
    pub range: Range,
    /// Character positions to split at (relative to range start)
    pub split_positions: Vec<usize>,
    /// New duration for each split segment
    pub new_duration: Option<u32>,
}

impl SplitKaraokeCommand {
    /// Create a new split karaoke command
    pub fn new(range: Range, split_positions: Vec<usize>) -> Self {
        Self {
            range,
            split_positions,
            new_duration: None,
        }
    }

    /// Set new duration for split segments
    #[must_use]
    pub fn duration(mut self, duration: u32) -> Self {
        self.new_duration = Some(duration);
        self
    }
}

impl EditorCommand for SplitKaraokeCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let original_text = document.text_range(self.range)?;
        let processed_text = self.split_karaoke_text(&original_text)?;

        document.replace_raw(self.range, &processed_text)?;

        let end_pos = Position::new(self.range.start.offset + processed_text.len());
        let range = Range::new(self.range.start, end_pos);

        Ok(CommandResult::success_with_change(range, end_pos))
    }

    fn description(&self) -> &str {
        "Split karaoke timing"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>() + self.split_positions.len() * core::mem::size_of::<usize>()
    }
}

impl SplitKaraokeCommand {
    /// Split karaoke text at specified positions
    fn split_karaoke_text(&self, text: &str) -> Result<String> {
        // For now, return simplified version
        // In practice, this would parse existing karaoke tags and split them
        let mut result = String::new();
        let mut last_pos = 0;

        for &pos in &self.split_positions {
            if pos <= last_pos || pos >= text.len() {
                continue;
            }

            let segment = &text[last_pos..pos];
            if !segment.is_empty() {
                let duration = self.new_duration.unwrap_or(50);
                result.push_str(&format!("{{\\k{duration}}}{segment}"));
            }
            last_pos = pos;
        }

        // Add remaining text
        if last_pos < text.len() {
            let segment = &text[last_pos..];
            if !segment.is_empty() {
                let duration = self.new_duration.unwrap_or(50);
                result.push_str(&format!("{{\\k{duration}}}{segment}"));
            }
        }

        Ok(result)
    }
}
