//! Search query execution
//!
//! Implements the core query dispatch for
//! [`DocumentSearchImpl`](super::DocumentSearchImpl), routing between FST,
//! regex, and basic search depending on enabled features and options.

use super::engine::DocumentSearchImpl;
use super::options::{SearchOptions, SearchResult};
use crate::core::Result;

#[cfg(feature = "std")]
use std::time::Instant;

#[cfg(not(feature = "std"))]
#[allow(dead_code)]
type Instant = u64;

#[cfg(feature = "search-index")]
use crate::core::Position;

#[cfg(feature = "search-index")]
use fst::{automaton, IntoStreamer, Streamer};

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

#[cfg(all(feature = "search-index", not(feature = "std")))]
use alloc::{format, string::String};

impl DocumentSearchImpl {
    pub(super) fn search_impl<'a>(
        &'a self,
        pattern: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult<'a>>> {
        #[cfg(feature = "std")]
        let _start_time = Instant::now();

        // Check cache first
        let cache_key = self.cache_key(pattern, options);
        if let Some(cached_results) = self.result_cache.get(&cache_key) {
            return Ok(cached_results.clone());
        }

        let results = if options.use_regex {
            // Use regex search
            #[cfg(all(feature = "formats", feature = "std"))]
            {
                self.regex_search(pattern, options)?
            }
            #[cfg(not(all(feature = "formats", feature = "std")))]
            {
                return Err(crate::core::EditorError::CommandFailed {
                    message: "Regex search requires the 'formats' feature to be enabled"
                        .to_string(),
                });
            }
        } else {
            // Check if we should use FST or basic search
            #[cfg(feature = "search-index")]
            {
                if let Some(ref fst) = self.word_index {
                    // Use FST for search
                    let search_pattern = if options.case_sensitive {
                        pattern.to_string()
                    } else {
                        pattern.to_lowercase()
                    };

                    let mut results = Vec::new();

                    // Create automaton for prefix search
                    let mut stream = fst
                        .search(automaton::Str::new(&search_pattern))
                        .into_stream();

                    while let Some(key) = stream.next() {
                        let word = String::from_utf8_lossy(key);
                        if let Some(positions) = self.word_positions.get(word.as_ref()) {
                            for &pos in positions {
                                results.push(SearchResult {
                                    start: pos,
                                    end: Position::new(pos.offset + pattern.len()),
                                    text: std::borrow::Cow::Owned(pattern.to_string()),
                                    context: std::borrow::Cow::Owned(format!(
                                        "Offset {}",
                                        pos.offset
                                    )),
                                    line: 0, // Would need document context to calculate actual line
                                    column: pos.offset,
                                });

                                if options.max_results > 0 && results.len() >= options.max_results {
                                    break;
                                }
                            }
                        }

                        if options.max_results > 0 && results.len() >= options.max_results {
                            break;
                        }
                    }

                    results
                } else {
                    // FST index not built yet, use basic search
                    self.basic_search(pattern, options)?
                }
            }
            #[cfg(not(feature = "search-index"))]
            {
                // Use basic text search when FST is not available
                self.basic_search(pattern, options)?
            }
        };

        // Update statistics (would need mutable access in real implementation)
        // self.stats.match_count = results.len();
        // self.stats.search_time_us = search_time;
        // self.stats.hit_limit = options.max_results > 0 && results.len() >= options.max_results;

        Ok(results)
    }
}
