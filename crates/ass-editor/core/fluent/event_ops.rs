//! Fluent API builder for event operations.

use super::event_deleter::EventDeleter;
use super::event_merge_timing::{EventMerger, EventTimer};
use super::event_toggle_effect::{EventEffector, EventToggler};
use super::{EventAccessor, EventInfo, EventQuery, EventSortCriteria, OwnedEvent};
use crate::commands::{
    BatchDeleteEventsCommand, DeleteEventCommand, EditorCommand, SplitEventCommand,
};
use crate::core::{EditorDocument, Position, Range, Result};
use ass_core::parser::ast::{Event, EventType, Section};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

/// Fluent API builder for event operations
pub struct EventOps<'a> {
    document: &'a mut EditorDocument,
}

impl<'a> EventOps<'a> {
    /// Create a new event operations builder
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self { document }
    }

    /// Split an event at a specific time
    pub fn split(self, event_index: usize, split_time: &str) -> Result<&'a mut EditorDocument> {
        let command = SplitEventCommand::new(event_index, split_time.to_string());
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Merge two consecutive events
    pub fn merge(self, first_index: usize, second_index: usize) -> EventMerger<'a> {
        EventMerger::new(self.document, first_index, second_index)
    }

    /// Adjust timing for events
    pub fn timing(self) -> EventTimer<'a> {
        EventTimer::new(self.document)
    }

    /// Toggle event types between Dialogue and Comment
    pub fn toggle_type(self) -> EventToggler<'a> {
        EventToggler::new(self.document)
    }

    /// Modify event effects
    pub fn effects(self) -> EventEffector<'a> {
        EventEffector::new(self.document)
    }

    // ============================================================================
    // Direct Event Access Methods (NEW!)
    // ============================================================================

    /// Get event information by index
    pub fn get(self, index: usize) -> Result<Option<EventInfo>> {
        self.document
            .parse_script_with(|script| -> Result<Option<EventInfo>> {
                let mut current_index = 0;

                for section in script.sections() {
                    if let Section::Events(events) = section {
                        for event in events {
                            if current_index == index {
                                let event_info = EventInfo {
                                    index,
                                    event: OwnedEvent::from(event),
                                    line_number: self.find_line_number_for_event(event)?,
                                    range: self.find_range_for_event(event)?,
                                };
                                return Ok(Some(event_info));
                            }
                            current_index += 1;
                        }
                    }
                }

                Ok(None)
            })?
    }

    /// Get event by index with fluent access to properties
    pub fn event(self, index: usize) -> EventAccessor<'a> {
        EventAccessor::new(self.document, index)
    }

    /// Get all events as a vector
    pub fn all(self) -> Result<Vec<EventInfo>> {
        EventQuery::new(self.document).execute()
    }

    /// Get event count
    pub fn count(self) -> Result<usize> {
        self.document.parse_script_with(|script| {
            let mut count = 0;

            for section in script.sections() {
                if let Section::Events(events) = section {
                    count += events.len();
                }
            }

            count
        })
    }

    // ============================================================================
    // Query and Filtering Methods (NEW!)
    // ============================================================================

    /// Start a query chain for filtering and sorting events
    pub fn query(self) -> EventQuery<'a> {
        EventQuery::new(self.document)
    }

    /// Shorthand for common filters
    pub fn dialogues(self) -> EventQuery<'a> {
        EventQuery::new(self.document).filter_by_type(EventType::Dialogue)
    }

    pub fn comments(self) -> EventQuery<'a> {
        EventQuery::new(self.document).filter_by_type(EventType::Comment)
    }

    pub fn in_time_range(self, start_cs: u32, end_cs: u32) -> EventQuery<'a> {
        EventQuery::new(self.document).filter_by_time_range(start_cs, end_cs)
    }

    pub fn with_style(self, pattern: &str) -> EventQuery<'a> {
        EventQuery::new(self.document).filter_by_style(pattern)
    }

    /// Find events by text pattern
    pub fn containing(self, text: &str) -> EventQuery<'a> {
        EventQuery::new(self.document).filter_by_text(text)
    }

    /// Get events in order they appear in document
    pub fn in_order(self) -> EventQuery<'a> {
        EventQuery::new(self.document).sort(EventSortCriteria::Index)
    }

    /// Get events sorted by time
    pub fn by_time(self) -> EventQuery<'a> {
        EventQuery::new(self.document).sort_by_time()
    }

    /// Delete a single event by index
    pub fn delete(self, index: usize) -> Result<&'a mut EditorDocument> {
        let command = DeleteEventCommand::new(index);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Delete multiple events by indices
    pub fn delete_multiple(self, indices: Vec<usize>) -> Result<&'a mut EditorDocument> {
        let command = BatchDeleteEventsCommand::new(indices);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Delete all events matching a query
    pub fn delete_query(self) -> EventDeleter<'a> {
        EventDeleter::new(self.document)
    }

    // ============================================================================
    // Helper Methods
    // ============================================================================

    fn find_line_number_for_event(&self, _event: &Event) -> Result<usize> {
        // For now, return a placeholder. This would need to be implemented
        // by tracking line numbers during parsing or using rope byte-to-line conversion
        Ok(1)
    }

    fn find_range_for_event(&self, _event: &Event) -> Result<Range> {
        // For now, return a placeholder range. This would need to be implemented
        // by using the event's span information from the parser
        Ok(Range::new(Position::new(0), Position::new(0)))
    }
}
