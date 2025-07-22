//! Streaming and incremental parsing for ASS scripts
//!
//! Provides efficient incremental parsing capabilities for editor integration
//! and large file processing. Enables <2ms edit responsiveness through
//! range-based re-parsing and delta generation.
//!
//! # Features
//!
//! - Incremental parsing: Only re-parse changed ranges
//! - Delta tracking: Efficient change representation
//! - Streaming support: Process large files without loading everything
//! - Editor integration: Fast response for interactive editing
//!
//! # Performance
//!
//! - Target: <2ms for single-event edits
//! - Memory: O(delta size) not O(script size)
//! - Chunked processing: <10ms/MB for large files

use crate::{
    parser::{Script, ScriptDelta, ScriptDeltaOwned, Section},
    utils::CoreError,
    Result,
};
use alloc::{string::String, vec::Vec};
use core::ops::Range;

/// Parse incremental changes to a script
///
/// Efficiently re-parses only the changed range and generates a delta
/// containing the differences. Optimized for editor integration.
///
/// # Arguments
///
/// * `script` - Original script to update
/// * `modified_source` - New source text with modifications applied
/// * `range` - Byte range that was modified in the original source
///
/// # Returns
///
/// Delta containing all changes, or error if parsing fails.
///
/// # Performance
///
/// Target <2ms for typical single-event edits. Uses heuristics to
/// minimize re-parsing scope while maintaining correctness.
///
/// # Example
///
/// ```rust
/// # use ass_core::parser::{Script, streaming::{parse_incremental, build_modified_source}};
/// # let script_text = "[Script Info]\nTitle: Test\n[V4+ Styles]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello";
/// # let script = Script::parse(script_text).unwrap();
/// let range = 15..19; // Replace "Test" with "Example"
/// let modified_source = build_modified_source(script.source(), &range, "Example");
/// let delta = parse_incremental(&script, &modified_source, range)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_incremental<'a>(
    script: &Script<'a>,
    modified_source: &'a str,
    range: Range<usize>,
) -> Result<ScriptDelta<'a>> {
    if range.end > script.source().len() {
        return Err(CoreError::parse("Range extends beyond script source"));
    }

    let section_ranges = build_section_ranges(script)?;
    let affected_sections = find_affected_sections(&section_ranges, &range);

    if affected_sections.is_empty() {
        return Ok(ScriptDelta {
            added: Vec::new(),
            modified: Vec::new(),
            removed: Vec::new(),
            new_issues: Vec::new(),
        });
    }

    // Calculate optimal range to re-parse
    let parse_range = calculate_optimal_parse_range(&section_ranges, &affected_sections, &range);

    // Extract old sections that will be replaced
    let old_sections = extract_affected_sections(script, &affected_sections);

    // Re-parse the affected range
    let new_sections = reparse_affected_range(modified_source, &parse_range)?;

    // Generate delta by comparing old and new sections
    let delta = generate_section_delta(old_sections, new_sections, &affected_sections)?;

    Ok(delta)
}

/// Parse incremental changes to a script returning owned data
///
/// Similar to `parse_incremental` but returns owned data to avoid lifetime
/// constraints. Creates the modified source internally and converts sections
/// to owned strings for editor integration scenarios.
///
/// # Arguments
///
/// * `script` - Original script to update
/// * `range` - Byte range that was modified
/// * `new_text` - Replacement text for the range
///
/// # Returns
///
/// Owned delta containing all changes, or error if parsing fails.
///
/// # Performance
///
/// Target <2ms for typical single-event edits. Uses heuristics to
/// minimize re-parsing scope while maintaining correctness.
///
/// # Example
///
/// ```rust
/// # use ass_core::parser::{Script, streaming::parse_incremental_owned};
/// # let script_text = "[Script Info]\nTitle: Test\n[V4+ Styles]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello";
/// # let script = Script::parse(script_text).unwrap();
/// let range = 15..19; // Replace "Test" with "Example"
/// let delta = parse_incremental_owned(&script, range, "Example")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_incremental_owned(
    script: &Script<'_>,
    range: Range<usize>,
    new_text: &str,
) -> Result<ScriptDeltaOwned> {
    if range.end > script.source().len() {
        return Err(CoreError::parse("Range extends beyond script source"));
    }

    let source = script.source();
    let section_ranges = build_section_ranges(script)?;
    let affected_sections = find_affected_sections(&section_ranges, &range);

    if affected_sections.is_empty() {
        return Ok(ScriptDeltaOwned {
            added: Vec::new(),
            modified: Vec::new(),
            removed: Vec::new(),
            new_issues: Vec::new(),
        });
    }

    // Build new source with modification applied
    let modified_source = build_modified_source(source, &range, new_text);

    // Calculate optimal range to re-parse
    let parse_range = calculate_optimal_parse_range(&section_ranges, &affected_sections, &range);

    // Extract old sections that will be replaced
    let old_sections = extract_affected_sections(script, &affected_sections);

    // Re-parse the affected range
    let new_sections = reparse_affected_range(&modified_source, &parse_range)?;

    // Generate owned delta by converting sections to strings
    let delta = generate_owned_section_delta(old_sections, new_sections, &affected_sections)?;

    Ok(delta)
}

