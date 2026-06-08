//! Duration calculations across dialogue event collections.
//!
//! Provides total-span and average-duration helpers for slices of
//! [`DialogueInfo`], operating in a single pass over the centisecond timing.

use crate::analysis::events::dialogue_info::DialogueInfo;

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

    let start = events.iter().map(DialogueInfo::start_time_cs).min()?;
    let end = events.iter().map(DialogueInfo::end_time_cs).max()?;

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

    let total_duration: u32 = events.iter().map(DialogueInfo::duration_cs).sum();
    Some(total_duration / u32::try_from(events.len()).unwrap_or(u32::MAX))
}
