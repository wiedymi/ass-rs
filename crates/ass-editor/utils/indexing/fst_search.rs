//! [`DocumentSearch`] implementation for [`FstSearchIndex`].

use super::common::calculate_hash;
use super::fst::FstSearchIndex;
use crate::core::{EditorDocument, Position, Range, Result};
use crate::utils::search::{DocumentSearch, SearchOptions, SearchResult, SearchStats};

use fst::{automaton, IntoStreamer, Streamer};

use std::time::Instant;

#[cfg(feature = "search-index")]
impl DocumentSearch for FstSearchIndex {
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

        let words = self.extract_words(&content);
        self.build_fst(words)?;

        Ok(())
    }

    fn update_index(&mut self, document: &EditorDocument, _changes: &[Range]) -> Result<()> {
        // For simplicity, rebuild entire index on changes
        // In production, this could be optimized for incremental updates
        self.build_index(document)
    }

    fn search<'a>(
        &'a self,
        pattern: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult<'a>>> {
        #[cfg(feature = "std")]
        let _start_time = Instant::now();
        let mut results = Vec::new();

        if let Some(ref fst_set) = self.fst_set {
            let query = if options.case_sensitive {
                pattern.to_string()
            } else {
                pattern.to_lowercase()
            };

            // For simplicity, use subsequence automaton for all searches
            // In production, this could be optimized with different automaton types
            let automaton = automaton::Subsequence::new(&query);
            let mut stream = fst_set.search(automaton).into_stream();
            let mut count = 0;

            while let Some(key) = stream.next() {
                if options.max_results > 0 && count >= options.max_results {
                    break;
                }

                let key_str = String::from_utf8_lossy(key);
                if let Some(entries) = self.position_map.get(key_str.as_ref()) {
                    for entry in entries {
                        // Apply scope filtering
                        if self.matches_scope(entry, &options.scope) {
                            results.push(SearchResult {
                                start: entry.position,
                                end: Position::new(entry.position.offset + pattern.len()),
                                text: std::borrow::Cow::Owned(pattern.to_string()),
                                context: std::borrow::Cow::Owned(entry.context.clone()),
                                line: entry.line,
                                column: entry.column,
                            });
                            count += 1;

                            if options.max_results > 0 && count >= options.max_results {
                                break;
                            }
                        }
                    }
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

        // Apply replacements in reverse order to maintain position validity
        let mut sorted_results = results.clone();
        sorted_results.sort_by_key(|r| core::cmp::Reverse(r.start.offset));

        for result in &sorted_results {
            let range = Range::new(result.start, result.end);
            document.replace(range, replacement)?;
        }

        Ok(results)
    }

    fn stats(&self) -> SearchStats {
        SearchStats {
            match_count: self.position_map.len(),
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
            index_size: self.index_size,
        }
    }

    fn clear_index(&mut self) {
        self.fst_set = None;
        self.position_map.clear();
        self.index_size = 0;
    }
}