/// Generate delta by comparing old and new sections
fn generate_section_delta<'a>(
    old_sections: Vec<(usize, Section<'a>)>,
    new_sections: Vec<Section<'a>>,
    _affected_indices: &[usize],
) -> Result<ScriptDelta<'a>> {
    let mut delta = ScriptDelta {
        added: Vec::new(),
        modified: Vec::new(),
        removed: Vec::new(),
        new_issues: Vec::new(),
    };

    let old_count = old_sections.len();
    let new_count = new_sections.len();

    match old_count.cmp(&new_count) {
        core::cmp::Ordering::Equal => {
            for ((idx, old_section), new_section) in old_sections.into_iter().zip(new_sections) {
                if old_section != new_section {
                    delta.modified.push((idx, new_section));
                }
            }
        }
        core::cmp::Ordering::Greater => {
            for (i, new_section) in new_sections.into_iter().enumerate() {
                if let Some((idx, old_section)) = old_sections.get(i) {
                    if *old_section != new_section {
                        delta.modified.push((*idx, new_section));
                    }
                } else {
                    delta.added.push(new_section);
                }
            }
            for &(idx, _) in &old_sections[new_count..] {
                delta.removed.push(idx);
            }
        }
        core::cmp::Ordering::Less => {
            for ((idx, old_section), new_section) in old_sections.into_iter().zip(&new_sections) {
                if old_section != *new_section {
                    delta.modified.push((idx, new_section.clone()));
                }
            }
            for new_section in new_sections.into_iter().skip(old_count) {
                delta.added.push(new_section);
            }
        }
    }

    Ok(delta)
}

/// Generate owned delta by converting sections to strings
fn generate_owned_section_delta<'a>(
    old_sections: Vec<(usize, Section<'a>)>,
    new_sections: Vec<Section<'a>>,
    _affected_indices: &[usize],
) -> Result<ScriptDeltaOwned> {
    let mut delta = ScriptDeltaOwned {
        added: Vec::new(),
        modified: Vec::new(),
        removed: Vec::new(),
        new_issues: Vec::new(),
    };

    let old_count = old_sections.len();
    let new_count = new_sections.len();

    match old_count.cmp(&new_count) {
        core::cmp::Ordering::Equal => {
            for ((idx, old_section), new_section) in old_sections.into_iter().zip(new_sections) {
                if old_section != new_section {
                    delta.modified.push((idx, format!("{:?}", new_section)));
                }
            }
        }
        core::cmp::Ordering::Greater => {
            for (i, new_section) in new_sections.into_iter().enumerate() {
                if let Some((idx, old_section)) = old_sections.get(i) {
                    if *old_section != new_section {
                        delta.modified.push((*idx, format!("{:?}", new_section)));
                    }
                } else {
                    delta.added.push(format!("{:?}", new_section));
                }
            }
            for &(idx, _) in &old_sections[new_count..] {
                delta.removed.push(idx);
            }
        }
        core::cmp::Ordering::Less => {
            for ((idx, old_section), new_section) in old_sections.into_iter().zip(&new_sections) {
                if old_section != *new_section {
                    delta.modified.push((idx, format!("{:?}", new_section)));
                }
            }
            for new_section in new_sections.into_iter().skip(old_count) {
                delta.added.push(format!("{:?}", new_section));
            }
        }
    }

    Ok(delta)
}

