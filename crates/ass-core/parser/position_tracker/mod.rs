//! Position tracking utilities for incremental parsing
//!
//! Provides efficient position tracking with line/column information
//! for accurate span generation during parsing.

mod tracker;

#[cfg(test)]
mod tests;

pub use tracker::PositionTracker;
