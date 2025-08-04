//! Tag management commands for ASS override tags
//!
//! Provides commands for inserting, removing, replacing, wrapping, and parsing
//! ASS override tags like \b1, \i1, \c&H00FF00&, \pos(100,200), etc. with
//! proper validation and nested tag handling.

#![allow(clippy::while_let_on_iterator)]

use super::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, EditorError, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

/// Insert ASS override tag at specified position
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertTagCommand {
    /// Position to insert tag at
    pub position: Position,
    /// Tag to insert (e.g., "\\b1", "\\c&H00FF00&")
    pub tag: String,
    /// Whether to wrap in override brackets {} if not present
    pub auto_wrap: bool,
}

impl InsertTagCommand {
    /// Create a new insert tag command
    pub fn new(position: Position, tag: String) -> Self {
        Self {
            position,
            tag,
            auto_wrap: true,
        }
    }

    /// Disable automatic wrapping in override brackets
    #[must_use]
    pub fn no_auto_wrap(mut self) -> Self {
        self.auto_wrap = false;
        self
    }

    /// Validate and format tag for insertion
    fn format_tag(&self) -> Result<String> {
        let tag = self.tag.trim();

        // Validate tag format - should start with backslash
        if !tag.starts_with('\\') {
            return Err(EditorError::command_failed(format!(
                "ASS override tag must start with backslash: '{tag}'"
            )));
        }

        // Check if already wrapped in override brackets
        if tag.starts_with('{') && tag.ends_with('}') {
            return Ok(tag.to_string());
        }

        // Auto-wrap if enabled
        if self.auto_wrap {
            Ok(format!("{{{tag}}}"))
        } else {
            Ok(tag.to_string())
        }
    }
}

impl EditorCommand for InsertTagCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let formatted_tag = self.format_tag()?;
        document.insert_raw(self.position, &formatted_tag)?;

        let end_pos = Position::new(self.position.offset + formatted_tag.len());
        let range = Range::new(self.position, end_pos);

        Ok(CommandResult::success_with_change(range, end_pos))
    }

    fn description(&self) -> &str {
        "Insert ASS tag"
    }

    fn memory_usage(&self) -> usize {
        core::mem::size_of::<Self>() + self.tag.len()
    }
}

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

/// Parse and extract ASS tags from text range
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseTagCommand {
    /// Range to parse tags from
    pub range: Range,
    /// Whether to include tag positions in results
    pub include_positions: bool,
}

impl ParseTagCommand {
    /// Create a new parse tag command
    pub fn new(range: Range) -> Self {
        Self {
            range,
            include_positions: false,
        }
    }

    /// Include position information in parsed results
    #[must_use]
    pub fn with_positions(mut self) -> Self {
        self.include_positions = true;
        self
    }
}

/// Parsed ASS tag information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTag {
    /// The tag content (e.g., "\\b1", "\\c&H00FF00&")
    pub tag: String,
    /// Position of tag in document (if requested)
    pub position: Option<Position>,
    /// Any parameters for the tag
    pub parameters: Vec<String>,
}

impl EditorCommand for ParseTagCommand {
    fn execute(&self, document: &mut EditorDocument) -> Result<CommandResult> {
        let text = document.text_range(self.range)?;
        let parsed_tags = self.parse_tags_from_text(&text)?;

        // Store results in a way that can be retrieved
        // For now, we'll just return success - in a real implementation,
        // you might want to store results in the command result or document metadata

        Ok(CommandResult::success().with_message(format!("Parsed {} ASS tags", parsed_tags.len())))
    }

    fn description(&self) -> &str {
        "Parse ASS tags from text"
    }
}

