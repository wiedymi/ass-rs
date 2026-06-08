//! Event query builder: filter and sort configuration.
//!
//! The [`EventQuery`] fields are `pub(super)` so the sibling execution
//! ([`super::event_query_exec`]) and filtering ([`super::event_query_filter`])
//! modules can read the configured filters, sort options, and document handle.

use super::{EventFilter, EventSortCriteria, EventSortOptions};
use crate::core::EditorDocument;
use ass_core::parser::ast::EventType;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

/// Main query builder for filtering and sorting events
pub struct EventQuery<'a> {
    pub(super) document: &'a mut EditorDocument,
    pub(super) filters: EventFilter,
    pub(super) sort_options: Option<EventSortOptions>,
    pub(super) limit: Option<usize>,
}

impl<'a> EventQuery<'a> {
    pub(crate) fn new(document: &'a mut EditorDocument) -> Self {
        Self {
            document,
            filters: EventFilter::default(),
            sort_options: None,
            limit: None,
        }
    }

    // Filter methods
    pub fn filter(mut self, filter: EventFilter) -> Self {
        self.filters = filter;
        self
    }

    pub fn filter_by_type(mut self, event_type: EventType) -> Self {
        self.filters.event_type = Some(event_type);
        self
    }

    pub fn filter_by_style(mut self, pattern: &str) -> Self {
        self.filters.style_pattern = Some(pattern.to_string());
        self
    }

    pub fn filter_by_speaker(mut self, pattern: &str) -> Self {
        self.filters.speaker_pattern = Some(pattern.to_string());
        self
    }

    pub fn filter_by_text(mut self, pattern: &str) -> Self {
        self.filters.text_pattern = Some(pattern.to_string());
        self
    }

    pub fn filter_by_time_range(mut self, start_cs: u32, end_cs: u32) -> Self {
        self.filters.time_range = Some((start_cs, end_cs));
        self
    }

    pub fn filter_by_layer(mut self, layer: u32) -> Self {
        self.filters.layer = Some(layer);
        self
    }

    pub fn filter_by_effect(mut self, pattern: &str) -> Self {
        self.filters.effect_pattern = Some(pattern.to_string());
        self
    }

    pub fn with_regex(mut self, use_regex: bool) -> Self {
        self.filters.use_regex = use_regex;
        self
    }

    pub fn case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.filters.case_sensitive = case_sensitive;
        self
    }

    // Sort methods
    pub fn sort(mut self, criteria: EventSortCriteria) -> Self {
        self.sort_options = Some(EventSortOptions {
            criteria,
            secondary: None,
            ascending: true,
        });
        self
    }

    pub fn sort_by(mut self, options: EventSortOptions) -> Self {
        self.sort_options = Some(options);
        self
    }

    pub fn sort_by_time(self) -> Self {
        self.sort(EventSortCriteria::StartTime)
    }

    pub fn sort_by_style(self) -> Self {
        self.sort(EventSortCriteria::Style)
    }

    pub fn sort_by_duration(self) -> Self {
        self.sort(EventSortCriteria::Duration)
    }

    pub fn descending(mut self) -> Self {
        if let Some(ref mut options) = self.sort_options {
            options.ascending = false;
        }
        self
    }

    pub fn then_by(mut self, criteria: EventSortCriteria) -> Self {
        if let Some(ref mut options) = self.sort_options {
            options.secondary = Some(criteria);
        }
        self
    }

    // Limit results
    pub fn limit(mut self, count: usize) -> Self {
        self.limit = Some(count);
        self
    }

    pub fn take(self, count: usize) -> Self {
        self.limit(count)
    }
}
