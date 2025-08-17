//! FST-based search indexing for fast ASS content queries
//!
//! Provides trie-based indexing for regex and fuzzy search queries as specified
//! in the architecture (lines 142-143). Fallback to linear search for WASM.

use crate::core::{EditorDocument, Position, Range, Result};
#[cfg(feature = "search-index")]
use crate::utils::search::SearchScope;
use crate::utils::search::{DocumentSearch, SearchOptions, SearchResult, SearchStats};

#[cfg(feature = "search-index")]
use fst::{automaton, IntoStreamer, Set, SetBuilder, Streamer};

#[cfg(feature = "std")]
use std::{borrow::Cow, collections::HashMap};

#[cfg(not(feature = "std"))]
use alloc::{borrow::Cow, collections::BTreeMap as HashMap, string::String};

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, string::ToString, vec::Vec};

#[cfg(feature = "std")]
use std::time::Instant;

#[cfg(not(feature = "std"))]
type Instant = u64;

/// FST-based search index for high-performance queries
#[cfg(feature = "search-index")]
pub struct FstSearchIndex {
    /// FST set for fast prefix/fuzzy matching
    fst_set: Option<Set<Vec<u8>>>,

    /// Map from FST keys to document positions
    position_map: HashMap<String, Vec<IndexEntry>>,

    /// Last known document content hash for cache invalidation
    content_hash: u64,

    /// Index build timestamp for statistics
    build_time: Instant,

    /// Index size in bytes
    index_size: usize,
}

/// Linear search fallback for when FST is not available
pub struct LinearSearchIndex {
    /// Simple word -> positions mapping
    word_positions: HashMap<String, Vec<IndexEntry>>,

    /// Content hash for invalidation
    content_hash: u64,

    /// Build timestamp
    build_time: Instant,
}

/// Entry in the search index
#[derive(Debug, Clone)]
pub struct IndexEntry {
    /// Position in document
    pub position: Position,

    /// Context around the match
    pub context: String,

    /// Line number (0-based)
    pub line: usize,

    /// Column number (0-based)  
    pub column: usize,

    /// Section type (Events, Styles, etc.)
    pub section_type: Option<String>,
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

    /// Extract words and positions from document content
    fn extract_words(&self, content: &str) -> Vec<(String, IndexEntry)> {
        let mut words = Vec::new();
        let mut line_start = 0;
        let mut current_section = None;

        for (current_line, line) in content.lines().enumerate() {
            // Track current section
            if line.starts_with('[') && line.ends_with(']') {
                current_section = Some(line[1..line.len() - 1].to_string());
            }

            // Extract words from line
            let mut word_start = 0;
            let mut in_word = false;

            for (char_offset, ch) in line.char_indices() {
                if ch.is_alphanumeric() || ch == '_' {
                    if !in_word {
                        word_start = char_offset;
                        in_word = true;
                    }
                } else if in_word {
                    // End of word
                    let word = &line[word_start..char_offset];
                    if word.len() >= 2 {
                        // Index words with 2+ characters
                        let entry = IndexEntry {
                            position: Position::new(line_start + word_start),
                            context: line.to_string(),
                            line: current_line,
                            column: word_start,
                            section_type: current_section.clone(),
                        };
                        words.push((word.to_lowercase(), entry));
                    }
                    in_word = false;
                }
            }

            // Handle word at end of line
            if in_word {
                let word = &line[word_start..];
                if word.len() >= 2 {
                    let entry = IndexEntry {
                        position: Position::new(line_start + word_start),
                        context: line.to_string(),
                        line: current_line,
                        column: word_start,
                        section_type: current_section.clone(),
                    };
                    words.push((word.to_lowercase(), entry));
                }
            }

            line_start += line.len() + 1; // +1 for newline
        }

        words
    }

