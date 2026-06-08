//! Execution and collection logic for [`EventQuery`].

use super::event_merge_timing::EventTimer;
use super::event_toggle_effect::{EventEffector, EventToggler};
use super::{EventInfo, EventQuery, EventSortOptions, OwnedEvent};
use crate::core::errors::EditorError;
use crate::core::{Position, Range, Result};
use ass_core::parser::ast::{Event, Section};
use core::cmp::Ordering;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

impl<'a> EventQuery<'a> {
    // Execution methods
    pub fn execute(self) -> Result<Vec<EventInfo>> {
        let mut results = self.collect_events()?;

        // Apply filters
        results = self.apply_filters(results)?;

        // Apply sorting
        if let Some(ref sort_options) = self.sort_options {
            self.apply_sort(&mut results, sort_options);
        }

        // Apply limit
        if let Some(limit) = self.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Execute and return only the indices
    pub fn indices(self) -> Result<Vec<usize>> {
        Ok(self.execute()?.into_iter().map(|info| info.index).collect())
    }

    /// Execute and return events with their indices as tuples
    pub fn with_indices(self) -> Result<Vec<(usize, OwnedEvent)>> {
        Ok(self
            .execute()?
            .into_iter()
            .map(|info| (info.index, info.event))
            .collect())
    }

    /// Execute and get the first matching event
    pub fn first(self) -> Result<Option<EventInfo>> {
        let mut results = self.limit(1).execute()?;
        Ok(results.pop())
    }

    /// Execute and get count of matching events
    pub fn count(self) -> Result<usize> {
        Ok(self.execute()?.len())
    }

    /// Chain with existing fluent operations
    pub fn timing(self) -> Result<EventTimer<'a>> {
        let _indices: Vec<usize> = self.execute()?.into_iter().map(|info| info.index).collect();
        // Need to create a new EventQuery to get document reference since self is consumed
        // This is a limitation of the current API design
        Err(EditorError::command_failed(
            "Cannot chain timing operations after query execution - use indices() first",
        ))
    }

    pub fn toggle_type(self) -> Result<EventToggler<'a>> {
        let _indices: Vec<usize> = self.execute()?.into_iter().map(|info| info.index).collect();
        Err(EditorError::command_failed(
            "Cannot chain toggle operations after query execution - use indices() first",
        ))
    }

    pub fn effects(self) -> Result<EventEffector<'a>> {
        let _indices: Vec<usize> = self.execute()?.into_iter().map(|info| info.index).collect();
        Err(EditorError::command_failed(
            "Cannot chain effect operations after query execution - use indices() first",
        ))
    }

    // ============================================================================
    // Implementation Details
    // ============================================================================

    fn collect_events(&self) -> Result<Vec<EventInfo>> {
        self.document
            .parse_script_with(|script| -> Result<Vec<EventInfo>> {
                let mut events = Vec::new();
                let mut event_index = 0;

                for section in script.sections() {
                    if let Section::Events(section_events) = section {
                        for event in section_events {
                            // Build EventInfo with position tracking
                            let event_info = EventInfo {
                                index: event_index,
                                event: OwnedEvent::from(event),
                                line_number: self.find_line_number(event)?,
                                range: self.find_event_range(event)?,
                            };
                            events.push(event_info);
                            event_index += 1;
                        }
                    }
                }

                Ok(events)
            })?
    }

    fn apply_filters(&self, events: Vec<EventInfo>) -> Result<Vec<EventInfo>> {
        let mut filtered = Vec::new();

        for event_info in events {
            if self.matches_filter(&event_info)? {
                filtered.push(event_info);
            }
        }

        Ok(filtered)
    }

    fn apply_sort(&self, events: &mut [EventInfo], options: &EventSortOptions) {
        events.sort_by(|a, b| {
            let primary_cmp = self.compare_by_criteria(a, b, &options.criteria);

            match primary_cmp {
                Ordering::Equal => {
                    if let Some(secondary) = &options.secondary {
                        let secondary_cmp = self.compare_by_criteria(a, b, secondary);
                        if options.ascending {
                            secondary_cmp
                        } else {
                            secondary_cmp.reverse()
                        }
                    } else {
                        Ordering::Equal
                    }
                }
                other => {
                    if options.ascending {
                        other
                    } else {
                        other.reverse()
                    }
                }
            }
        });
    }

    fn find_line_number(&self, _event: &Event) -> Result<usize> {
        // For now, return a placeholder. This would need to be implemented
        // by tracking line numbers during parsing or using rope byte-to-line conversion
        Ok(1)
    }

    fn find_event_range(&self, _event: &Event) -> Result<Range> {
        // For now, return a placeholder range. This would need to be implemented
        // by using the event's span information from the parser
        Ok(Range::new(Position::new(0), Position::new(0)))
    }
}
