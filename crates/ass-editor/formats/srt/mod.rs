//! SRT (SubRip) format support with style preservation.
//!
//! This module provides import/export functionality for SRT files,
//! with comprehensive style preservation through ASS-style tags.

mod export;
mod format;
mod import;
mod styling;
mod time;

#[cfg(test)]
mod conversion_tests;
#[cfg(test)]
mod roundtrip_tests;

pub use format::SrtFormat;
