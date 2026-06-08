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

mod duration;
mod ordering;
mod overlaps;

#[cfg(test)]
mod tests;

pub use duration::{calculate_average_duration, calculate_total_duration};
pub use ordering::{find_events_in_range, sort_events_by_time};
pub use overlaps::{count_overlapping_dialogue_events, find_overlapping_dialogue_events};