    /// Build FST from extracted words
    fn build_fst(&mut self, words: Vec<(String, IndexEntry)>) -> Result<()> {
        // Group entries by word
        let mut word_map: HashMap<String, Vec<IndexEntry>> = HashMap::new();
        for (word, entry) in words {
            word_map.entry(word).or_default().push(entry);
        }

        // Build FST set
        let mut set_builder = SetBuilder::memory();
        let mut keys: Vec<_> = word_map.keys().cloned().collect();
        keys.sort();

        for key in &keys {
            set_builder
                .insert(key)
                .map_err(|e| crate::EditorError::SearchIndexError {
                    message: format!("FST insert error: {e}"),
                })?;
        }

        let fst_bytes =
            set_builder
                .into_inner()
                .map_err(|e| crate::EditorError::SearchIndexError {
                    message: format!("FST build error: {e}"),
                })?;
        self.index_size = fst_bytes.len();
        self.fst_set =
            Some(
                Set::new(fst_bytes).map_err(|e| crate::EditorError::SearchIndexError {
                    message: format!("FST creation error: {e}"),
                })?,
            );
        self.position_map = word_map;

        Ok(())
    }
}

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

    fn search(&self, pattern: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
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
        sorted_results.sort_by(|a, b| b.start.offset.cmp(&a.start.offset));

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

#[cfg(feature = "search-index")]
impl FstSearchIndex {
    fn matches_scope(&self, entry: &IndexEntry, scope: &SearchScope) -> bool {
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

// Linear search fallback implementation
impl Default for LinearSearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl LinearSearchIndex {
    /// Create a new linear search index
    pub fn new() -> Self {
        Self {
            word_positions: HashMap::new(),
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
        }
    }
}

impl DocumentSearch for LinearSearchIndex {
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

        // Simple word extraction for linear search
        self.word_positions.clear();

        let mut line_start = 0;

        for (current_line, line) in content.lines().enumerate() {
            for (word_start, word) in line.split_whitespace().enumerate() {
                let entry = IndexEntry {
                    position: Position::new(line_start + word_start),
                    context: line.to_string(),
                    line: current_line,
                    column: word_start,
                    section_type: None,
                };
                self.word_positions
                    .entry(word.to_lowercase())
                    .or_default()
                    .push(entry);
            }
            line_start += line.len() + 1;
        }

        Ok(())
    }

    fn update_index(&mut self, document: &EditorDocument, _changes: &[Range]) -> Result<()> {
        self.build_index(document)
    }

    fn search(&self, pattern: &str, _options: &SearchOptions) -> Result<Vec<SearchResult<'_>>> {
        let query = pattern.to_lowercase();
        let mut results = Vec::new();

        // Simple linear search through indexed words
        for (word, entries) in &self.word_positions {
            if word.contains(&query) {
                for entry in entries {
                    results.push(SearchResult {
                        start: entry.position,
                        end: Position::new(entry.position.offset + pattern.len()),
                        text: Cow::Owned(pattern.to_string()),
                        context: Cow::Owned(entry.context.clone()),
                        line: entry.line,
                        column: entry.column,
                    });
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

        for result in &results {
            let range = Range::new(result.start, result.end);
            document.replace(range, replacement)?;
        }

        Ok(results)
    }

    fn stats(&self) -> SearchStats {
        SearchStats {
            match_count: self.word_positions.len(),
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
            index_size: self.word_positions.len() * 64, // Rough estimate
        }
    }

    fn clear_index(&mut self) {
        self.word_positions.clear();
    }
}

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

/// Simple hash function for content change detection
fn calculate_hash(content: &str) -> u64 {
    // Simple FNV hash - in production might use a proper hasher
    let mut hash = 0xcbf29ce484222325u64;
    for byte in content.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::search::SearchScope;
    #[cfg(not(feature = "std"))]
    use crate::EditorDocument;

    #[test]
    fn test_linear_search_index() {
        let content = r#"[Script Info]
Title: Test Movie
Author: Test Author

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:05.00,0:00:10.00,Default,John,0,0,0,,Hello world
Dialogue: 0,0:00:12.00,0:00:15.00,Default,Jane,0,0,0,,How are you"#;

        let document = EditorDocument::from_content(content).unwrap();
        let mut search_index = LinearSearchIndex::new();

        search_index.build_index(&document).unwrap();

        let results = search_index
            .search("Hello", &SearchOptions::default())
            .unwrap();
        assert!(!results.is_empty());

        let stats = search_index.stats();
        assert!(stats.match_count > 0);
    }

    #[cfg(feature = "search-index")]
    #[test]
    fn test_fst_search_index() {
        let content = r#"[Script Info]
Title: Test Movie

[Events]  
Dialogue: 0,0:00:05.00,0:00:10.00,Default,John,0,0,0,,Hello world"#;

        let document = EditorDocument::from_content(content).unwrap();
        let mut search_index = FstSearchIndex::new();

        search_index.build_index(&document).unwrap();

        let results = search_index
            .search("hello", &SearchOptions::default())
            .unwrap();
        assert!(!results.is_empty());

        let stats = search_index.stats();
        assert!(stats.index_size > 0);
    }

    #[test]
    fn test_search_factory() {
        let index = create_search_index();

        let content = "[Script Info]\nTitle: Test";
        let _document = EditorDocument::from_content(content).unwrap();

        // Just test that we can create and use the index
        let stats = index.stats();
        assert_eq!(stats.match_count, 0); // No index built yet
    }

    #[test]
    fn test_scope_filtering() {
        let content = r#"[Script Info]
Title: Test

[Events]
Dialogue: Hello world"#;

        let document = EditorDocument::from_content(content).unwrap();
        let mut index = LinearSearchIndex::new();
        index.build_index(&document).unwrap();

        // Test different scopes
        let line_scope = SearchOptions {
            scope: SearchScope::Lines { start: 0, end: 2 },
            ..Default::default()
        };

        let _results = index.search("Test", &line_scope).unwrap();
        // Should find results in the specified line range
    }
}
