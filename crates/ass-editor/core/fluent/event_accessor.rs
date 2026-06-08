//! Fluent accessor for individual event properties and operations.

use super::event_merge_timing::EventTimer;
use super::event_ops::EventOps;
use super::event_toggle_effect::{EventEffector, EventToggler};
use super::EventInfo;
use crate::core::{EditorDocument, Result};
use ass_core::parser::ast::EventType;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Fluent accessor for individual event properties and operations
pub struct EventAccessor<'a> {
    document: &'a mut EditorDocument,
    index: usize,
}

impl<'a> EventAccessor<'a> {
    pub(crate) fn new(document: &'a mut EditorDocument, index: usize) -> Self {
        Self { document, index }
    }

    /// Get the full event information
    pub fn get(self) -> Result<Option<EventInfo>> {
        EventOps::new(self.document).get(self.index)
    }

    /// Get just the event text
    pub fn text(self) -> Result<Option<String>> {
        Ok(self.get()?.map(|info| info.event.text))
    }

    /// Get event style name
    pub fn style(self) -> Result<Option<String>> {
        Ok(self.get()?.map(|info| info.event.style))
    }

    /// Get event speaker/actor name
    pub fn speaker(self) -> Result<Option<String>> {
        Ok(self.get()?.map(|info| info.event.name))
    }

    /// Get event timing as (start, end) in centiseconds
    pub fn timing(self) -> Result<Option<(String, String)>> {
        Ok(self.get()?.map(|info| (info.event.start, info.event.end)))
    }

    /// Get event start time
    pub fn start_time(self) -> Result<Option<String>> {
        Ok(self.get()?.map(|info| info.event.start))
    }

    /// Get event end time
    pub fn end_time(self) -> Result<Option<String>> {
        Ok(self.get()?.map(|info| info.event.end))
    }

    /// Get event layer
    pub fn layer(self) -> Result<Option<String>> {
        Ok(self.get()?.map(|info| info.event.layer))
    }

    /// Get event effect
    pub fn effect(self) -> Result<Option<String>> {
        Ok(self.get()?.map(|info| info.event.effect))
    }

    /// Get event type (Dialogue/Comment)
    pub fn event_type(self) -> Result<Option<EventType>> {
        Ok(self.get()?.map(|info| info.event.event_type))
    }

    /// Check if event exists
    pub fn exists(self) -> Result<bool> {
        Ok(self.get()?.is_some())
    }

    /// Get event margins as (left, right, vertical)
    pub fn margins(self) -> Result<Option<(String, String, String)>> {
        Ok(self.get()?.map(|info| {
            (
                info.event.margin_l,
                info.event.margin_r,
                info.event.margin_v,
            )
        }))
    }

    /// Convert to timing operations for this specific event
    pub fn timing_ops(self) -> EventTimer<'a> {
        EventTimer::new(self.document).event(self.index)
    }

    /// Convert to toggle operations for this specific event
    pub fn toggle_ops(self) -> EventToggler<'a> {
        EventToggler::new(self.document).event(self.index)
    }

    /// Convert to effect operations for this specific event
    pub fn effect_ops(self) -> EventEffector<'a> {
        EventEffector::new(self.document).event(self.index)
    }
}
