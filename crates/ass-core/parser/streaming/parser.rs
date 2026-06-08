//! High-performance streaming parser state machine
//!
//! Defines [`StreamingParser`], which incrementally consumes input chunks,
//! buffers partial lines, and dispatches complete lines to the
//! [`super::LineProcessor`] before producing a [`super::StreamingResult`].

use super::{LineProcessor, ParseDelta, StreamingResult};
use crate::{utils::CoreError, Result, ScriptVersion};
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

/// High-performance streaming parser for ASS scripts
///
/// Processes input chunks incrementally using a state machine approach.
/// Supports partial lines, incomplete sections, and memory-efficient parsing.
pub struct StreamingParser {
    /// Line processor for parsing individual lines
    processor: LineProcessor,
    /// Buffer for incomplete lines
    pub(super) buffer: String,
    /// Parsed sections in document order
    pub(super) sections: Vec<String>,

    #[cfg(feature = "benches")]
    /// Peak memory usage for benchmarking
    peak_memory: usize,
}

impl StreamingParser {
    /// Create new streaming parser
    #[must_use]
    pub const fn new() -> Self {
        Self {
            processor: LineProcessor::new(),
            buffer: String::new(),
            sections: Vec::new(),

            #[cfg(feature = "benches")]
            peak_memory: 0,
        }
    }

    /// Create parser with custom capacity
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            processor: LineProcessor::new(),
            buffer: String::new(),
            sections: Vec::with_capacity(capacity),

            #[cfg(feature = "benches")]
            peak_memory: 0,
        }
    }

    /// Feed chunk of data to parser
    ///
    /// # Errors
    ///
    /// Returns an error if the chunk contains invalid UTF-8 or parsing fails.
    pub fn feed_chunk(&mut self, chunk: &[u8]) -> Result<Vec<ParseDelta<'static>>> {
        if chunk.is_empty() {
            return Ok(Vec::new());
        }

        let chunk_str = core::str::from_utf8(chunk)
            .map_err(|e| CoreError::parse(format!("Invalid UTF-8: {e}")))?;

        self.buffer.push_str(chunk_str);

        let mut all_deltas = Vec::new();
        let lines: Vec<String> = self.buffer.lines().map(str::to_string).collect();
        let ends_with_newline = self.buffer.ends_with('\n') || self.buffer.ends_with('\r');

        let complete_lines = if ends_with_newline {
            lines.len()
        } else {
            lines.len().saturating_sub(1)
        };

        // Process complete lines
        for line in &lines[..complete_lines] {
            let deltas = self.processor.process_line(line)?;
            all_deltas.extend(deltas.into_deltas());
        }

        // Update buffer with incomplete line
        if complete_lines < lines.len() {
            self.buffer.clone_from(&lines[complete_lines]);
        } else {
            self.buffer.clear();
        }

        #[cfg(feature = "benches")]
        {
            let current_memory = self.calculate_memory_usage();
            if current_memory > self.peak_memory {
                self.peak_memory = current_memory;
            }
        }

        Ok(all_deltas)
    }

    /// Finish parsing and return final result
    ///
    /// # Errors
    ///
    /// Returns an error if the final line processing fails.
    pub fn finish(mut self) -> Result<StreamingResult> {
        if !self.buffer.trim().is_empty() {
            let _deltas = self.processor.process_line(&self.buffer.clone())?;
        }

        Ok(StreamingResult {
            sections: self.sections,
            version: ScriptVersion::AssV4,
            issues: Vec::new(),
        })
    }

    /// Reset parser state for reuse
    pub fn reset(&mut self) {
        self.processor.reset();
        self.buffer.clear();
        self.sections.clear();

        #[cfg(feature = "benches")]
        {
            self.peak_memory = 0;
        }
    }

    /// Get peak memory usage (benchmarks only)
    #[cfg(feature = "benches")]
    #[must_use]
    pub const fn peak_memory(&self) -> usize {
        self.peak_memory
    }

    #[cfg(feature = "benches")]
    /// Calculate current memory usage for benchmarking
    fn calculate_memory_usage(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.buffer.capacity()
            + self.sections.capacity() * core::mem::size_of::<String>()
    }
}

impl Default for StreamingParser {
    fn default() -> Self {
        Self::new()
    }
}
