//! Filtering, pattern matching, time parsing, and comparison for [`EventQuery`].
//!
//! [`EventQuery::matches_filter`] and [`EventQuery::compare_by_criteria`] are
//! `pub(super)` so the execution module [`super::event_query_exec`] can invoke
//! them while applying filters and sorting.

use super::{EventInfo, EventQuery, EventSortCriteria};
use crate::core::errors::EditorError;
use crate::core::Result;
use core::cmp::Ordering;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

impl EventQuery<'_> {
    pub(super) fn matches_filter(&self, event_info: &EventInfo) -> Result<bool> {
        // Apply each filter criteria
        if let Some(event_type) = self.filters.event_type {
            if event_info.event.event_type != event_type {
                return Ok(false);
            }
        }

        if let Some(ref pattern) = self.filters.style_pattern {
            if !self.matches_pattern(&event_info.event.style, pattern)? {
                return Ok(false);
            }
        }

        if let Some(ref pattern) = self.filters.text_pattern {
            if !self.matches_pattern(&event_info.event.text, pattern)? {
                return Ok(false);
            }
        }

        if let Some(ref pattern) = self.filters.speaker_pattern {
            if !self.matches_pattern(&event_info.event.name, pattern)? {
                return Ok(false);
            }
        }

        if let Some(ref pattern) = self.filters.effect_pattern {
            if !self.matches_pattern(&event_info.event.effect, pattern)? {
                return Ok(false);
            }
        }

        if let Some(layer) = self.filters.layer {
            if let Ok(event_layer) = event_info.event.layer.parse::<u32>() {
                if event_layer != layer {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        if let Some((start_cs, end_cs)) = self.filters.time_range {
            // Parse timing - this is a simplified implementation
            // In practice, you'd want proper time parsing from ass_core
            if let (Ok(event_start), Ok(event_end)) = (
                self.parse_time_to_cs(&event_info.event.start),
                self.parse_time_to_cs(&event_info.event.end),
            ) {
                if event_start < start_cs || event_end > end_cs {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn matches_pattern(&self, text: &str, pattern: &str) -> Result<bool> {
        if self.filters.use_regex {
            // For now, just do simple string matching
            // In a full implementation, you'd use regex crate
            Ok(if self.filters.case_sensitive {
                text.contains(pattern)
            } else {
                text.to_lowercase().contains(&pattern.to_lowercase())
            })
        } else {
            Ok(if self.filters.case_sensitive {
                text.contains(pattern)
            } else {
                text.to_lowercase().contains(&pattern.to_lowercase())
            })
        }
    }

    fn parse_time_to_cs(&self, time_str: &str) -> Result<u32> {
        // Simplified time parsing - in practice use ass_core utilities
        // Format: H:MM:SS.CS
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 3 {
            return Err(EditorError::command_failed("Invalid time format"));
        }

        let hours: u32 = parts[0]
            .parse()
            .map_err(|_| EditorError::command_failed("Invalid hours"))?;
        let minutes: u32 = parts[1]
            .parse()
            .map_err(|_| EditorError::command_failed("Invalid minutes"))?;

        let sec_cs_parts: Vec<&str> = parts[2].split('.').collect();
        if sec_cs_parts.len() != 2 {
            return Err(EditorError::command_failed("Invalid seconds format"));
        }

        let seconds: u32 = sec_cs_parts[0]
            .parse()
            .map_err(|_| EditorError::command_failed("Invalid seconds"))?;
        let centiseconds: u32 = sec_cs_parts[1]
            .parse()
            .map_err(|_| EditorError::command_failed("Invalid centiseconds"))?;

        Ok(hours * 360000 + minutes * 6000 + seconds * 100 + centiseconds)
    }

    pub(super) fn compare_by_criteria(
        &self,
        a: &EventInfo,
        b: &EventInfo,
        criteria: &EventSortCriteria,
    ) -> Ordering {
        match criteria {
            EventSortCriteria::StartTime => {
                let a_time = self.parse_time_to_cs(&a.event.start).unwrap_or(0);
                let b_time = self.parse_time_to_cs(&b.event.start).unwrap_or(0);
                a_time.cmp(&b_time)
            }
            EventSortCriteria::EndTime => {
                let a_time = self.parse_time_to_cs(&a.event.end).unwrap_or(0);
                let b_time = self.parse_time_to_cs(&b.event.end).unwrap_or(0);
                a_time.cmp(&b_time)
            }
            EventSortCriteria::Duration => {
                let a_start = self.parse_time_to_cs(&a.event.start).unwrap_or(0);
                let a_end = self.parse_time_to_cs(&a.event.end).unwrap_or(0);
                let b_start = self.parse_time_to_cs(&b.event.start).unwrap_or(0);
                let b_end = self.parse_time_to_cs(&b.event.end).unwrap_or(0);
                let a_duration = a_end.saturating_sub(a_start);
                let b_duration = b_end.saturating_sub(b_start);
                a_duration.cmp(&b_duration)
            }
            EventSortCriteria::Style => a.event.style.cmp(&b.event.style),
            EventSortCriteria::Speaker => a.event.name.cmp(&b.event.name),
            EventSortCriteria::Layer => {
                let a_layer = a.event.layer.parse::<u32>().unwrap_or(0);
                let b_layer = b.event.layer.parse::<u32>().unwrap_or(0);
                a_layer.cmp(&b_layer)
            }
            EventSortCriteria::Index => a.index.cmp(&b.index),
            EventSortCriteria::Text => a.event.text.cmp(&b.event.text),
        }
    }
}
