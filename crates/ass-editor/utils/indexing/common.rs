//! Shared search index types and helpers.
//!
//! Defines the [`IndexEntry`] record stored by every search index backend and
//! the FNV content-hash helper used for cache invalidation.

use crate::core::Position;

#[cfg(not(feature = "std"))]
use alloc::string::String;

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

/// Simple hash function for content change detection
pub(super) fn calculate_hash(content: &str) -> u64 {
    // Simple FNV hash - in production might use a proper hasher
    let mut hash = 0xcbf29ce484222325u64;
    for byte in content.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
