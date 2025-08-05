//! Utility modules for the ass-editor
//!
//! Contains helper functionality like validation, search, and other
//! utilities that support the main editor operations.

pub mod indexing;
pub mod search;
pub mod validator;

#[cfg(feature = "formats")]
pub mod formats;

// Re-export commonly used types
pub use validator::{
    LazyValidator, ValidationIssue, ValidationResult, ValidationSeverity, ValidatorConfig,
};

pub use indexing::{create_search_index, IndexEntry};
pub use search::{DocumentSearch, SearchOptions, SearchResult, SearchScope, SearchStats};

#[cfg(feature = "search-index")]
pub use indexing::FstSearchIndex;

#[cfg(not(feature = "search-index"))]
pub use indexing::LinearSearchIndex;

#[cfg(feature = "formats")]
pub use formats::{
    export_to_file, import_from_file, ConversionOptions, FormatConverter, FormatOptions,
    SubtitleFormat,
};
