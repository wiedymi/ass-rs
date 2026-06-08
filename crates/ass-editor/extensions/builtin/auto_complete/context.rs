//! Cursor-context analysis for auto-completion.
//!
//! Computes the [`CompletionContext`] at a document position and decides when
//! style-name completion applies within event lines.

use super::extension::AutoCompleteExtension;
use super::types::CompletionContext;
use crate::core::{EditorDocument, Position, Result};

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

impl AutoCompleteExtension {
    /// Get completion context at position
    pub(super) fn get_completion_context(
        &self,
        document: &EditorDocument,
        position: Position,
    ) -> Result<CompletionContext> {
        let content = document.text();
        let offset = position.offset;

        // Find current line
        let line_start = content[..offset].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let line_end = content[offset..]
            .find('\n')
            .map(|p| offset + p)
            .unwrap_or(content.len());

        let line = content[line_start..line_end].to_string();
        let column = offset - line_start;

        // Find current section
        let mut current_section = None;
        for line in content[..line_start].lines().rev() {
            if line.starts_with('[') && line.ends_with(']') {
                current_section = Some(line[1..line.len() - 1].to_string());
                break;
            }
        }

        // Check if we're in an override tag
        let before_cursor = &line[..column.min(line.len())];
        let in_override_tag = before_cursor
            .rfind('{')
            .is_some_and(|open| before_cursor[open..].find('}').is_none());

        // Get current tag if in override
        let current_tag = if in_override_tag {
            before_cursor.rfind('{').and_then(|pos| {
                let tag_text = &before_cursor[pos + 1..];
                tag_text.rfind('\\').map(|slash| {
                    let tag_start = &tag_text[slash + 1..];
                    tag_start
                        .find(|c: char| !c.is_alphanumeric())
                        .map(|end| tag_start[..end].to_string())
                        .unwrap_or_else(|| tag_start.to_string())
                })
            })
        } else {
            None
        };

        Ok(CompletionContext {
            line,
            column,
            section: current_section,
            in_override_tag,
            current_tag,
        })
    }

    /// Check if we should complete style names
    pub(super) fn should_complete_style(&self, context: &CompletionContext) -> bool {
        if let Some(ref section) = context.section {
            if section == "Events" {
                // Check if we're in the style field of an event
                let line = context.line.trim_start();
                if line.starts_with("Dialogue:") || line.starts_with("Comment:") {
                    // Count commas to determine field
                    let before_cursor = &context.line[..context.column];
                    let comma_count = before_cursor.matches(',').count();
                    // Style is the 4th field (after 3 commas)
                    return comma_count == 3;
                }
            }
        }
        false
    }
}
