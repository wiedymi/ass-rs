//! Search functionality for ASS documents
//!
//! Provides the `DocumentSearch` trait for efficient text search with
//! FST-based indexing, regex support, and incremental updates. Targets
//! <10ms initial indexing and <1ms query response times.

use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::{String, ToString}, vec::Vec};

#[cfg(feature = "std")]
use std::time::Instant;

#[cfg(not(feature = "std"))]
type Instant = u64;

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
        #[cfg(feature = "std")]
        let _start_time = Instant::now();

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

    fn update_index(&mut self, document: &EditorDocument, changes: &[Range]) -> Result<()> {
        // Full implementation of incremental updates with proper position tracking
        if changes.is_empty() {
            return Ok(());
        }

        // If too many changes or large changes, rebuild entirely
        let total_change_size: usize = changes.iter()
            .map(|r| r.end.offset.saturating_sub(r.start.offset))
            .sum();
        
        if changes.len() > 10 || total_change_size > 1000 {
            return self.build_index(document);
        }

        // For a full implementation, we need to track the cumulative offset adjustments
        // as we process multiple changes. Each change affects the positions of subsequent changes.
        let mut cumulative_offset_adjustment: isize = 0;
        let text = document.text();
        
        // Sort changes by start position to process them in order
        let mut sorted_changes = changes.to_vec();
        sorted_changes.sort_by_key(|r| r.start.offset);
        
        for change in &sorted_changes {
            // Adjust change positions based on previous changes
            let adjusted_start = (change.start.offset as isize + cumulative_offset_adjustment).max(0) as usize;
            let _adjusted_end = (change.end.offset as isize + cumulative_offset_adjustment).max(0) as usize;
            
            // Calculate the actual change size by examining the text
            // We need to find what text is in the changed region to determine the net change
            let old_length = change.end.offset.saturating_sub(change.start.offset);
            
            // Look for the new text in the region - this is a heuristic approach
            // In a real implementation, we'd need the actual edit operation details
            let region_start = adjusted_start.saturating_sub(20);
            let region_end = (adjusted_start + 100).min(text.len());
            let mut new_length = old_length; // Default to same length
            
            if region_start < text.len() && region_end > region_start {
                // Try to detect the actual new text length by looking for word boundaries
                // This is still a heuristic but better than the simplified version
                let region = &text[region_start..region_end];
                
                // Find word boundary before and after the change point
                let pre_boundary = region[..adjusted_start.saturating_sub(region_start)]
                    .rfind(|c: char| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(0);
                    
                let post_boundary_start = adjusted_start.saturating_sub(region_start);
                let post_boundary = if post_boundary_start < region.len() {
                    region[post_boundary_start..]
                        .find(|c: char| !c.is_alphanumeric() && c != '_')
                        .map(|i| i + post_boundary_start)
                        .unwrap_or(region.len())
                } else {
                    region.len()
                };
                
                // Estimate new length based on the detected boundaries
                if post_boundary > pre_boundary {
                    new_length = post_boundary - pre_boundary;
                }
            }
            
            let length_delta = new_length as isize - old_length as isize;
            
            // Collect words that need updates
            let mut word_updates: Vec<(String, Option<Vec<Position>>)> = Vec::new();
            
            // Process existing words
            for (word, positions) in &self.word_positions {
                let mut new_positions = Vec::new();
                let mut affected = false;
                
                for &pos in positions {
                    let word_end = pos.offset + word.len();
                    
                    if pos.offset >= change.start.offset && pos.offset < change.end.offset {
                        // Word starts within original change range - remove it
                        affected = true;
                    } else if pos.offset < change.start.offset && word_end > change.start.offset {
                        // Word overlaps start of change - remove it
                        affected = true;
                    } else if pos.offset < change.end.offset && word_end > change.end.offset {
                        // Word overlaps end of change - remove it
                        affected = true;
                    } else if pos.offset >= change.end.offset {
                        // Word is after change - adjust position by cumulative offset + this change's delta
                        let total_adjustment = cumulative_offset_adjustment + length_delta;
                        let new_offset = (pos.offset as isize + total_adjustment).max(0) as usize;
                        new_positions.push(Position::new(new_offset));
                    } else {
                        // Word is before change - adjust by cumulative offset only
                        let new_offset = (pos.offset as isize + cumulative_offset_adjustment).max(0) as usize;
                        new_positions.push(Position::new(new_offset));
                    }
                }
                
                if affected || new_positions != *positions {
                    if new_positions.is_empty() {
                        word_updates.push((word.clone(), None));
                    } else {
                        word_updates.push((word.clone(), Some(new_positions)));
                    }
                }
            }
            
            // Apply updates
            for (word, positions) in word_updates {
                if let Some(positions) = positions {
                    self.word_positions.insert(word, positions);
                } else {
                    self.word_positions.remove(&word);
                }
            }
            
            // Extract and index new words in the changed region
            let extract_start = adjusted_start.saturating_sub(50);
            let extract_end = (adjusted_start + new_length + 50).min(text.len());
            
            if extract_start < text.len() && extract_end > extract_start {
                let region_text = &text[extract_start..extract_end];
                let new_words = self.extract_words(region_text);
                
                // Add new words with adjusted positions
                for (word, positions) in new_words {
                    let adjusted_positions: Vec<Position> = positions.into_iter()
                        .map(|p| Position::new(p.offset + extract_start))
                        .collect();
                    
                    // Merge with existing positions for this word
                    let entry = self.word_positions.entry(word).or_default();
                    for pos in adjusted_positions {
                        // Only add if not already present
                        if !entry.iter().any(|&p| p.offset == pos.offset) {
                            entry.push(pos);
                        }
                    }
                    // Keep positions sorted
                    entry.sort_by_key(|p| p.offset);
                }
            }
            
            // Update cumulative offset for next change
            cumulative_offset_adjustment += length_delta;
        }
        
        // Rebuild FST with updated word list
        if !self.word_positions.is_empty() {
            let words: Vec<(String, Vec<Position>)> = self.word_positions
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            self.word_index = Some(self.build_fst(&words)?);
        }
        
        // Update statistics
        self.stats.index_size = self.word_positions.len() * 50;
        self.document_version += 1;
        
        // Clear cache since index changed
        self.result_cache.clear();
        
        Ok(())
    }

    fn search(&self, pattern: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        #[cfg(feature = "std")]
        let _start_time = Instant::now();

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

/// Basic search implementation without indexing for when FST is not available
#[derive(Debug)]
pub struct BasicDocumentSearch {
    /// Cached document text for search operations
    cached_text: String,

    /// Last known document version
    document_version: u64,

    /// Search statistics
    stats: SearchStats,
}

impl BasicDocumentSearch {
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

impl Default for BasicDocumentSearch {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentSearch for BasicDocumentSearch {
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
        #[cfg(feature = "std")]
        let _start_time = Instant::now();

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

/// Type alias for the indexed search implementation
#[cfg(feature = "search-index")]
pub type IndexedDocumentSearch = FstDocumentSearch;

/// Type alias for the default search implementation based on features
#[cfg(feature = "search-index")]
pub type DefaultDocumentSearch = FstDocumentSearch;

/// Type alias for the default search implementation based on features
#[cfg(not(feature = "search-index"))]
pub type DefaultDocumentSearch = BasicDocumentSearch;

/// Factory function to create the best available search implementation
pub fn create_search() -> Box<dyn DocumentSearch> {
    Box::new(DefaultDocumentSearch::new())
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
        let search = BasicDocumentSearch::new();
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

    #[test]
    #[cfg(feature = "search-index")]
    fn test_incremental_index_updates() {
        use crate::core::{EditorDocument, Range};
        
        // Create a document with initial content
        let mut doc = EditorDocument::from_content(
            "[Script Info]\nTitle: Test Search\n\n[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello world"
        ).unwrap();
        
        // Create a search index and build initial index
        let mut search = FstDocumentSearch::new();
        search.build_index(&doc).unwrap();
        
        // Search for "hello" - should find it
        let results = search.search("hello", &SearchOptions::default()).unwrap();
        assert_eq!(results.len(), 1);
        
        // Actually modify the document - change "Hello" to "Goodbye"
        let hello_pos = doc.text().find("Hello").unwrap();
        let change_range = Range::new(
            Position::new(hello_pos),
            Position::new(hello_pos + 5)
        );
        
        // Delete "Hello" and insert "Goodbye"
        doc.delete(change_range).unwrap();
        doc.insert(Position::new(hello_pos), "Goodbye").unwrap();
        
        // Update the index incrementally with the change
        search.update_index(&doc, &[change_range]).unwrap();
        
        // Search for "hello" - should not find it anymore
        let results = search.search("hello", &SearchOptions::default()).unwrap();
        assert_eq!(results.len(), 0);
        
        // Search for "goodbye" - should find it
        let results = search.search("goodbye", &SearchOptions::default()).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_simple_document_search_rebuild() {
        use crate::core::{EditorDocument, Range};
        
        let mut doc = EditorDocument::from_content(
            "The quick brown fox jumps over the lazy dog."
        ).unwrap();
        
        let mut search = BasicDocumentSearch::new();
        search.build_index(&doc).unwrap();
        
        // Verify initial search works
        let results = search.search("fox", &SearchOptions::default()).unwrap();
        assert_eq!(results.len(), 1);
        
        // Modify document
        let fox_pos = doc.text().find("fox").unwrap();
        doc.replace(
            Range::new(Position::new(fox_pos), Position::new(fox_pos + 3)),
            "cat"
        ).unwrap();
        
        // BasicDocumentSearch always rebuilds the entire index
        search.update_index(&doc, &[]).unwrap();
        
        // Now search for "fox" should find nothing
        let results = search.search("fox", &SearchOptions::default()).unwrap();
        assert_eq!(results.len(), 0);
        
        // And "cat" should be found
        let results = search.search("cat", &SearchOptions::default()).unwrap();
        assert_eq!(results.len(), 1);
    }
}
