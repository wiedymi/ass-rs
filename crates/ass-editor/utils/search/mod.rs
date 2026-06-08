//! Search functionality for ASS documents
//!
//! Provides the `DocumentSearch` trait for efficient text search with
//! FST-based indexing, regex support, and incremental updates. Targets
//! <10ms initial indexing and <1ms query response times.

mod basic;
mod engine;
mod incremental;
mod index;
mod options;
mod query;
mod regex_search;
mod trait_impl;
mod traits;

#[cfg(test)]
mod regex_tests;
#[cfg(test)]
mod tests;

pub use engine::DocumentSearchImpl;
pub use options::{SearchError, SearchOptions, SearchResult, SearchScope, SearchStats};
pub use traits::{create_search, DocumentSearch};
