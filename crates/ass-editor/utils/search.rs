//! Search functionality for ASS documents
//!
//! Provides the `DocumentSearch` trait for efficient text search with
//! FST-based indexing, regex support, and incremental updates. Targets
//! <10ms initial indexing and <1ms query response times.

use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;

#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};

#[cfg(not(feature = "std"))]
use hashbrown::HashMap;

#[cfg(feature = "std")]
use std::time::Instant;

#[cfg(not(feature = "std"))]
#[allow(dead_code)]
type Instant = u64;

#[cfg(feature = "search-index")]
use fst::{automaton, IntoStreamer, Set, SetBuilder, Streamer};

#[cfg(all(feature = "formats", feature = "std"))]
use regex::Regex;

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
pub struct SearchResult<'a> {
    /// Position where the match starts
    pub start: Position,

    /// Position where the match ends
    pub end: Position,

    /// The matched text
    pub text: Cow<'a, str>,

    /// Context around the match (for display purposes)
    pub context: Cow<'a, str>,

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

/// Document search implementation with optional FST indexing
#[derive(Debug)]
pub struct DocumentSearchImpl {
    /// FST set for fast prefix/substring searches (when feature enabled)
    #[cfg(feature = "search-index")]
    word_index: Option<Set<Vec<u8>>>,

    /// Mapping from words to their positions in the document
    #[cfg(feature = "search-index")]
    word_positions: HashMap<String, Vec<Position>>,

    /// Last known document version for incremental updates
    document_version: u64,

    /// Cache of recent search results (owned to persist across calls)
    result_cache: HashMap<String, Vec<SearchResult<'static>>>,

    /// Maximum cache size
    max_cache_entries: usize,

    /// Search statistics
    stats: SearchStats,

    /// Cached document text for regex and basic searches
    cached_text: String,
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

