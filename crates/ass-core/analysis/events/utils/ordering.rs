//! Ordering and range filtering for dialogue event collections.
//!
//! Provides stable time-based sorting and temporal range filtering over
//! slices of [`DialogueInfo`], using centisecond timing for comparisons.

use crate::analysis::events::dialogue_info::DialogueInfo;
use alloc::vec::Vec;
use core::cmp::Ordering;

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
