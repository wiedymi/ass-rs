//! Fluent API builders for merging events and adjusting their timing.

use crate::commands::{EditorCommand, MergeEventsCommand, TimingAdjustCommand};
use crate::core::{EditorDocument, Result};

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

/// Fluent API builder for merging events
pub struct EventMerger<'a> {
    document: &'a mut EditorDocument,
    first_index: usize,
    second_index: usize,
    separator: String,
}

impl<'a> EventMerger<'a> {
    /// Create a new event merger
    pub(crate) fn new(
        document: &'a mut EditorDocument,
        first_index: usize,
        second_index: usize,
    ) -> Self {
        Self {
            document,
            first_index,
            second_index,
            separator: " ".to_string(),
        }
    }

    /// Set the text separator for merged text
    pub fn with_separator(mut self, separator: &str) -> Self {
        self.separator = separator.to_string();
        self
    }

    /// Execute the merge
    pub fn apply(self) -> Result<&'a mut EditorDocument> {
        let command = MergeEventsCommand::new(self.first_index, self.second_index)
            .with_separator(self.separator);
        command.execute(self.document)?;
        Ok(self.document)
    }
}

/// Fluent API builder for timing adjustments
pub struct EventTimer<'a> {
    document: &'a mut EditorDocument,
    event_indices: Vec<usize>,
}

impl<'a> EventTimer<'a> {
    /// Create a new event timer
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self {
            document,
            event_indices: Vec::new(), // Default to all events
        }
    }

    /// Specify which events to adjust
    pub fn events(mut self, indices: Vec<usize>) -> Self {
        self.event_indices = indices;
        self
    }

    /// Adjust a single event
    pub fn event(mut self, index: usize) -> Self {
        self.event_indices = vec![index];
        self
    }

    /// Shift start and end times by the same offset (preserves duration)
    pub fn shift(self, offset_cs: i32) -> Result<&'a mut EditorDocument> {
        let command = TimingAdjustCommand::new(self.event_indices, offset_cs, offset_cs);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Shift only start times (changes duration)
    pub fn shift_start(self, offset_cs: i32) -> Result<&'a mut EditorDocument> {
        let command = TimingAdjustCommand::new(self.event_indices, offset_cs, 0);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Shift only end times (changes duration)
    pub fn shift_end(self, offset_cs: i32) -> Result<&'a mut EditorDocument> {
        let command = TimingAdjustCommand::new(self.event_indices, 0, offset_cs);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Scale duration by a factor
    pub fn scale_duration(self, factor: f64) -> Result<&'a mut EditorDocument> {
        let command = TimingAdjustCommand::scale_duration(self.event_indices, factor);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Custom start and end offsets
    pub fn adjust(
        self,
        start_offset_cs: i32,
        end_offset_cs: i32,
    ) -> Result<&'a mut EditorDocument> {
        let command = TimingAdjustCommand::new(self.event_indices, start_offset_cs, end_offset_cs);
        command.execute(self.document)?;
        Ok(self.document)
    }
}
