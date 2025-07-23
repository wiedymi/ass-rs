//! Utility functions for dialogue event analysis
//!
//! Provides common operations for collections of dialogue events including
//! sorting, duration calculations, and overlap detection. Optimized for
//! performance with large event collections.
//!
//! # Features
//!
//! - Efficient sorting by timing with stable ordering
//! - Total duration calculation across event collections
//! - Overlap detection delegation to efficient algorithms
//! - Zero-allocation operations where possible
//!
//! # Performance
//!
//! - Sorting: O(n log n) with optimized comparison
//! - Duration: O(n) single pass with min/max tracking
//! - Overlap detection: O(n log n) via sweep-line algorithm

use crate::analysis::events::{dialogue_info::DialogueInfo, overlap::find_overlapping_event_refs};
use crate::parser::Event;
use alloc::vec::Vec;
use core::cmp::Ordering;

/// Find overlapping dialogue events using efficient timing analysis
///
/// Returns pairs of event indices that have overlapping timing.
/// Delegates to the efficient O(n log n) sweep-line algorithm for optimal performance.
///
/// # Arguments
///
/// * `events` - Slice of `DialogueInfo` to analyze for overlaps
///
/// # Returns
///
/// Vector of (index1, index2) pairs representing overlapping events.
///
/// # Example
///
/// ```rust
/// # use ass_core::analysis::events::utils::find_overlapping_dialogue_events;
/// # use ass_core::analysis::events::dialogue_info::DialogueInfo;
/// # use ass_core::parser::Event;
/// let event1 = Event { start: "0:00:00.00", end: "0:00:05.00", ..Default::default() };
/// let event2 = Event { start: "0:00:03.00", end: "0:00:08.00", ..Default::default() };
/// let events = vec![
///     DialogueInfo::analyze(&event1)?,
///     DialogueInfo::analyze(&event2)?,
/// ];
///
/// let overlaps = find_overlapping_dialogue_events(&events);
/// assert_eq!(overlaps.len(), 1);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn find_overlapping_dialogue_events(events: &[DialogueInfo<'_>]) -> Vec<(usize, usize)> {
    let event_refs: Vec<&Event> = events
        .iter()
        .map(super::dialogue_info::DialogueInfo::event)
        .collect();
    find_overlapping_event_refs(&event_refs).unwrap_or_else(|_| Vec::new())
}

