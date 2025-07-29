//! Search functionality for ASS documents
//!
//! Provides the `DocumentSearch` trait for efficient text search with
//! FST-based indexing, regex support, and incremental updates. Targets
//! <10ms initial indexing and <1ms query response times.

use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(all(feature = "search-index", not(feature = "nostd")))]
use std::collections::HashMap;

#[cfg(all(feature = "search-index", feature = "nostd"))]
use hashbrown::HashMap;

#[cfg(feature = "search-index")]
use fst::{automaton, IntoStreamer, Set, SetBuilder, Streamer};

/// Search options for document queries
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchOptions {
    /// Whether to perform case-sensitive search
    pub case_sensitive: bool,

    /// Whether to match whole words only
    pub whole_words: bool,

    /// Maximum number of results to return (0 = unlimited)
    pub max_results: usize,

    /// Whether to use regular expressions
    pub use_regex: bool,

    /// Search scope (e.g., specific sections or lines)
    pub scope: SearchScope,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_words: false,
            max_results: 100,
            use_regex: false,
            scope: SearchScope::All,
        }
    }
}

/// Defines the scope of a search operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchScope {
    /// Search entire document
    All,

    /// Search within specific line range
    Lines { start: usize, end: usize },

    /// Search within specific sections (e.g., Events, Styles)
    Sections(Vec<String>),

    /// Search within a character range
    Range(Range),
}

/// A single search result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    /// Position where the match starts
    pub start: Position,

    /// Position where the match ends
    pub end: Position,

    /// The matched text
    pub text: String,

    /// Context around the match (for display purposes)
    pub context: String,

    /// Line number where the match occurs (0-based)
    pub line: usize,

    /// Column where the match starts (0-based)
    pub column: usize,
}

/// Statistics about search operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchStats {
    /// Number of matches found
    pub match_count: usize,

    /// Time taken for the search in microseconds
    pub search_time_us: u64,

    /// Whether the search hit the result limit
    pub hit_limit: bool,

    /// Size of the search index in bytes
    pub index_size: usize,
}

/// Error types specific to search operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchError {
    /// Invalid regular expression
    InvalidRegex { pattern: String, error: String },

    /// Search index is corrupted or outdated
    IndexCorrupted,

    /// Feature not available (e.g., FST not compiled)
    FeatureNotAvailable { feature: String },

    /// Search operation timed out
    Timeout { duration_ms: u64 },
}

/// Main trait for document search functionality
pub trait DocumentSearch {
    /// Build or rebuild the search index for the document
    fn build_index(&mut self, document: &EditorDocument) -> Result<()>;

    /// Update the search index incrementally after document changes
    fn update_index(&mut self, document: &EditorDocument, changes: &[Range]) -> Result<()>;

    /// Search for a pattern in the document
    fn search(&self, pattern: &str, options: &SearchOptions) -> Result<Vec<SearchResult>>;

    /// Find and replace text in the document
    fn find_replace(
        &self,
        document: &mut EditorDocument,
        pattern: &str,
        replacement: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>>;

    /// Get search statistics
    fn stats(&self) -> SearchStats;

    /// Clear the search index to free memory
    fn clear_index(&mut self);
}

/// FST-based search implementation for high performance
#[cfg(feature = "search-index")]
#[derive(Debug)]
pub struct FstDocumentSearch {
    /// FST set for fast prefix/substring searches
    word_index: Option<Set<Vec<u8>>>,

    /// Mapping from words to their positions in the document
    word_positions: HashMap<String, Vec<Position>>,

    /// Last known document version for incremental updates
    document_version: u64,

    /// Cache of recent search results
    result_cache: HashMap<String, Vec<SearchResult>>,

    /// Maximum cache size
    max_cache_entries: usize,

    /// Search statistics
    stats: SearchStats,
}

#[cfg(feature = "search-index")]
impl FstDocumentSearch {
    /// Create a new FST-based search instance
    pub fn new() -> Self {
        Self {
            word_index: None,
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
        }
    }

    /// Set maximum number of cached search results
    pub fn set_cache_size(&mut self, size: usize) {
        self.max_cache_entries = size;
        self.trim_cache();
    }

