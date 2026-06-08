//! Replace command for ASS override tags
//!
//! Provides [`ReplaceTagCommand`] for finding tags matching a pattern within a
//! range and replacing them with another tag, either once or for all matches.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

/// Replace ASS override tag with another tag
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceTagCommand {
    /// Range to search for tags to replace
    pub range: Range,
    /// Tag pattern to find (e.g., "\\b1")
    pub find_pattern: String,
    /// Tag to replace with (e.g., "\\b0")
    pub replace_with: String,
    /// Whether to replace all occurrences or just the first
    pub replace_all: bool,
}

impl ReplaceTagCommand {
    /// Create a new replace tag command
    pub fn new(range: Range, find_pattern: String, replace_with: String) -> Self {
        Self {
            range,
            find_pattern,
            replace_with,
            replace_all: false,
        }
    }

    /// Replace all occurrences instead of just the first
    #[must_use]
    pub fn all(mut self) -> Self {
        self.replace_all = true;
        self
    }
}

impl EditorCommand for ReplaceTagCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let original_text = document.text_range(self.range)?;
        let replaced_text = self.replace_tags_in_text(&original_text)?;

        document.replace_raw(self.range, &replaced_text)?;

        let end_pos = Position::new(self.range.start.offset + replaced_text.len());
        let range = Range::new(self.range.start, end_pos);

        Ok(CommandResult::success_with_change(range, end_pos))
    }

    fn description(&self) -> &str {
        "Replace ASS tags"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>() + self.find_pattern.len() + self.replace_with.len()
    }
}

impl ReplaceTagCommand {
    /// Replace tags in text content
    fn replace_tags_in_text(&self, text: &str) -> Result<String> {
        let mut result = String::new();
        let mut chars = text.chars().peekable();
        let mut replacements_made = 0;

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Found override block
                result.push(ch);
                let mut override_content = String::new();
                let mut brace_count = 1;

                while let Some(inner_ch) = chars.next() {
                    if inner_ch == '{' {
                        brace_count += 1;
                    } else if inner_ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            // Process override content before closing
                            let processed = self.process_override_for_replacement(
                                &override_content,
                                &mut replacements_made,
                            );
                            result.push_str(&processed);
                            result.push(inner_ch);
                            break;
                        }
                    }
                    override_content.push(inner_ch);
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    /// Process override content for tag replacement using ass-core's parser
    fn process_override_for_replacement(
        &self,
        content: &str,
        replacements_made: &mut usize,
    ) -> String {
        use ass_core::analysis::events::tags::parse_override_block;

        if !self.replace_all && *replacements_made > 0 {
            return content.to_string();
        }

        let mut tags = Vec::new();
        let mut diagnostics = Vec::new();
        parse_override_block(content, 0, &mut tags, &mut diagnostics);

        let mut result = String::new();

        // Process tags from ass-core parser
        for tag in &tags {
            // Reconstruct the full tag to match against the find pattern
            let tag_full = format!("\\{}{}", tag.name(), tag.args());

            // Check if this tag matches our find pattern
            if tag_full.starts_with(&self.find_pattern)
                && (self.replace_all || *replacements_made == 0)
            {
                // Replace the tag - preserve any suffix
                let suffix = &tag_full[self.find_pattern.len()..];
                result.push_str(&self.replace_with);
                result.push_str(suffix);
                *replacements_made += 1;
            } else {
                // Add the original tag back
                result.push_str(&tag_full);
            }
        }

        result
    }
}