/// Count overlapping dialogue events efficiently
///
/// Convenience wrapper that returns only the count of overlapping pairs.
/// More memory efficient when the specific overlap pairs aren't needed.
///
/// # Arguments
///
/// * `events` - Slice of `DialogueInfo` to check for overlaps
///
/// # Returns
///
/// Number of overlapping event pairs found.
#[must_use]
pub fn count_overlapping_dialogue_events(events: &[DialogueInfo<'_>]) -> usize {
    find_overlapping_dialogue_events(events).len()
}

/// Sort events by timing with stable ordering
///
/// Sorts events by start time first, then by end time for events that
/// start simultaneously. Uses stable sort to preserve relative order
/// of equal elements.
///
/// # Arguments
///
/// * `events` - Mutable slice of `DialogueInfo` to sort in-place
///
/// # Performance
///
/// O(n log n) time complexity with minimal allocations.
/// Comparison operations are optimized for centisecond timing.
///
/// # Example
///
/// ```rust
/// # use ass_core::analysis::events::utils::sort_events_by_time;
/// # use ass_core::analysis::events::dialogue_info::DialogueInfo;
/// # use ass_core::parser::Event;
/// let event1 = Event {
///     start: "0:00:05.00",
///     end: "0:00:10.00",
///     ..Default::default()
/// };
/// let event2 = Event {
///     start: "0:00:01.00",
///     end: "0:00:06.00",
///     ..Default::default()
/// };
/// let dialogue_info1 = DialogueInfo::analyze(&event1)?;
/// let dialogue_info2 = DialogueInfo::analyze(&event2)?;
/// let mut events = vec![dialogue_info1, dialogue_info2];
/// sort_events_by_time(&mut events);
/// // Events are now sorted by start time, then end time
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn sort_events_by_time(events: &mut [DialogueInfo<'_>]) {
    events.sort_by(|a, b| match a.start_time_cs().cmp(&b.start_time_cs()) {
        Ordering::Equal => a.end_time_cs().cmp(&b.end_time_cs()),
        other => other,
    });
}

/// Calculate total duration spanning all events
///
/// Computes the duration from the earliest start time to the latest
/// end time across all events. Returns None for empty collections.
///
/// # Arguments
///
/// * `events` - Slice of `DialogueInfo` to analyze
///
/// # Returns
///
/// Total duration in centiseconds, or None if no events provided.
///
/// # Performance
///
/// Single O(n) pass with iterator optimizations for min/max operations.
///
/// # Example
///
/// ```rust
/// # use ass_core::analysis::events::utils::calculate_total_duration;
/// # use ass_core::analysis::events::dialogue_info::DialogueInfo;
/// # use ass_core::parser::Event;
/// let event1 = Event {
///     start: "0:00:01.00",
///     end: "0:00:05.00",
///     ..Default::default()
/// };
/// let event2 = Event {
///     start: "0:00:03.00",
///     end: "0:00:08.00",
///     ..Default::default()
/// };
/// let dialogue_info1 = DialogueInfo::analyze(&event1)?;
/// let dialogue_info2 = DialogueInfo::analyze(&event2)?;
/// let events = vec![dialogue_info1, dialogue_info2];
/// if let Some(duration) = calculate_total_duration(&events) {
///     println!("Total span: {}ms", duration);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[must_use]
pub fn calculate_total_duration(events: &[DialogueInfo<'_>]) -> Option<u32> {
    if events.is_empty() {
        return None;
    }

    let start = events
        .iter()
        .map(super::dialogue_info::DialogueInfo::start_time_cs)
        .min()?;
    let end = events
        .iter()
        .map(super::dialogue_info::DialogueInfo::end_time_cs)
        .max()?;

    Some(end - start)
}

/// Calculate average event duration
///
/// Computes the mean duration of all events in the collection.
/// Returns None for empty collections.
///
/// # Arguments
///
/// * `events` - Slice of `DialogueInfo` to analyze
///
/// # Returns
///
/// Average duration in centiseconds, or None if no events.
#[must_use]
pub fn calculate_average_duration(events: &[DialogueInfo<'_>]) -> Option<u32> {
    if events.is_empty() {
        return None;
    }

    let total_duration: u32 = events
        .iter()
        .map(super::dialogue_info::DialogueInfo::duration_cs)
        .sum();
    Some(total_duration / events.len() as u32)
}

/// Find events within a specific time range
///
/// Returns indices of events that overlap with the given time range.
/// Useful for temporal filtering and range-based analysis.
///
/// # Arguments
///
/// * `events` - Slice of `DialogueInfo` to search
/// * `start_cs` - Range start time in centiseconds
/// * `end_cs` - Range end time in centiseconds
///
/// # Returns
///
/// Vector of indices for events overlapping the time range.
#[must_use]
pub fn find_events_in_range(events: &[DialogueInfo<'_>], start_cs: u32, end_cs: u32) -> Vec<usize> {
    events
        .iter()
        .enumerate()
        .filter_map(|(idx, event)| {
            if event.overlaps_time_range(start_cs, end_cs) {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Event;

    fn create_test_dialogue_info(start: &'static str, end: &'static str) -> DialogueInfo<'static> {
        // Create a static event for the lifetime requirement
        // Using Box::leak is acceptable in tests for simplicity
        let event = Box::leak(Box::new(Event {
            event_type: crate::parser::ast::EventType::Dialogue,
            start,
            end,
            text: "Test",
            layer: "0",
            style: "Default",
            name: "",
            margin_l: "0",
            margin_r: "0",
            margin_v: "0",
            effect: "",
        }));
        DialogueInfo::analyze(event).unwrap()
    }

    #[test]
    fn empty_events_no_overlaps() {
        let events = vec![];
        assert_eq!(find_overlapping_dialogue_events(&events).len(), 0);
        assert_eq!(count_overlapping_dialogue_events(&events), 0);
    }

    #[test]
    fn calculate_duration_empty() {
        let events = vec![];
        assert_eq!(calculate_total_duration(&events), None);
        assert_eq!(calculate_average_duration(&events), None);
    }

    #[test]
    fn calculate_duration_single_event() {
        let events = vec![create_test_dialogue_info("0:00:00.00", "0:00:05.00")];
        assert_eq!(calculate_total_duration(&events), Some(500)); // 5 seconds = 500cs
        assert_eq!(calculate_average_duration(&events), Some(500));
    }

    #[test]
    fn calculate_duration_multiple_events() {
        let events = vec![
            create_test_dialogue_info("0:00:00.00", "0:00:05.00"),
            create_test_dialogue_info("0:00:03.00", "0:00:08.00"),
            create_test_dialogue_info("0:00:10.00", "0:00:15.00"),
        ];

        // Total: 0:00:00.00 to 0:00:15.00 = 1500cs
        assert_eq!(calculate_total_duration(&events), Some(1500));

        // Average: (500 + 500 + 500) / 3 = 500cs
        assert_eq!(calculate_average_duration(&events), Some(500));
    }

    #[test]
    fn sort_events_maintains_order() {
        let mut events = vec![
            create_test_dialogue_info("0:00:05.00", "0:00:10.00"),
            create_test_dialogue_info("0:00:00.00", "0:00:05.00"),
            create_test_dialogue_info("0:00:02.00", "0:00:07.00"),
        ];

        sort_events_by_time(&mut events);

        assert_eq!(events[0].start_time_cs(), 0); // 0:00:00.00
        assert_eq!(events[1].start_time_cs(), 200); // 0:00:02.00
        assert_eq!(events[2].start_time_cs(), 500); // 0:00:05.00
    }

    #[test]
    fn find_events_in_range_filters_correctly() {
        let events = vec![
            create_test_dialogue_info("0:00:00.00", "0:00:05.00"), // 0-500cs
            create_test_dialogue_info("0:00:03.00", "0:00:08.00"), // 300-800cs
            create_test_dialogue_info("0:00:10.00", "0:00:15.00"), // 1000-1500cs
        ];

        let indices = find_events_in_range(&events, 250, 600); // 2.5s to 6s
        assert_eq!(indices, vec![0, 1]); // First two events overlap this range
    }
}
