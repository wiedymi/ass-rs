//! Overlap detection helpers for dialogue event collections.
//!
//! Wraps the efficient sweep-line overlap algorithm for slices of
//! [`DialogueInfo`], returning either the overlapping index pairs or just
//! their count.

use crate::analysis::events::{dialogue_info::DialogueInfo, overlap::find_overlapping_event_refs};
use crate::parser::Event;
use alloc::vec::Vec;

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
    let event_refs: Vec<&Event> = events.iter().map(DialogueInfo::event).collect();
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
