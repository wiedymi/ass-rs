//! Sweep-line event primitives for overlap detection.
//!
//! Defines the [`SweepEvent`] record and its [`SweepEventType`] discriminant
//! together with the ordering rules that drive the sweep-line algorithm in
//! [`super::detection`].

use core::cmp::Ordering;

/// Event type discriminant for sweep-line algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SweepEventType {
    /// Event represents dialogue start time
    Start,
    /// Event represents dialogue end time
    End,
}

/// Sweep-line event for overlap detection algorithm
#[derive(Debug, Clone)]
pub(super) struct SweepEvent {
    /// Time of this sweep event in centiseconds
    pub(super) time: u32,
    /// Type of event (start or end)
    pub(super) event_type: SweepEventType,
    /// Index of the original event in the input vector
    pub(super) event_index: usize,
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