    /// Extract words from document text for indexing
    fn extract_words(&self, text: &str) -> Vec<(String, Vec<Position>)> {
        let mut words = HashMap::new();
        let mut word_start_byte = None;
        let mut current_word = String::new();

        for (byte_idx, ch) in text.char_indices() {
            if ch.is_alphanumeric() || ch == '_' {
                if word_start_byte.is_none() {
                    word_start_byte = Some(byte_idx);
                }
                current_word.push(ch);
            } else {
                if let Some(start_byte) = word_start_byte {
                    if !current_word.is_empty() {
                        words
                            .entry(current_word.to_lowercase())
                            .or_insert_with(Vec::new)
                            .push(Position::new(start_byte));
                    }
                }
                current_word.clear();
                word_start_byte = None;
            }
        }

        // Handle final word
        if let Some(start_byte) = word_start_byte {
            if !current_word.is_empty() {
                words
                    .entry(current_word.to_lowercase())
                    .or_insert_with(Vec::new)
                    .push(Position::new(start_byte));
            }
        }

        words.into_iter().collect()
    }

    /// Build FST from word list
    fn build_fst(&self, words: &[(String, Vec<Position>)]) -> Result<Set<Vec<u8>>> {
        let mut builder = SetBuilder::memory();
        let mut word_list: Vec<&String> = words.iter().map(|(word, _)| word).collect();
        word_list.sort();
        word_list.dedup();

        for word in word_list {
            builder.insert(word.as_bytes()).map_err(|e| {
                crate::core::EditorError::CommandFailed {
                    message: format!("FST build error: {e}"),
                }
            })?;
        }

        Ok(builder.into_set())
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
    fn cache_key(&self, pattern: &str, options: &SearchOptions) -> String {
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

#[cfg(feature = "search-index")]
impl Default for FstDocumentSearch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "search-index")]
impl DocumentSearch for FstDocumentSearch {
    fn build_index(&mut self, document: &EditorDocument) -> Result<()> {
        let _start_time = std::time::Instant::now();

        let text = document.text();
        let words = self.extract_words(&text);

        // Build FST index
        let fst = self.build_fst(&words)?;
        self.word_index = Some(fst);

        // Store word positions
        self.word_positions.clear();
        for (word, positions) in words {
            self.word_positions.insert(word, positions);
        }

        // Update statistics
        self.stats.index_size = self.word_positions.len() * 50; // Rough estimate
        self.document_version += 1; // Simple increment for cache invalidation

        // Clear cache since index changed
        self.result_cache.clear();

        Ok(())
    }

    fn update_index(&mut self, document: &EditorDocument, _changes: &[Range]) -> Result<()> {
        // For now, just rebuild the entire index
        // TODO: Implement true incremental updates
        self.build_index(document)
    }

    fn search(&self, pattern: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        let _start_time = std::time::Instant::now();

        // Check cache first
        let cache_key = self.cache_key(pattern, options);
        if let Some(cached_results) = self.result_cache.get(&cache_key) {
            return Ok(cached_results.clone());
        }

        let results = if let Some(ref fst) = self.word_index {
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
                            text: pattern.to_string(),
                            context: format!("Offset {}", pos.offset),
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
            // Fallback to simple search
            Vec::new() // Would need document reference for simple search
        };

        // Update statistics (would need mutable access in real implementation)
        // self.stats.match_count = results.len();
        // self.stats.search_time_us = search_time;
        // self.stats.hit_limit = options.max_results > 0 && results.len() >= options.max_results;

        Ok(results)
    }

    fn find_replace(
        &self,
        document: &mut EditorDocument,
        pattern: &str,
        replacement: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
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
        self.word_index = None;
        self.word_positions.clear();
        self.result_cache.clear();
        self.stats.index_size = 0;
    }
}

/// Simple search implementation for when FST is not available
#[derive(Debug)]
pub struct SimpleDocumentSearch {
    /// Cached document text for search operations
    cached_text: String,

    /// Last known document version
    document_version: u64,

    /// Search statistics
    stats: SearchStats,
}

impl SimpleDocumentSearch {
    /// Create a new simple search instance
    pub fn new() -> Self {
        Self {
            cached_text: String::new(),
            document_version: 0,
            stats: SearchStats {
                match_count: 0,
                search_time_us: 0,
                hit_limit: false,
                index_size: 0,
            },
        }
    }
}

impl Default for SimpleDocumentSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentSearch for SimpleDocumentSearch {
    fn build_index(&mut self, document: &EditorDocument) -> Result<()> {
        self.cached_text = document.text();
        self.document_version += 1; // Simple increment for cache invalidation
        self.stats.index_size = self.cached_text.len();
        Ok(())
    }