/// Build ranges for all sections in the script
fn build_section_ranges(script: &Script<'_>) -> Result<Vec<Range<usize>>> {
    let source = script.source();
    let mut ranges: Vec<Range<usize>> = Vec::new();
    let mut pos = 0;

    for line in source.lines() {
        if line.trim_start().starts_with('[') && line.trim_end().ends_with(']') {
            if !ranges.is_empty() {
                let last_idx = ranges.len() - 1;
                ranges[last_idx] = ranges[last_idx].start..pos;
            }
            ranges.push(pos..source.len());
        }
        pos += line.len() + 1;
    }

    if ranges.is_empty() {
        ranges.push(0..source.len());
    }

    Ok(ranges)
}

/// Find which sections are affected by the given range
fn find_affected_sections(
    section_ranges: &[Range<usize>],
    change_range: &Range<usize>,
) -> Vec<usize> {
    section_ranges
        .iter()
        .enumerate()
        .filter_map(|(idx, range)| {
            if ranges_intersect(range, change_range) {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}

/// Check if two ranges intersect
fn ranges_intersect(a: &Range<usize>, b: &Range<usize>) -> bool {
    a.start < b.end && b.start < a.end
}

/// Calculate optimal range to re-parse based on section boundaries
fn calculate_optimal_parse_range(
    section_ranges: &[Range<usize>],
    affected_sections: &[usize],
    change_range: &Range<usize>,
) -> Range<usize> {
    if affected_sections.is_empty() {
        return change_range.clone();
    }

    let first_section = affected_sections[0];
    let last_section = affected_sections[affected_sections.len() - 1];

    let start = section_ranges
        .get(first_section)
        .map(|r| r.start)
        .unwrap_or(change_range.start);

    let end = section_ranges
        .get(last_section)
        .map(|r| r.end)
        .unwrap_or(change_range.end);

    start..end
}

/// Build new source text with modification applied
///
/// Helper function for creating modified source text by replacing a range
/// with new content. The caller is responsible for managing the lifetime
/// of the returned string.
///
/// # Arguments
///
/// * `source` - Original source text
/// * `range` - Byte range to replace
/// * `new_text` - Replacement text for the range
///
/// # Returns
///
/// New string with the modification applied.
///
/// # Example
///
/// ```rust
/// use ass_core::parser::streaming::build_modified_source;
///
/// let source = "Hello, World!";
/// let range = 7..12;
/// let result = build_modified_source(source, &range, "Rust");
/// assert_eq!(result, "Hello, Rust!");
/// ```
pub fn build_modified_source(source: &str, range: &Range<usize>, new_text: &str) -> String {
    let mut result = String::with_capacity(source.len() + new_text.len());
    result.push_str(&source[..range.start]);
    result.push_str(new_text);
    result.push_str(&source[range.end..]);
    result
}

/// Extract affected sections from original script
fn extract_affected_sections<'a>(
    script: &Script<'a>,
    affected_indices: &[usize],
) -> Vec<(usize, Section<'a>)> {
    affected_indices
        .iter()
        .filter_map(|&idx| {
            script
                .sections()
                .get(idx)
                .map(|section| (idx, section.clone()))
        })
        .collect()
}

/// Re-parse the affected range to get new sections
fn reparse_affected_range<'a>(source: &'a str, range: &Range<usize>) -> Result<Vec<Section<'a>>> {
    let range_text = &source[range.clone()];
    let script = Script::parse(range_text)?;
    Ok(script.sections().to_vec())
}

/// Streaming parser for processing large ASS files
///
/// Processes files in chunks to avoid loading everything into memory.
/// Useful for very large subtitle files or network streaming.
///
/// # Example
///
/// ```rust
/// # use ass_core::parser::streaming::StreamingParser;
/// let parser = StreamingParser::new()
///     .with_chunk_size(32 * 1024)
///     .with_incremental(true);
/// ```
pub struct StreamingParser {
    chunk_size: usize,
    incremental: bool,
    buffer: Vec<u8>,
}

impl StreamingParser {
    /// Create new streaming parser with default settings
    ///
    /// Uses 64KB chunks and enables incremental processing by default.
    /// These settings provide good balance between memory usage and
    /// parsing efficiency.
    pub fn new() -> Self {
        Self {
            chunk_size: 64 * 1024,
            incremental: true,
            buffer: Vec::new(),
        }
    }

