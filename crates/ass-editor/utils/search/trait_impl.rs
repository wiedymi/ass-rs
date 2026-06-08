//! `DocumentSearch` implementation for `DocumentSearchImpl`
//!
//! Wires the trait methods to the inherent search/index helpers defined in the
//! sibling modules and holds the index-build and find-replace logic.

use super::engine::DocumentSearchImpl;
use super::options::{SearchOptions, SearchResult, SearchStats};
use super::traits::DocumentSearch;
use crate::core::{EditorDocument, Range, Result};

#[cfg(feature = "std")]
use std::time::Instant;

#[cfg(not(feature = "std"))]
#[allow(dead_code)]
type Instant = u64;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

impl DocumentSearch for DocumentSearchImpl {
    fn build_index(&mut self, document: &EditorDocument) -> Result<()> {
        #[cfg(feature = "std")]
        let _start_time = Instant::now();

        let text = document.text();
        self.cached_text = text.clone(); // Cache for regex and basic searches

        #[cfg(feature = "search-index")]
        {
            let words = self.extract_words(&text);

            // Build FST index
            let fst = self.build_fst(&words)?;
            self.word_index = Some(fst);

            // Store word positions
            self.word_positions.clear();
            for (word, positions) in words {
                self.word_positions.insert(word.into_owned(), positions);
            }

            // Update statistics
            self.stats.index_size = self.word_positions.len() * 50; // Rough estimate
        }

        #[cfg(not(feature = "search-index"))]
        {
            self.stats.index_size = self.cached_text.len();
        }

        self.document_version += 1; // Simple increment for cache invalidation

        // Clear cache since index changed
        self.result_cache.clear();

        Ok(())
    }

    fn update_index(&mut self, document: &EditorDocument, changes: &[Range]) -> Result<()> {
        self.update_index_impl(document, changes)
    }

    fn search<'a>(
        &'a self,
        pattern: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult<'a>>> {
        self.search_impl(pattern, options)
    }

    fn find_replace<'a>(
        &'a self,
        document: &mut EditorDocument,
        pattern: &str,
        replacement: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult<'a>>> {
        let results = self.search(pattern, options)?;
        let mut replaced = Vec::new();

        // Apply replacements in reverse order to maintain position validity
        for result in results.iter().rev() {
            let range = Range::new(result.start, result.end);
            document.delete(range)?;
            document.insert(result.start, replacement)?;
            replaced.push(result.clone());
        }

        replaced.reverse(); // Restore original order
        Ok(replaced)
    }

    fn stats(&self) -> SearchStats {
        self.stats.clone()
    }

    fn clear_index(&mut self) {
        #[cfg(feature = "search-index")]
        {
            self.word_index = None;
            self.word_positions.clear();
        }
        self.result_cache.clear();
        self.cached_text.clear();
        self.stats.index_size = 0;
    }
}
