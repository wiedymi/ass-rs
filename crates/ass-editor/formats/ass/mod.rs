//! ASS (Advanced SubStation Alpha) format support.
//!
//! This module provides import/export functionality for ASS files,
//! leveraging ass-core's native parsing and serialization capabilities.

mod format;

#[cfg(test)]
mod format_tests;

#[cfg(test)]
mod format_export_tests;

pub use format::AssFormat;
