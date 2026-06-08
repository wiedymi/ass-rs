//! Parse delta operations for streaming updates
//!
//! Provides delta tracking for efficient incremental parsing and editor
//! integration. Deltas represent minimal changes between parsing states.

mod batch;
mod parse_delta;

#[cfg(test)]
mod batch_tests;
#[cfg(test)]
mod delta_tests;

pub use batch::DeltaBatch;
pub use parse_delta::ParseDelta;
