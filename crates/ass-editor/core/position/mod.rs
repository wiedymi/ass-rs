//! Position and range types for document editing
//!
//! Provides types and builders for working with positions and ranges
//! in documents. Supports both byte offsets and line/column positions
//! with efficient conversion between them using the rope data structure.

mod builder;
mod points;
mod range;
mod selection;

#[cfg(test)]
mod tests;

pub use builder::PositionBuilder;
pub use points::{LineColumn, Position};
pub use range::Range;
pub use selection::Selection;