impl ParseTagCommand {
    /// Parse tags from text content
    pub fn parse_tags_from_text(&self, text: &str) -> Result<Vec<ParsedTag>> {
        let mut tags = Vec::new();
        let mut chars = text.chars().enumerate().peekable();

        while let Some((pos, ch)) = chars.next() {
            if ch == '{' {
                // Found override block
                let mut override_content = String::new();
                let mut brace_count = 1;
                let start_pos = pos;

                while let Some((_, inner_ch)) = chars.next() {
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

                // Parse tags from override content
                let mut override_tags =
                    self.extract_tags_from_override(&override_content, start_pos)?;
                tags.append(&mut override_tags);
            }
        }

        Ok(tags)
    }

    /// Extract individual tags from override block content using ass-core's parser
    fn extract_tags_from_override(
        &self,
        content: &str,
        base_position: usize,
    ) -> Result<Vec<ParsedTag>> {
        use ass_core::analysis::events::tags::parse_override_block;

        let mut core_tags = Vec::new();
        let mut diagnostics = Vec::new();
        parse_override_block(content, 0, &mut core_tags, &mut diagnostics);

        let mut tags = Vec::new();

        // Convert ass-core tags to ParsedTag format
        for core_tag in &core_tags {
            let args = core_tag.args();

            // Parse parameters and determine tag format based on argument structure
            let mut parameters = Vec::new();
            let tag = if args.starts_with('(') && args.ends_with(')') {
                // Arguments in parentheses - extract to parameters field, tag is just name
                let param_content = &args[1..args.len() - 1];
                if !param_content.is_empty() {
                    parameters = param_content
                        .split(',')
                        .map(|p| p.trim().to_string())
                        .collect();
                }
                format!("\\{}", core_tag.name())
            } else {
                // Arguments not in parentheses - include in tag field
                format!("\\{}{}", core_tag.name(), args)
            };

            let position = if self.include_positions {
                Some(Position::new(
                    self.range.start.offset + base_position + core_tag.position(),
                ))
            } else {
                None
            };

            tags.push(ParsedTag {
                tag,
                position,
                parameters,
            });
        }

        Ok(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EditorDocument;

    #[test]
    fn insert_tag_basic() {
        let mut doc = EditorDocument::from_content("Hello World").unwrap();
        let command = InsertTagCommand::new(Position::new(5), "\\b1".to_string());

        let result = command.execute(&mut doc).unwrap();
        assert!(result.success);
        assert_eq!(doc.text(), "Hello{\\b1} World");
    }

    #[test]
    fn insert_tag_no_auto_wrap() {
        let mut doc = EditorDocument::from_content("Hello World").unwrap();
        let command = InsertTagCommand::new(Position::new(5), "\\b1".to_string()).no_auto_wrap();

        let result = command.execute(&mut doc).unwrap();
        assert!(result.success);
        assert_eq!(doc.text(), "Hello\\b1 World");
    }

    #[test]
    fn remove_tag_all() {
        let mut doc = EditorDocument::from_content("Hello {\\b1}World{\\i1} Test").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));
        let command = RemoveTagCommand::new(range);

        let result = command.execute(&mut doc).unwrap();
        assert!(result.success);
        assert_eq!(doc.text(), "Hello World Test");
    }

    #[test]
    fn remove_tag_specific_pattern() {
        let mut doc = EditorDocument::from_content("Hello {\\b1\\i1}World").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));
        let command = RemoveTagCommand::new(range).pattern("\\b".to_string());

        let result = command.execute(&mut doc).unwrap();
        assert!(result.success);
        assert_eq!(doc.text(), "Hello {\\i1}World");
    }

    #[test]
    fn replace_tag() {
        let mut doc = EditorDocument::from_content("Hello {\\b1}World{\\b1} Test").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));
        let command = ReplaceTagCommand::new(range, "\\b1".to_string(), "\\b0".to_string()).all();

        let result = command.execute(&mut doc).unwrap();
        assert!(result.success);
        assert_eq!(doc.text(), "Hello {\\b0}World{\\b0} Test");
    }

    #[test]
    fn wrap_tag() {
        let mut doc = EditorDocument::from_content("Hello World").unwrap();
        let range = Range::new(Position::new(6), Position::new(11));
        let command = WrapTagCommand::new(range, "\\b1".to_string());

        let result = command.execute(&mut doc).unwrap();
        assert!(result.success);
        assert_eq!(doc.text(), "Hello {\\b1}World{\\b0}");
    }

    #[test]
    fn parse_tags() {
        let mut doc =
            EditorDocument::from_content("Hello {\\b1\\c&H00FF00&\\pos(100,200)}World").unwrap();
        let range = Range::new(Position::new(0), Position::new(doc.text().len()));
        let command = ParseTagCommand::new(range).with_positions();

        let result = command.execute(&mut doc).unwrap();
        assert!(result.success);

        // Test parsing functionality directly
        let text = doc.text();
        let parsed = command.parse_tags_from_text(&text).unwrap();

        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].tag, "\\b1");
        assert_eq!(parsed[1].tag, "\\c&H00FF00&");
        assert_eq!(parsed[2].tag, "\\pos");
        assert_eq!(parsed[2].parameters, vec!["100", "200"]);
    }

    #[test]
    fn tag_validation() {
        let mut doc = EditorDocument::new();

        // Test invalid tag (no backslash)
        let command = InsertTagCommand::new(Position::new(0), "b1".to_string());
        let result = command.execute(&mut doc);
        assert!(result.is_err());
    }
}