    /// Extract words from document text for indexing
    #[cfg(feature = "search-index")]
    fn extract_words<'a>(&self, text: &'a str) -> Vec<(Cow<'a, str>, Vec<Position>)> {
        let mut words: HashMap<Cow<'a, str>, Vec<Position>> = HashMap::new();
        let mut word_start_byte = None;
        let mut current_word = String::new();
        let mut word_start_idx = 0;

        for (byte_idx, ch) in text.char_indices() {
            if ch.is_alphanumeric() || ch == '_' {
                if word_start_byte.is_none() {
                    word_start_byte = Some(byte_idx);
                    word_start_idx = byte_idx;
                }
                current_word.push(ch);
            } else {
                if let Some(start_byte) = word_start_byte {
                    if !current_word.is_empty() {
                        // Use Cow to avoid allocation when possible
                        let word_slice = &text[word_start_idx..byte_idx];
                        let word_key = if word_slice.chars().all(|c| c.is_lowercase()) {
                            Cow::Borrowed(word_slice)
                        } else {
                            Cow::Owned(current_word.to_lowercase())
                        };

                        words
                            .entry(word_key)
                            .or_default()
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
                let word_slice = &text[word_start_idx..];
                let word_key = if word_slice.chars().all(|c| c.is_lowercase()) {
                    Cow::Borrowed(word_slice)
                } else {
                    Cow::Owned(current_word.to_lowercase())
                };

                words
                    .entry(word_key)
                    .or_default()
                    .push(Position::new(start_byte));
            }
        }

        words.into_iter().collect()
    }

    /// Build FST from word list
    #[cfg(feature = "search-index")]
    fn build_fst(&self, words: &[(Cow<str>, Vec<Position>)]) -> Result<Set<Vec<u8>>> {
        let mut builder = SetBuilder::memory();
        let mut word_list: Vec<&str> = words.iter().map(|(word, _)| word.as_ref()).collect();
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

    /// Perform basic text search without FST indexing
    fn basic_search(
        &self,
        pattern: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult<'static>>> {
        let mut results = Vec::new();
        let search_pattern = if options.case_sensitive {
            pattern.to_string()
        } else {
            pattern.to_lowercase()
        };

        let search_text = if options.case_sensitive {
            Cow::Borrowed(&self.cached_text)
        } else {
            Cow::Owned(self.cached_text.to_lowercase())
        };

        // Apply search scope
        let (search_start, search_end) = match &options.scope {
            SearchScope::All => (0, search_text.len()),
            SearchScope::Range(range) => {
                (range.start.offset, range.end.offset.min(search_text.len()))
            }
            SearchScope::Lines { start, end } => {
                // Calculate byte offsets for line range
                let mut line_num = 0;
                let mut start_offset = 0;
                let mut end_offset = search_text.len();

                for (i, ch) in search_text.char_indices() {
                    if line_num == *start && start_offset == 0 {
                        start_offset = i;
                    }
                    if ch == '\n' {
                        line_num += 1;
                        if line_num > *end {
                            end_offset = i;
                            break;
                        }
                    }
                }
                (start_offset, end_offset)
            }
            SearchScope::Sections(_) => {
                // For sections, would need to parse and identify section boundaries
                // For now, search entire document
                (0, search_text.len())
            }
        };

        let search_region = &search_text[search_start..search_end];
        let mut region_offset = 0;

        while let Some(match_idx) = search_region[region_offset..].find(&search_pattern) {
            let absolute_idx = search_start + region_offset + match_idx;
            let match_end = absolute_idx + search_pattern.len();

            // Calculate line and column
            let mut line = 0;
            let mut line_start = 0;

            for (i, ch) in self.cached_text[..absolute_idx].char_indices() {
                if ch == '\n' {
                    line += 1;
                    line_start = i + 1;
                }
            }

            let column = absolute_idx - line_start;

            // Extract context (line containing the match)
            let context_start = self.cached_text[..absolute_idx]
                .rfind('\n')
                .map(|i| i + 1)
                .unwrap_or(0);
            let context_end = self.cached_text[absolute_idx..]
                .find('\n')
                .map(|i| absolute_idx + i)
                .unwrap_or(self.cached_text.len());
            let context = self.cached_text[context_start..context_end].to_string();

            results.push(SearchResult {
                start: Position::new(absolute_idx),
                end: Position::new(match_end),
                text: Cow::Owned(self.cached_text[absolute_idx..match_end].to_string()),
                context: Cow::Owned(context),
                line,
                column,
            });

            if options.max_results > 0 && results.len() >= options.max_results {
                break;
            }

            region_offset += match_idx + 1;
        }

        Ok(results)
    }

    /// Perform regex-based search
    #[cfg(all(feature = "formats", feature = "std"))]
    fn regex_search(
        &self,
        pattern: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult<'static>>> {
        use crate::core::EditorError;

        // Compile regex
        let regex_pattern = if options.case_sensitive {
            Regex::new(pattern)
        } else {
            Regex::new(&format!("(?i){pattern}"))
        };

        let regex = regex_pattern.map_err(|e| EditorError::CommandFailed {
            message: format!("Invalid regex pattern: {e}"),
        })?;

        let mut results = Vec::new();
        let text = &self.cached_text;

        // Apply search scope
        let (search_start, search_end) = match &options.scope {
            SearchScope::All => (0, text.len()),
            SearchScope::Range(range) => (range.start.offset, range.end.offset.min(text.len())),
            SearchScope::Lines { start, end } => {
                // Calculate byte offsets for line range
                let mut line_num = 0;
                let mut start_offset = 0;
                let mut end_offset = text.len();

                for (i, ch) in text.char_indices() {
                    if line_num == *start && start_offset == 0 {
                        start_offset = i;
                    }
                    if ch == '\n' {
                        line_num += 1;
                        if line_num > *end {
                            end_offset = i;
                            break;
                        }
                    }
                }
                (start_offset, end_offset)
            }
            SearchScope::Sections(_sections) => {
                // For sections, would need to parse and identify section boundaries
                // For now, search entire document
                (0, text.len())
            }
        };

        let search_text = &text[search_start..search_end];

        // Find all matches
        for mat in regex.find_iter(search_text) {
            let match_start = search_start + mat.start();
            let match_end = search_start + mat.end();

            // Calculate line and column
            let mut line = 0;
            let mut line_start = 0;
            for (i, ch) in text[..match_start].char_indices() {
                if ch == '\n' {
                    line += 1;
                    line_start = i + 1;
                }
            }
            let column = match_start - line_start;

            // Extract context (line containing the match)
            let context_start = text[..match_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let context_end = text[match_start..]
                .find('\n')
                .map(|i| match_start + i)
                .unwrap_or(text.len());
            let context = text[context_start..context_end].to_string();

            results.push(SearchResult {
                start: Position::new(match_start),
                end: Position::new(match_end),
                text: Cow::Owned(text[match_start..match_end].to_string()),
                context: Cow::Owned(context),
                line,
                column,
            });

            if options.max_results > 0 && results.len() >= options.max_results {
                break;
            }
        }

        Ok(results)
    }
}

impl Default for DocumentSearchImpl {
    fn default() -> Self {
        Self::new()
    }
}

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
        if changes.is_empty() {
            return Ok(());
        }

        // Update cached text first
        self.cached_text = document.text();

        // For non-FST search, we're done after updating the text
        #[cfg(not(feature = "search-index"))]
        {
            self.document_version += 1;
            self.result_cache.clear();
            Ok(())
        }

        // For FST search, perform incremental updates
        #[cfg(feature = "search-index")]
        {
            // Calculate total change size
            let total_change_size: usize = changes
                .iter()
                .map(|r| r.end.offset.saturating_sub(r.start.offset))
                .sum();

            // If too many changes or large changes, rebuild entirely
            if changes.len() > 10 || total_change_size > 1000 {
                return self.build_index(document);
            }

            // Sort changes by position to process them in order
            let mut sorted_changes = changes.to_vec();
            sorted_changes.sort_by_key(|r| r.start.offset);

            // Calculate cumulative offset adjustments
            let mut offset_adjustments: Vec<(usize, isize)> = Vec::new();
            let mut cumulative_adjustment = 0isize;

            for change in &sorted_changes {
                let old_len = change.end.offset - change.start.offset;
                // Find the actual new text in this region
                let new_text_region = &self.cached_text[change
                    .start
                    .offset
                    .saturating_add_signed(cumulative_adjustment)..];

                // Estimate new length by finding next unchanged content
                let new_len = if let Some(next_change) = sorted_changes
                    .iter()
                    .find(|c| c.start.offset > change.end.offset)
                {
                    let expected_gap = next_change.start.offset - change.end.offset;
                    new_text_region.len().min(expected_gap)
                } else {
                    // No more changes, estimate based on word boundaries
                    new_text_region
                        .find(['\n', '\r'])
                        .unwrap_or(new_text_region.len().min(old_len * 2))
                };

                let adjustment = new_len as isize - old_len as isize;
                offset_adjustments.push((change.start.offset, adjustment));
                cumulative_adjustment += adjustment;
            }

            // Update word positions
            let mut updated_positions = HashMap::new();

            for (word, positions) in &self.word_positions {
                let mut new_positions = Vec::new();

                for &pos in positions {
                    let mut adjusted_offset = pos.offset;
                    let mut should_remove = false;

                    // Check each change to see how it affects this position
                    for (i, change) in sorted_changes.iter().enumerate() {
                        if pos.offset >= change.start.offset && pos.offset < change.end.offset {
                            // Word was in changed region - remove it
                            should_remove = true;
                            break;
                        } else if pos.offset >= change.end.offset {
                            // Word is after this change - apply adjustment
                            let (_, adjustment) = offset_adjustments[i];
                            adjusted_offset = adjusted_offset.saturating_add_signed(adjustment);
                        }
                    }

                    if !should_remove {
                        new_positions.push(Position::new(adjusted_offset));
                    }
                }

                if !new_positions.is_empty() {
                    updated_positions.insert(word.clone(), new_positions);
                }
            }

            // Extract new words from changed regions
            for (i, change) in sorted_changes.iter().enumerate() {
                let start_adjustment: isize =
                    offset_adjustments[..i].iter().map(|(_, adj)| adj).sum();
                let adjusted_start = change.start.offset.saturating_add_signed(start_adjustment);

                // Get text around the change
                let extract_start = adjusted_start.saturating_sub(50);
                let extract_end = (adjusted_start + 100).min(self.cached_text.len());

                if extract_start < extract_end {
                    let region_text = &self.cached_text[extract_start..extract_end];
                    let new_words = self.extract_words(region_text);

                    // Add new words with adjusted positions
                    for (word, positions) in new_words {
                        let entry = updated_positions.entry(word.into_owned()).or_default();
                        for pos in positions {
                            let global_pos = Position::new(pos.offset + extract_start);
                            if !entry.contains(&global_pos) {
                                entry.push(global_pos);
                            }
                        }
                        entry.sort_by_key(|p| p.offset);
                    }
                }
            }

            // Update the word positions
            self.word_positions = updated_positions;

            // Rebuild FST with updated word list
            if !self.word_positions.is_empty() {
                let words: Vec<(Cow<str>, Vec<Position>)> = self
                    .word_positions
                    .iter()
                    .map(|(k, v)| (Cow::Borrowed(k.as_str()), v.clone()))
                    .collect();
                self.word_index = Some(self.build_fst(&words)?);
            } else {
                self.word_index = None;
            }

            // Update statistics
            self.stats.index_size = self.word_positions.len() * 50;
            self.document_version += 1;
            self.result_cache.clear();

            Ok(())
        }
    }

    fn search(&self, pattern: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
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

/// Factory function to create a document search implementation
pub fn create_search() -> Box<dyn DocumentSearch> {
    Box::new(DocumentSearchImpl::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::{borrow::Cow, string::ToString, vec};

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
            text: Cow::Borrowed("hello"),
            context: Cow::Borrowed("hello world"),
            line: 0,
            column: 0,
        };

        assert_eq!(result.text, "hello");
        assert_eq!(result.line, 0);
        assert_eq!(result.column, 0);
    }

    #[test]
    fn document_search_creation() {
        let search = DocumentSearchImpl::new();
        let stats = search.stats();
        assert_eq!(stats.index_size, 0);
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
    fn search_cache_settings() {
        let mut search = DocumentSearchImpl::new();
        assert_eq!(search.max_cache_entries, 100);
        search.set_cache_size(50);
        assert_eq!(search.max_cache_entries, 50);
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
        let mut search = DocumentSearchImpl::new();
        search.build_index(&doc).unwrap();

        // Search for "hello" - should find it
        let results = search.search("hello", &SearchOptions::default()).unwrap();
        assert_eq!(results.len(), 1);

        // Actually modify the document - change "Hello" to "Goodbye"
        let hello_pos = doc.text().find("Hello").unwrap();
        let change_range = Range::new(Position::new(hello_pos), Position::new(hello_pos + 5));

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

        let mut doc =
            EditorDocument::from_content("The quick brown fox jumps over the lazy dog.").unwrap();

        let mut search = DocumentSearchImpl::new();
        search.build_index(&doc).unwrap();

        // Verify initial search works
        let results = search.search("fox", &SearchOptions::default()).unwrap();
        assert_eq!(results.len(), 1);

        // Modify document
        let fox_pos = doc.text().find("fox").unwrap();
        let change_range = Range::new(Position::new(fox_pos), Position::new(fox_pos + 3));
        doc.replace(change_range, "cat").unwrap();

        // Update index with the change
        search.update_index(&doc, &[change_range]).unwrap();

        // Now search for "fox" should find nothing
        let results = search.search("fox", &SearchOptions::default()).unwrap();
        assert_eq!(results.len(), 0);

        // And "cat" should be found
        let results = search.search("cat", &SearchOptions::default()).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    #[cfg(all(feature = "formats", feature = "std"))]
    fn test_regex_search_basic() {
        use crate::core::EditorDocument;

        let doc = EditorDocument::from_content(
            "[Script Info]\nTitle: Test123\nPlayResX: 1920\nPlayResY: 1080",
        )
        .unwrap();

        let mut search = DocumentSearchImpl::new();
        search.build_index(&doc).unwrap();

        // Test basic regex pattern
        let options = SearchOptions {
            use_regex: true,
            ..Default::default()
        };

        // Search for numbers
        let results = search.search(r"\d+", &options).unwrap();
        assert_eq!(results.len(), 3); // 123, 1920, 1080

        // Search for "Play" followed by any word characters
        let results = search.search(r"Play\w+", &options).unwrap();
        assert_eq!(results.len(), 2); // PlayResX, PlayResY
    }

    #[test]
    #[cfg(all(feature = "formats", feature = "std"))]
    fn test_regex_search_case_insensitive() {
        use crate::core::EditorDocument;

        let doc = EditorDocument::from_content("Hello WORLD\nhello world\nHeLLo WoRlD").unwrap();

        let mut search = DocumentSearchImpl::new();
        search.build_index(&doc).unwrap();

        let options = SearchOptions {
            use_regex: true,
            case_sensitive: false,
            ..Default::default()
        };

        // Case-insensitive regex search
        let results = search.search(r"hello\s+world", &options).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    #[cfg(all(feature = "formats", feature = "std", feature = "search-index"))]
    fn test_fst_regex_search() {
        use crate::core::EditorDocument;

        let doc = EditorDocument::from_content(
            "[Events]\nDialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Test dialogue",
        )
        .unwrap();

        let mut search = DocumentSearchImpl::new();
        search.build_index(&doc).unwrap();

        let options = SearchOptions {
            use_regex: true,
            ..Default::default()
        };

        // Search for time codes pattern
        let results = search.search(r"\d:\d{2}:\d{2}\.\d{2}", &options).unwrap();
        assert_eq!(results.len(), 2); // Two time codes

        // Verify the matches are correct
        assert_eq!(results[0].text, "0:00:00.00");
        assert_eq!(results[1].text, "0:00:05.00");
    }

    #[test]
    #[cfg(all(feature = "formats", feature = "std"))]
    fn test_regex_search_with_scope() {
        use crate::core::EditorDocument;

        let doc =
            EditorDocument::from_content("Line 1: ABC\nLine 2: DEF\nLine 3: ABC\nLine 4: GHI")
                .unwrap();

        let mut search = DocumentSearchImpl::new();
        search.build_index(&doc).unwrap();

        let options = SearchOptions {
            use_regex: true,
            scope: SearchScope::Lines { start: 1, end: 2 },
            ..Default::default()
        };

        // Search for ABC in lines 1-2 only (0-based: lines at index 1 and 2)
        let results = search.search("ABC", &options).unwrap();
        assert_eq!(results.len(), 1); // ABC is on line 3 (index 2)

        // Search for DEF in lines 1-2
        let results = search.search("DEF", &options).unwrap();
        assert_eq!(results.len(), 1); // DEF is on line 2 (index 1)
    }

    #[test]
    #[cfg(not(all(feature = "formats", feature = "std")))]
    fn test_regex_search_feature_disabled() {
        use crate::core::EditorDocument;

        let doc = EditorDocument::from_content("Test content").unwrap();

        let mut search = DocumentSearchImpl::new();
        search.build_index(&doc).unwrap();

        let options = SearchOptions {
            use_regex: true,
            ..Default::default()
        };

        // Should return error when regex feature is not enabled
        let result = search.search("test", &options);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("Regex") || error_msg.contains("regex"),
            "Expected error to contain 'regex', but got: {}",
            error_msg
        );
    }
}
