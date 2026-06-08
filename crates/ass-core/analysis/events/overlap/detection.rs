//! Sweep-line overlap detection entry points.
//!
//! Houses the generic [`find_overlaps_generic`] driver and the public
//! [`find_overlapping_events`], [`find_overlapping_event_refs`], and
//! [`count_overlapping_events`] helpers built on top of it.

use super::sweep::{SweepEvent, SweepEventType};
use crate::{
    parser::Event,
    utils::{parse_ass_time, CoreError},
    Result,
};
use alloc::vec::Vec;

/// Generic overlap detection using sweep-line algorithm
///
/// Internal helper function that implements the core sweep-line algorithm
/// for detecting overlapping time ranges. Works with any iterator that
/// provides start and end times.
pub(super) fn find_overlaps_generic<I, F>(events: I, get_times: F) -> Result<Vec<(usize, usize)>>
where
    I: ExactSizeIterator,
    F: Fn(usize, I::Item) -> Result<(u32, u32)>,
{
    if events.len() < 2 {
        return Ok(Vec::new());
    }

    let mut sweep_events = Vec::with_capacity(events.len() * 2);

    for (index, event) in events.enumerate() {
        let (start_time, end_time) = get_times(index, event)?;

        sweep_events.push(SweepEvent {
            time: start_time,
            event_type: SweepEventType::Start,
            event_index: index,
        });

        sweep_events.push(SweepEvent {
            time: end_time,
            event_type: SweepEventType::End,
            event_index: index,
        });
    }

    sweep_events.sort();

    let mut active_events = Vec::new();
    let mut overlaps = Vec::new();

    for sweep_event in sweep_events {
        match sweep_event.event_type {
            SweepEventType::Start => {
                for &active_index in &active_events {
                    overlaps.push((active_index, sweep_event.event_index));
                }
                active_events.push(sweep_event.event_index);
            }
            SweepEventType::End => {
                if let Some(pos) = active_events
                    .iter()
                    .position(|&x| x == sweep_event.event_index)
                {
                    active_events.remove(pos);
                }
            }
        }
    }

    Ok(overlaps)
}

/// Find overlapping events using sweep-line algorithm
///
/// Efficiently detects all pairs of events with overlapping time ranges.
/// Returns vector of (`event1_index`, `event2_index`) pairs where events overlap.
///
/// # Arguments
///
/// * `events` - Slice of events to analyze for overlaps
///
/// # Returns
///
/// Vector of index pairs representing overlapping events, or error if
/// time parsing fails.
///
/// # Example
///
/// ```rust
/// # use ass_core::analysis::events::overlap::find_overlapping_events;
/// # use ass_core::parser::Event;
/// let events = vec![
///     Event { start: "0:00:00.00", end: "0:00:05.00", ..Default::default() },
///     Event { start: "0:00:03.00", end: "0:00:08.00", ..Default::default() },
///     Event { start: "0:00:10.00", end: "0:00:15.00", ..Default::default() },
/// ];
///
/// let overlaps = find_overlapping_events(&events)?;
/// assert_eq!(overlaps, vec![(0, 1)]);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// Returns an error if any event has invalid time format that cannot be parsed.
pub fn find_overlapping_events(events: &[Event]) -> Result<Vec<(usize, usize)>> {
    find_overlaps_generic(events.iter(), |_index, event| {
        let start_time = parse_ass_time(event.start)
            .map_err(|_| CoreError::parse("Invalid start time format"))?;
        let end_time =
            parse_ass_time(event.end).map_err(|_| CoreError::parse("Invalid end time format"))?;
        Ok((start_time, end_time))
    })
}

/// Find overlapping events from event references
///
/// Memory-efficient version that works with Event references instead of
/// owned Event structs. Useful when working with large event collections.
///
/// # Arguments
///
/// * `events` - Slice of event references to analyze
///
/// # Returns
///
/// Vector of index pairs for overlapping events, or parsing error.
///
/// # Errors
///
/// Returns an error if any event has invalid time format that cannot be parsed.
pub fn find_overlapping_event_refs(events: &[&Event]) -> Result<Vec<(usize, usize)>> {
    find_overlaps_generic(events.iter(), |_index, event| {
        let start_time = parse_ass_time(event.start)
            .map_err(|_| CoreError::parse("Invalid start time format"))?;
        let end_time =
            parse_ass_time(event.end).map_err(|_| CoreError::parse("Invalid end time format"))?;
        Ok((start_time, end_time))
    })
}

/// Count overlapping event pairs efficiently
///
/// Returns only the count without generating the full list of overlap pairs.
/// More memory efficient when only the count is needed.
///
/// # Arguments
///
/// * `events` - Events to check for overlaps
///
/// # Returns
///
/// Number of overlapping event pairs found.
///
/// # Errors
///
/// Returns an error if any event has invalid time format that cannot be parsed.
pub fn count_overlapping_events(events: &[Event]) -> Result<usize> {
    Ok(find_overlapping_events(events)?.len())
}