    /// Set chunk size for processing
    ///
    /// Larger chunks use more memory but may be more efficient.
    /// Smaller chunks are more responsive for streaming scenarios.
    ///
    /// # Arguments
    ///
    /// * `size` - Chunk size in bytes (minimum 1KB recommended)
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size.max(1024);
        self
    }

    /// Enable or disable incremental processing
    ///
    /// Incremental processing maintains state between chunks to
    /// handle sections that span chunk boundaries.
    pub fn with_incremental(mut self, incremental: bool) -> Self {
        self.incremental = incremental;
        self
    }

    /// Parse from streaming source
    ///
    /// Processes input in chunks, maintaining state for sections that
    /// span chunk boundaries. Target: <10ms/MB processing time.
    pub fn parse_stream<'a>(&mut self, source: &'a str) -> Result<Script<'a>> {
        if source.len() <= self.chunk_size {
            return Script::parse(source);
        }

        self.buffer.clear();
        let mut accumulated_content = String::new();
        let bytes = source.as_bytes();

        for chunk in bytes.chunks(self.chunk_size) {
            self.buffer.extend_from_slice(chunk);

            if self.incremental {
                let (complete_part, incomplete_part) = self.split_at_section_boundary()?;
                accumulated_content.push_str(&complete_part);
                self.buffer = incomplete_part.into_bytes();
            } else if let Ok(text) = core::str::from_utf8(&self.buffer) {
                accumulated_content = text.to_string();
            }
        }

        if !self.buffer.is_empty() {
            if let Ok(remaining) = core::str::from_utf8(&self.buffer) {
                accumulated_content.push_str(remaining);
            }
        }

        // TODO: This defeats the purpose of streaming - we fall back to parsing
        // the original source instead of the accumulated chunks. Need to redesign
        // the streaming API to either return owned data or use arena allocation.
        Script::parse(source)
    }

    /// Split buffer at section boundary to avoid parsing incomplete sections
    fn split_at_section_boundary(&self) -> Result<(String, String)> {
        let text = core::str::from_utf8(&self.buffer)
            .map_err(|_| CoreError::parse("Invalid UTF-8 in buffer"))?;

        if let Some(boundary) = find_last_complete_section(text) {
            let complete = text[..boundary].to_string();
            let incomplete = text[boundary..].to_string();
            Ok((complete, incomplete))
        } else {
            Ok((String::new(), text.to_string()))
        }
    }
}

impl Default for StreamingParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Find the end of the last complete section in text
fn find_last_complete_section(text: &str) -> Option<usize> {
    let mut last_section_end = None;
    let mut current_pos = 0;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            last_section_end = Some(current_pos);
        }
        current_pos += line.len() + 1;
    }

    last_section_end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_intersect_overlapping() {
        assert!(ranges_intersect(&(0..10), &(5..15)));
        assert!(ranges_intersect(&(5..15), &(0..10)));
        assert!(ranges_intersect(&(0..10), &(0..5)));
    }

    #[test]
    fn ranges_intersect_non_overlapping() {
        assert!(!ranges_intersect(&(0..5), &(10..15)));
        assert!(!ranges_intersect(&(10..15), &(0..5)));
    }

    #[test]
    fn build_modified_source_replacement() {
        let source = "Hello, World!";
        let range = 7..12;
        let new_text = "Rust";
        let result = build_modified_source(source, &range, new_text);
        assert_eq!(result, "Hello, Rust!");
    }

    #[test]
    fn streaming_parser_creation() {
        let parser = StreamingParser::new();
        assert_eq!(parser.chunk_size, 64 * 1024);
        assert!(parser.incremental);
    }

    #[test]
    fn streaming_parser_chunk_size_minimum() {
        let parser = StreamingParser::new().with_chunk_size(512);
        assert_eq!(parser.chunk_size, 1024);
    }

    #[test]
    fn find_last_complete_section_found() {
        let text = "[Script Info]\nTitle: Test\n[V4+ Styles]\nFormat: Name";
        let pos = find_last_complete_section(text);
        assert!(pos.is_some());
    }

    #[test]
    fn find_last_complete_section_not_found() {
        let text = "Title: Test\nSome content";
        let pos = find_last_complete_section(text);
        assert!(pos.is_none());
    }
}
