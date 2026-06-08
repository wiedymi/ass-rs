//! Remove command for ASS override tags
//!
//! Provides [`RemoveTagCommand`] for stripping all override tags, or only tags
//! matching a specific pattern, from a range, optionally cleaning up empty
//! override brackets.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// Remove ASS override tags from specified range
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveTagCommand {
    /// Range to remove tags from
    pub range: Range,
    /// Specific tag pattern to remove (e.g., "\\b", "\\c"). If None, removes all tags
    pub tag_pattern: Option<String>,
    /// Whether to remove the override brackets {} if they become empty
    pub clean_empty_overrides: bool,
}

impl RemoveTagCommand {
    /// Create a new remove tag command for all tags in range
    pub fn new(range: Range) -> Self {
        Self {
            range,
            tag_pattern: None,
            clean_empty_overrides: true,
        }
    }

    /// Remove only specific tag pattern
    #[must_use]
    pub fn pattern(mut self, pattern: String) -> Self {
        self.tag_pattern = Some(pattern);
        self
    }

    /// Keep empty override brackets
    #[must_use]
    pub fn keep_empty_overrides(mut self) -> Self {
        self.clean_empty_overrides = false;
        self
    }
}

impl EditorCommand for RemoveTagCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let original_text = document.text_range(self.range)?;
        let cleaned_text = self.remove_tags_from_text(&original_text)?;

        document.replace_raw(self.range, &cleaned_text)?;

        let end_pos = Position::new(self.range.start.offset + cleaned_text.len());
        let range = Range::new(self.range.start, end_pos);

        Ok(CommandResult::success_with_change(range, end_pos))
    }

    fn description(&self) -> &str {
        match &self.tag_pattern {
            Some(_pattern) => "Remove specific ASS tags",
            None => "Remove all ASS tags",
        }
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>() + self.tag_pattern.as_ref().map_or(0, |p| p.len())
    }
}

impl RemoveTagCommand {
    /// Remove tags from text content
    fn remove_tags_from_text(&self, text: &str) -> Result<String> {
        let mut result = String::new();
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                // Found override block - process it
                let mut override_content = String::new();
                let mut brace_count = 1;

                while let Some(inner_ch) = chars.next() {
                    if inner_ch == '{' {
                        brace_count += 1;
                    } else if inner_ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            break;
                        }
                    }
                    override_content.push(inner_ch);
                }

                // Process override content
                let cleaned_override = self.process_override_content(&override_content);

                // Add back if not empty or if we're keeping empty overrides
                if !cleaned_override.is_empty() || !self.clean_empty_overrides {
                    result.push('{');
                    result.push_str(&cleaned_override);
                    result.push('}');
                }
            } else {
                result.push(ch);
            }
        }

        Ok(result)
    }

    /// Process content inside override brackets using ass-core's parser
    fn process_override_content(&self, content: &str) -> String {
        use ass_core::analysis::events::tags::parse_override_block;

        if let Some(pattern) = &self.tag_pattern {
            // Remove only specific pattern using ass-core's parser
            let mut tags = Vec::new();
            let mut diagnostics = Vec::new();
            parse_override_block(content, 0, &mut tags, &mut diagnostics);

            // Rebuild content without matching tags
            let mut result = String::new();
            let mut processed_positions = Vec::new();

            // Process tags from ass-core parser
            for tag in &tags {
                let pattern_without_backslash = &pattern[1..]; // Remove leading backslash from pattern
                if !tag.name().starts_with(pattern_without_backslash) {
                    // This tag should be kept - track its position
                    processed_positions.push((tag.position(), tag.name(), tag.args()));
                }
            }

            // Rebuild the override content, keeping only non-matching tags
            for (_, name, args) in processed_positions {
                result.push('\\');
                result.push_str(name);
                result.push_str(args);
            }

            result
        } else {
            // Remove all tags - return empty string
            String::new()
        }
    }
}