    fn update_index(&mut self, document: &EditorDocument, _changes: &[Range]) -> Result<()> {
        self.build_index(document)
    }

    fn search(&self, pattern: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        let _start_time = std::time::Instant::now();

        let mut results = Vec::new();
        let search_pattern = if options.case_sensitive {
            pattern.to_string()
        } else {
            pattern.to_lowercase()
        };

        let search_text = if options.case_sensitive {
            self.cached_text.clone()
        } else {
            self.cached_text.to_lowercase()
        };

        let mut start_idx = 0;
        while let Some(match_idx) = search_text[start_idx..].find(&search_pattern) {
            let absolute_idx = start_idx + match_idx;
            let match_end = absolute_idx + search_pattern.len();

            // Calculate line and column
            let mut current_line = 0;
            let mut line_start_idx = 0;

            for (i, ch) in self.cached_text.char_indices() {
                if i >= absolute_idx {
                    break;
                }
                if ch == '\n' {
                    current_line += 1;
                    line_start_idx = i + 1;
                }
            }

            let column = absolute_idx - line_start_idx;

            // Extract context (line containing the match)
            let line_end = self.cached_text[line_start_idx..]
                .find('\n')
                .map(|pos| line_start_idx + pos)
                .unwrap_or(self.cached_text.len());
            let context = self.cached_text[line_start_idx..line_end].to_string();

            results.push(SearchResult {
                start: Position::new(absolute_idx),
                end: Position::new(match_end),
                text: self.cached_text[absolute_idx..match_end].to_string(),
                context,
                line: current_line,
                column,
            });

            if options.max_results > 0 && results.len() >= options.max_results {
                break;
            }

            start_idx = absolute_idx + 1;
        }

        Ok(results)
    }

    fn find_replace(
        &self,
        document: &mut EditorDocument,
        pattern: &str,
        replacement: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
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
        self.cached_text.clear();
        self.stats.index_size = 0;
    }
}

/// Factory function to create the best available search implementation
pub fn create_search() -> Box<dyn DocumentSearch> {
    #[cfg(feature = "search-index")]
    {
        Box::new(FstDocumentSearch::new())
    }

    #[cfg(not(feature = "search-index"))]
    {
        Box::new(SimpleDocumentSearch::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_options_default() {
        let options = SearchOptions::default();
        assert!(!options.case_sensitive);
        assert!(!options.whole_words);
        assert_eq!(options.max_results, 100);
        assert!(!options.use_regex);
        assert_eq!(options.scope, SearchScope::All);
    }

    #[test]
    fn search_result_creation() {
        let result = SearchResult {
            start: Position::new(0),
            end: Position::new(5),
            text: "hello".to_string(),
            context: "hello world".to_string(),
            line: 0,
            column: 0,
        };

        assert_eq!(result.text, "hello");
        assert_eq!(result.line, 0);
        assert_eq!(result.column, 0);
    }

    #[test]
    fn simple_search_creation() {
        let search = SimpleDocumentSearch::new();
        assert_eq!(search.stats.index_size, 0);
        assert_eq!(search.document_version, 0);
    }

    #[test]
    fn search_scope_variants() {
        let scope_all = SearchScope::All;
        let scope_lines = SearchScope::Lines { start: 0, end: 10 };
        let scope_sections = SearchScope::Sections(vec!["Events".to_string()]);

        assert_eq!(scope_all, SearchScope::All);
        assert!(matches!(scope_lines, SearchScope::Lines { .. }));
        assert!(matches!(scope_sections, SearchScope::Sections(_)));
    }

    #[test]
    #[cfg(feature = "search-index")]
    fn fst_search_creation() {
        let search = FstDocumentSearch::new();
        assert!(search.word_index.is_none());
        assert_eq!(search.word_positions.len(), 0);
        assert_eq!(search.max_cache_entries, 100);
    }

    #[test]
    fn create_search_factory() {
        let search = create_search();
        let stats = search.stats();
        assert_eq!(stats.match_count, 0);
        assert_eq!(stats.search_time_us, 0);
        assert!(!stats.hit_limit);
    }
}
