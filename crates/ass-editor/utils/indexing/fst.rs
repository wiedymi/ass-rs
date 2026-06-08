//! FST search index state and lightweight helpers.
//!
//! Holds the [`FstSearchIndex`] struct definition along with its constructor
//! and search-scope filtering used by the
//! [`DocumentSearch`](crate::utils::search::DocumentSearch) implementation.

use super::common::IndexEntry;
use crate::utils::search::SearchScope;

use fst::Set;

use std::{collections::HashMap, time::Instant};

/// FST-based search index for high-performance queries
#[cfg(feature = "search-index")]
pub struct FstSearchIndex {
    /// FST set for fast prefix/fuzzy matching
    pub(super) fst_set: Option<Set<Vec<u8>>>,

    /// Map from FST keys to document positions
    pub(super) position_map: HashMap<String, Vec<IndexEntry>>,

    /// Last known document content hash for cache invalidation
    pub(super) content_hash: u64,

    /// Index build timestamp for statistics
    pub(super) build_time: Instant,

    /// Index size in bytes
    pub(super) index_size: usize,
}

#[cfg(feature = "search-index")]
impl Default for FstSearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "search-index")]
impl FstSearchIndex {
    /// Create a new FST search index
    pub fn new() -> Self {
        Self {
            fst_set: None,
            position_map: HashMap::new(),
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
            index_size: 0,
        }
    }
}

#[cfg(feature = "search-index")]
impl FstSearchIndex {
    pub(super) fn matches_scope(&self, entry: &IndexEntry, scope: &SearchScope) -> bool {
        match scope {
            SearchScope::All => true,
            SearchScope::Lines { start, end } => entry.line >= *start && entry.line <= *end,
            SearchScope::Sections(sections) => {
                if let Some(ref section) = entry.section_type {
                    sections.contains(section)
                } else {
                    false
                }
            }
            SearchScope::Range(range) => {
                entry.position.offset >= range.start.offset
                    && entry.position.offset <= range.end.offset
            }
        }
    }
}
