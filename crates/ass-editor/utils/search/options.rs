//! Search option, result, statistics, and error types
//!
//! Defines the configuration and data types used by the
//! [`DocumentSearch`](super::DocumentSearch) trait for document queries.

use crate::core::{Position, Range};

#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

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
