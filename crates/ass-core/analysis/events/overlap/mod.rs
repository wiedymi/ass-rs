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

mod detection;
mod sweep;

#[cfg(test)]
mod detection_tests;
#[cfg(test)]
mod error_tests;
#[cfg(test)]
mod sweep_tests;

pub use detection::{
    count_overlapping_events, find_overlapping_event_refs, find_overlapping_events,
};
