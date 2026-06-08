//! FST-based search indexing for fast ASS content queries
//!
//! Provides trie-based indexing for regex and fuzzy search queries as specified
//! in the architecture (lines 142-143). Fallback to linear search for WASM.

mod common;
mod linear;

#[cfg(feature = "search-index")]
mod fst;
#[cfg(feature = "search-index")]
mod fst_build;
#[cfg(feature = "search-index")]
mod fst_search;

#[cfg(test)]
mod tests;

pub use common::IndexEntry;
pub use linear::LinearSearchIndex;

#[cfg(feature = "search-index")]
pub use fst::FstSearchIndex;

use crate::utils::search::DocumentSearch;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

/// Factory function to create the appropriate search index
pub fn create_search_index() -> Box<dyn DocumentSearch> {
    #[cfg(feature = "search-index")]
    {
        Box::new(FstSearchIndex::new())
    }
    #[cfg(not(feature = "search-index"))]
    {
        Box::new(LinearSearchIndex::new())
    }
}
