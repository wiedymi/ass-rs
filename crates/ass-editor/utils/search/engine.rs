//! `DocumentSearchImpl` state and cache management
//!
//! Holds the search implementation struct along with construction, cache
//! sizing, and cache-key helpers shared by the sibling search modules.

use super::options::{SearchOptions, SearchResult, SearchStats};

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use hashbrown::HashMap;

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec::Vec};

#[cfg(feature = "search-index")]
use crate::core::Position;

#[cfg(feature = "search-index")]
use fst::Set;

/// Document search implementation with optional FST indexing
#[derive(Debug)]
pub struct DocumentSearchImpl {
    /// FST set for fast prefix/substring searches (when feature enabled)
    #[cfg(feature = "search-index")]
    pub(super) word_index: Option<Set<Vec<u8>>>,

    /// Mapping from words to their positions in the document
    #[cfg(feature = "search-index")]
    pub(super) word_positions: HashMap<String, Vec<Position>>,

    /// Last known document version for incremental updates
    pub(super) document_version: u64,

    /// Cache of recent search results (owned to persist across calls)
    pub(super) result_cache: HashMap<String, Vec<SearchResult<'static>>>,

    /// Maximum cache size
    pub(super) max_cache_entries: usize,

    /// Search statistics
    pub(super) stats: SearchStats,

    /// Cached document text for regex and basic searches
    pub(super) cached_text: String,
}

impl DocumentSearchImpl {
    /// Create a new unified search instance
    ///
    /// # Examples
    ///
    /// ```
    /// use ass_editor::utils::search::DocumentSearchImpl;
    ///
    /// let mut search = DocumentSearchImpl::new();
    /// // Use search with documents...
    /// ```
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "search-index")]
            word_index: None,
            #[cfg(feature = "search-index")]
            word_positions: HashMap::new(),
            document_version: 0,
            result_cache: HashMap::new(),
            max_cache_entries: 100,
            stats: SearchStats {
                match_count: 0,
                search_time_us: 0,
                hit_limit: false,
                index_size: 0,
            },
            cached_text: String::new(),
        }
    }

    /// Set maximum number of cached search results
    pub fn set_cache_size(&mut self, size: usize) {
        self.max_cache_entries = size;
        self.trim_cache();
    }

    /// Trim cache to maximum size
    fn trim_cache(&mut self) {
        if self.result_cache.len() > self.max_cache_entries {
            // Simple LRU-like eviction: remove half of the entries
            let keys_to_remove: Vec<String> = self
                .result_cache
                .keys()
                .take(self.result_cache.len() / 2)
                .cloned()
                .collect();

            for key in keys_to_remove {
                self.result_cache.remove(&key);
            }
        }
    }

    /// Generate cache key for search parameters
    pub(super) fn cache_key(&self, pattern: &str, options: &SearchOptions) -> String {
        format!(
            "{}|{}|{}|{}|{:?}",
            pattern,
            options.case_sensitive,
            options.whole_words,
            options.max_results,
            options.scope
        )
    }
}

impl Default for DocumentSearchImpl {
    fn default() -> Self {
        Self::new()
    }
}
