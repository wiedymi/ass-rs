//! Parse command for ASS override tags
//!
//! Provides [`ParseTagCommand`] for extracting override tags from a range into
//! [`ParsedTag`] records, optionally including their positions.

use crate::commands::{CommandResult, EditorCommand};
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

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
