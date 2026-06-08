//! Format import/export functionality for subtitle files.
//!
//! This module provides traits and implementations for importing and exporting
//! various subtitle formats, reusing ass-core's parsing capabilities where possible.

mod registry;
mod traits;
mod types;

pub use registry::FormatRegistry;
pub use traits::{Format, FormatExporter, FormatImporter};
pub use types::{FormatInfo, FormatOptions, FormatResult};

// Individual format modules
pub mod ass;
pub mod srt;
pub mod webvtt;

#[cfg(test)]
mod formats_tests;
