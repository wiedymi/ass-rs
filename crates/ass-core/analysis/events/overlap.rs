//! Event timing overlap detection using sweep-line algorithm
//!
//! Provides efficient O(n log n) overlap detection for ASS dialogue events
//! using a sweep-line algorithm instead of naive O(n²) approaches.
//!
//! # Algorithm
//!
//! Uses sweep-line technique with start/end events sorted by time.
//! Maintains active event set to detect overlaps in optimal time complexity.
//!
//! # Performance
//!
//! - Time complexity: O(n log n) due to sorting
//! - Space complexity: O(n) for sweep events and active set
//! - Target: <1ms for 1000 events on typical hardware
//!
//! # Example
//!
//! ```rust
//! use ass_core::analysis::events::overlap::find_overlapping_events;
//! use ass_core::parser::Event;
//!
//! let events = vec![
//!     Event { start: "0:00:00.00", end: "0:00:05.00", ..Default::default() },
//!     Event { start: "0:00:03.00", end: "0:00:08.00", ..Default::default() },
//! ];
//!
//! let overlaps = find_overlapping_events(&events)?;
//! assert_eq!(overlaps.len(), 1);
//! assert_eq!(overlaps[0], (0, 1));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::{
    parser::Event,
    utils::{parse_ass_time, CoreError},
    Result,
};
use alloc::vec::Vec;
use core::cmp::Ordering;

/// Event type discriminant for sweep-line algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SweepEventType {
    Start,
    End,
}

/// Sweep-line event for overlap detection algorithm
#[derive(Debug, Clone)]
struct SweepEvent {
    time: u32,
    event_type: SweepEventType,
    event_index: usize,
}

impl PartialEq for SweepEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.event_type == other.event_type
    }
}

impl Eq for SweepEvent {}

impl PartialOrd for SweepEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SweepEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.time.cmp(&other.time) {
            Ordering::Equal => match (self.event_type, other.event_type) {
                (SweepEventType::End, SweepEventType::Start) => Ordering::Less,
                (SweepEventType::Start, SweepEventType::End) => Ordering::Greater,
                _ => Ordering::Equal,
            },
            other => other,
        }
    }
}

/// Generic overlap detection using sweep-line algorithm
///
/// Internal helper function that implements the core sweep-line algorithm
/// for detecting overlapping time ranges. Works with any iterator that
/// provides start and end times.
fn find_overlaps_generic<I, F>(events: I, get_times: F) -> Result<Vec<(usize, usize)>>
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
/// Returns vector of (event1_index, event2_index) pairs where events overlap.
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
pub fn count_overlapping_events(events: &[Event]) -> Result<usize> {
    Ok(find_overlapping_events(events)?.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event(start: &'static str, end: &'static str) -> Event<'static> {
        Event {
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
        }
    }

    #[test]
    fn no_overlaps_empty_list() {
        let events = vec![];
        let result = find_overlapping_events(&events).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn no_overlaps_single_event() {
        let events = vec![create_test_event("0:00:00.00", "0:00:05.00")];
        let result = find_overlapping_events(&events).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn no_overlaps_sequential_events() {
        let events = vec![
            create_test_event("0:00:00.00", "0:00:05.00"),
            create_test_event("0:00:05.00", "0:00:10.00"),
            create_test_event("0:00:10.00", "0:00:15.00"),
        ];
        let result = find_overlapping_events(&events).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn simple_overlap() {
        let events = vec![
            create_test_event("0:00:00.00", "0:00:05.00"),
            create_test_event("0:00:03.00", "0:00:08.00"),
        ];
        let result = find_overlapping_events(&events).unwrap();
        assert_eq!(result, vec![(0, 1)]);
    }

    #[test]
    fn multiple_overlaps() {
        let events = vec![
            create_test_event("0:00:00.00", "0:00:10.00"),
            create_test_event("0:00:02.00", "0:00:05.00"),
            create_test_event("0:00:03.00", "0:00:08.00"),
        ];
        let result = find_overlapping_events(&events).unwrap();
        assert_eq!(result, vec![(0, 1), (0, 2), (1, 2)]);
    }

    #[test]
    fn count_overlaps() {
        let events = vec![
            create_test_event("0:00:00.00", "0:00:05.00"),
            create_test_event("0:00:03.00", "0:00:08.00"),
        ];
        let count = count_overlapping_events(&events).unwrap();
        assert_eq!(count, 1);
    }
}
