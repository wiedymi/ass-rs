//! Error categorization and display utilities for ASS-RS
//!
//! Provides error categorization for filtering, grouping, and user interface
//! organization. Includes suggestion system for common error scenarios to
//! help users resolve issues quickly.

#[cfg(not(feature = "std"))]
extern crate alloc;

mod core_error;
mod error_category;

#[cfg(test)]
mod tests;

pub use error_category::ErrorCategory;
