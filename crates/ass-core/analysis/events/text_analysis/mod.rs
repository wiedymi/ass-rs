//! Text content analysis for ASS dialogue events
//!
//! Provides comprehensive analysis of dialogue text including override tag parsing,
//! Unicode complexity detection, and character counting. Uses zero-copy design
//! with lifetime-generic references to original text.
//!
//! # Features
//!
//! - Override tag extraction and complexity scoring
//! - Plain text extraction (tags removed)
//! - Unicode bidirectional text detection
//! - Character and line counting
//! - Zero-copy tag argument references
//!
//! # Performance
//!
//! - Target: <0.5ms per event text analysis
//! - Memory: Minimal allocations via string slices
//! - Unicode: Efficient detection without full normalization

mod analysis;
mod helpers;
mod parser;

#[cfg(test)]
mod edge_tests;
#[cfg(test)]
mod override_tests;
#[cfg(test)]
mod parsing_tests;
#[cfg(test)]
mod unicode_tests;

pub use analysis::TextAnalysis;
