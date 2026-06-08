//! Builder patterns for ASS types
//!
//! Provides fluent builder APIs for creating ASS events, styles, and other structures
//! with ergonomic method chaining and validation.

mod event;
mod event_build;
mod style;
mod style_build;
mod style_format;
mod style_setters;

#[cfg(test)]
mod event_tests;
#[cfg(test)]
mod style_tests;

pub use event::EventBuilder;
pub use style::StyleBuilder;
