//! Override tag parsing and complexity analysis
//!
//! Provides efficient parsing of ASS override tags within dialogue text blocks.
//! Handles malformed syntax gracefully with diagnostic collection and uses
//! zero-copy references for optimal performance.
//!
//! # Features
//!
//! - Zero-copy tag argument parsing via string slices
//! - Robust error recovery for malformed tag syntax
//! - Complexity scoring for rendering optimization
//! - Diagnostic collection for parser feedback
//! - Unicode-aware character handling
//!
//! # Performance
//!
//! - Target: <0.1ms per tag block
//! - Memory: Zero allocations via borrowed references
//! - Complexity: O(n) where n = character count

mod complexity;
mod parser;
mod types;

#[cfg(feature = "plugins")]
mod registry_parser;

#[cfg(test)]
mod complexity_tests;
#[cfg(test)]
mod parser_tests;

pub use complexity::calculate_tag_complexity;
pub use parser::parse_override_block;
pub use types::{DiagnosticKind, OverrideTag, TagDiagnostic};

#[cfg(feature = "plugins")]
pub use registry_parser::parse_override_block_with_registry;
