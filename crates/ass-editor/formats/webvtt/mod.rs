//! WebVTT format support with style preservation.
//!
//! This module provides import/export functionality for WebVTT files,
//! with comprehensive style preservation and positioning support.

mod cue;
mod exporter;
mod format;
mod importer;
mod styling;
mod time;

#[cfg(test)]
mod conversion_tests;
#[cfg(test)]
mod roundtrip_tests;

pub use format::WebVttFormat;
