//! Fluent API for ASS tag operations.

use crate::commands::{
    EditorCommand, InsertTagCommand, ParseTagCommand, ParsedTag, RemoveTagCommand,
    ReplaceTagCommand, WrapTagCommand,
};
use crate::core::errors::EditorError;
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

/// Fluent API for ASS tag operations
pub struct TagOps<'a> {
    document: &'a mut EditorDocument,
    range: Option<Range>,
    position: Option<Position>,
}

impl<'a> TagOps<'a> {
    /// Create new tag operations
    pub(super) fn new(document: &'a mut EditorDocument) -> Self {
        Self {
            document,
            range: None,
            position: None,
        }
    }

    /// Set position for tag insertion
    #[must_use]
    pub fn at(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    /// Set range for tag operations
    #[must_use]
    pub fn in_range(mut self, range: Range) -> Self {
        self.range = Some(range);
        self
    }

    /// Insert ASS override tag at position
    pub fn insert(self, tag: &str) -> Result<&'a mut EditorDocument> {
        let position = self
            .position
            .ok_or_else(|| EditorError::command_failed("Position required for tag insertion"))?;

        let command = InsertTagCommand::new(position, tag.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Insert tag without auto-wrapping in {}
    pub fn insert_raw(self, tag: &str) -> Result<&'a mut EditorDocument> {
        let position = self
            .position
            .ok_or_else(|| EditorError::command_failed("Position required for tag insertion"))?;

        let command = InsertTagCommand::new(position, tag.to_string()).no_auto_wrap();
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Remove all tags from range
    pub fn remove_all(self) -> Result<&'a mut EditorDocument> {
        let range = self
            .range
            .ok_or_else(|| EditorError::command_failed("Range required for tag removal"))?;

        let command = RemoveTagCommand::new(range);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Remove specific tag pattern from range
    pub fn remove_pattern(self, pattern: &str) -> Result<&'a mut EditorDocument> {
        let range = self
            .range
            .ok_or_else(|| EditorError::command_failed("Range required for tag removal"))?;

        let command = RemoveTagCommand::new(range).pattern(pattern.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Replace tag pattern with another tag
    pub fn replace(self, find_pattern: &str, replace_with: &str) -> Result<&'a mut EditorDocument> {
        let range = self
            .range
            .ok_or_else(|| EditorError::command_failed("Range required for tag replacement"))?;

        let command =
            ReplaceTagCommand::new(range, find_pattern.to_string(), replace_with.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Replace all occurrences of tag pattern
    pub fn replace_all(
        self,
        find_pattern: &str,
        replace_with: &str,
    ) -> Result<&'a mut EditorDocument> {
        let range = self
            .range
            .ok_or_else(|| EditorError::command_failed("Range required for tag replacement"))?;

        let command =
            ReplaceTagCommand::new(range, find_pattern.to_string(), replace_with.to_string()).all();
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Wrap range with opening and closing tags
    pub fn wrap(self, opening_tag: &str) -> Result<&'a mut EditorDocument> {
        let range = self
            .range
            .ok_or_else(|| EditorError::command_failed("Range required for tag wrapping"))?;

        let command = WrapTagCommand::new(range, opening_tag.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Wrap range with explicit opening and closing tags
    pub fn wrap_with(self, opening_tag: &str, closing_tag: &str) -> Result<&'a mut EditorDocument> {
        let range = self
            .range
            .ok_or_else(|| EditorError::command_failed("Range required for tag wrapping"))?;

        let command = WrapTagCommand::new(range, opening_tag.to_string())
            .closing_tag(closing_tag.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Parse tags from range and return parsed information
    pub fn parse(self) -> Result<Vec<ParsedTag>> {
        let range = self.range.unwrap_or_else(|| {
            Range::new(Position::new(0), Position::new(self.document.text().len()))
        });

        let command = ParseTagCommand::new(range).with_positions();
        let text = self.document.text_range(range)?;
        command.parse_tags_from_text(&text)
    }
}
