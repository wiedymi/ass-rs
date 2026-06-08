//! Streaming and incremental parsing for ASS scripts
//!
//! Provides efficient streaming parsing capabilities with true incremental
//! processing through state machine design. Enables <5ms responsiveness
//! for large files and editor integration.
//!
//! # Features
//!
//! - True streaming: Process chunks without loading entire file
//! - State machine: Handle partial lines and incomplete sections
//! - Delta tracking: Efficient change representation for editors
//! - Memory efficiency: O(line) not O(file) memory usage
//!
//! # Performance
//!
//! - Target: <5ms per 1MB chunk processing
//! - Memory: <1.1x input size peak usage
//! - Incremental: <2ms for single-event edits
//! - Supports files up to 2GB on 64-bit systems
//!
//! # Example
//!
//! ```rust
//! use ass_core::parser::streaming::StreamingParser;
//!
//! let mut parser = StreamingParser::new();
//!
//! // Process chunks incrementally
//! let chunk1 = b"[Script Info]\nTitle: Example\n";
//! let deltas1 = parser.feed_chunk(chunk1)?;
//!
//! let chunk2 = b"[Events]\nFormat: Layer, Start, End\n";
//! let deltas2 = parser.feed_chunk(chunk2)?;
//!
//! let result = parser.finish()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#[cfg(not(feature = "std"))]
extern crate alloc;

mod delta;
mod parser;
mod processor;
mod result;
mod source;
mod state;

#[cfg(test)]
mod feed_tests;
#[cfg(test)]
mod parser_tests;
#[cfg(test)]
mod result_tests;

// Re-export public API
pub use delta::{DeltaBatch, ParseDelta};
pub use parser::StreamingParser;
pub use processor::LineProcessor;
pub use result::StreamingResult;
pub use source::build_modified_source;
pub use state::{ParserState, SectionKind, StreamingContext};
