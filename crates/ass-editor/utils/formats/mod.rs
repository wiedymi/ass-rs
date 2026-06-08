//! Format conversion utilities for importing/exporting subtitles
//!
//! Provides conversion between ASS and other subtitle formats like SRT and WebVTT.
//! Supports both import and export operations with format auto-detection.

mod converter;
mod export_srt;
mod export_webvtt;
mod import_srt;
mod import_webvtt;
mod plain_text;
mod types;

#[cfg(feature = "std")]
mod file_io;

#[cfg(test)]
mod tests;

pub use converter::FormatConverter;
pub use types::{ConversionOptions, FormatOptions, SubtitleFormat};

#[cfg(feature = "std")]
pub use file_io::{export_to_file, import_from_file};
