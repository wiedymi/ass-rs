//! Performance issue detection rule for ASS script linting.
//!
//! Detects potential performance issues in subtitle scripts that could
//! impact rendering speed, memory usage, or playback smoothness.

mod rule;

#[cfg(test)]
mod tests;

pub use rule::PerformanceRule;
