//! Linear search index fallback used when the FST backend is unavailable.

use super::common::{calculate_hash, IndexEntry};
use crate::core::{EditorDocument, Position, Range, Result};
use crate::utils::search::{DocumentSearch, SearchOptions, SearchResult, SearchStats};

#[cfg(feature = "std")]
use std::{borrow::Cow, collections::HashMap, time::Instant};

#[cfg(not(feature = "std"))]
use alloc::{
    borrow::Cow,
    collections::BTreeMap as HashMap,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(not(feature = "std"))]
type Instant = u64;

/// Linear search fallback for when FST is not available
pub struct LinearSearchIndex {
    /// Simple word -> positions mapping
    word_positions: HashMap<String, Vec<IndexEntry>>,

    /// Content hash for invalidation
    content_hash: u64,

    /// Build timestamp
    build_time: Instant,
}

impl Default for LinearSearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl LinearSearchIndex {
    /// Create a new linear search index
    pub fn new() -> Self {
        Self {
            word_positions: HashMap::new(),
            content_hash: 0,
            build_time: {
                #[cfg(feature = "std")]
                {
                    Instant::now()
                }
                #[cfg(not(feature = "std"))]
                {
                    0
                }
            },
        }
    }
}

impl DocumentSearch for LinearSearchIndex {
    fn build_index(&mut self, document: &EditorDocument) -> Result<()> {
        let content = document.text();
        self.content_hash = calculate_hash(&content);
        self.build_time = {
            #[cfg(feature = "std")]
            {
                Instant::now()
            }
            #[cfg(not(feature = "std"))]
            {
                0
            }
        };

        // Simple word extraction for linear search
        self.word_positions.clear();

        let mut line_start = 0;

        for (current_line, line) in content.lines().enumerate() {
            for (word_start, word) in line.split_whitespace().enumerate() {
                let entry = IndexEntry {
                    position: Position::new(line_start + word_start),
                    context: line.to_string(),
                    line: current_line,
                    column: word_start,
                    section_type: None,
                };
                self.word_positions
                    .entry(word.to_lowercase())
                    .or_default()
                    .push(entry);
            }
            line_start += line.len() + 1;
        }

        Ok(())
    }

    fn update_index(&mut self, document: &EditorDocument, _changes: &[Range]) -> Result<()> {
        self.build_index(document)
    }

    fn search(&self, pattern: &str, _options: &SearchOptions) -> Result<Vec<SearchResult<'_>>> {
        let query = pattern.to_lowercase();
        let mut results = Vec::new();

        // Simple linear search through indexed words
        for (word, entries) in &self.word_positions {
            if word.contains(&query) {
                for entry in entries {
                    results.push(SearchResult {
                        start: entry.position,
                        end: Position::new(entry.position.offset + pattern.len()),
                        text: Cow::Owned(pattern.to_string()),
                        context: Cow::Owned(entry.context.clone()),
                        line: entry.line,
                        column: entry.column,
                    });
                }
            }
        }

        Ok(results)
    }

    fn find_replace<'a>(
        &'a self,
        document: &mut EditorDocument,
        pattern: &str,
        replacement: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult<'a>>> {
        let results = self.search(pattern, options)?;

        for result in &results {
            let range = Range::new(result.start, result.end);
            document.replace(range, replacement)?;
        }

        Ok(results)
    }

    fn stats(&self) -> SearchStats {
        SearchStats {
            match_count: self.word_positions.len(),
            search_time_us: {
                #[cfg(feature = "std")]
                {
                    self.build_time.elapsed().as_micros() as u64
                }
                #[cfg(not(feature = "std"))]
                {
                    0
                }
            },
            hit_limit: false,
            index_size: self.word_positions.len() * 64, // Rough estimate
        }
    }

    fn clear_index(&mut self) {
        self.word_positions.clear();
    }
}
