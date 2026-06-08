//! Fluent API for deleting events based on queries.

use super::EventQuery;
use crate::commands::{BatchDeleteEventsCommand, EditorCommand};
use crate::core::{EditorDocument, Result};
use ass_core::parser::ast::EventType;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Fluent API for deleting events based on queries
pub struct EventDeleter<'a> {
    document: &'a mut EditorDocument,
    indices: Vec<usize>,
}

impl<'a> EventDeleter<'a> {
    /// Create a new event deleter
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self {
            document,
            indices: Vec::new(),
        }
    }

    /// Delete events by their indices
    pub fn by_indices(mut self, indices: Vec<usize>) -> Self {
        self.indices = indices;
        self
    }

    /// Delete all dialogue events
    pub fn dialogues(self) -> Result<&'a mut EditorDocument> {
        let indices = EventQuery::new(self.document)
            .filter_by_type(EventType::Dialogue)
            .indices()?;
        let command = BatchDeleteEventsCommand::new(indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Delete all comment events
    pub fn comments(self) -> Result<&'a mut EditorDocument> {
        let indices = EventQuery::new(self.document)
            .filter_by_type(EventType::Comment)
            .indices()?;
        let command = BatchDeleteEventsCommand::new(indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Delete events in a time range
    pub fn in_time_range(self, start_cs: u32, end_cs: u32) -> Result<&'a mut EditorDocument> {
        let indices = EventQuery::new(self.document)
            .filter_by_time_range(start_cs, end_cs)
            .indices()?;
        let command = BatchDeleteEventsCommand::new(indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Delete events with a specific style
    pub fn with_style(self, style: &str) -> Result<&'a mut EditorDocument> {
        let indices = EventQuery::new(self.document)
            .filter_by_style(style)
            .indices()?;
        let command = BatchDeleteEventsCommand::new(indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Delete events containing specific text
    pub fn containing(self, text: &str) -> Result<&'a mut EditorDocument> {
        let indices = EventQuery::new(self.document)
            .filter_by_text(text)
            .indices()?;
        let command = BatchDeleteEventsCommand::new(indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Delete all events
    pub fn all(self) -> Result<&'a mut EditorDocument> {
        let indices = EventQuery::new(self.document).indices()?;
        let command = BatchDeleteEventsCommand::new(indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Execute deletion with the configured indices
    pub fn execute(self) -> Result<&'a mut EditorDocument> {
        if self.indices.is_empty() {
            return Ok(self.document);
        }
        let command = BatchDeleteEventsCommand::new(self.indices);
        command.execute(self.document)?;
        Ok(self.document)
    }
}
