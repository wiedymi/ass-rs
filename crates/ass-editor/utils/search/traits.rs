//! `DocumentSearch` trait definition and search factory
//!
//! Declares the unified search interface implemented by
//! [`DocumentSearchImpl`](super::DocumentSearchImpl) along with the
//! [`create_search`] factory helper.

use super::engine::DocumentSearchImpl;
use super::options::{SearchOptions, SearchResult, SearchStats};
use crate::core::{EditorDocument, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, vec::Vec};

/// Main trait for document search functionality
///
/// Provides unified search capabilities with optional FST-based indexing for
/// fast substring searches and regex support for complex pattern matching.
///
/// # Examples
///
/// ```
/// use ass_editor::{EditorDocument, utils::search::*};
///
/// let mut doc = EditorDocument::from_content(r#"
/// [Events]
/// Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World
/// Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,Goodbye World
/// "#).unwrap();
///
/// // Create and use a search instance
/// let mut search = DocumentSearchImpl::new();
/// search.build_index(&doc).unwrap();
///
/// // Basic text search
/// let options = SearchOptions::default();
/// let results = search.search("World", &options).unwrap();
/// assert_eq!(results.len(), 2);
///
/// // Case-insensitive search with options
/// let options = SearchOptions {
///     case_sensitive: false,
///     max_results: 10,
///     ..Default::default()
/// };
/// ```
pub trait DocumentSearch {
    /// Build or rebuild the search index for the document
    fn build_index(&mut self, document: &EditorDocument) -> Result<()>;

    /// Update the search index incrementally after document changes
    fn update_index(&mut self, document: &EditorDocument, changes: &[Range]) -> Result<()>;

    /// Search for a pattern in the document
    fn search<'a>(
        &'a self,
        pattern: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult<'a>>>;

    /// Find and replace text in the document
    fn find_replace<'a>(
        &'a self,
        document: &mut EditorDocument,
        pattern: &str,
        replacement: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult<'a>>>;

    /// Get search statistics
    fn stats(&self) -> SearchStats;

    /// Clear the search index to free memory
    fn clear_index(&mut self);
}

/// Factory function to create a document search implementation
pub fn create_search() -> Box<dyn DocumentSearch> {
    Box::new(DocumentSearchImpl::new())
}
