//! Line processing logic for streaming ASS parser
//!
//! Handles incremental processing of individual lines during streaming parsing,
//! with context-aware processing based on current parser state.

mod line_processor;
mod section_lines;

#[cfg(test)]
mod content_tests;
#[cfg(test)]
mod dispatch_tests;

pub use line_processor::LineProcessor;
