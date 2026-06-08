//! Owned result type produced when a streaming parse completes
//!
//! Defines [`StreamingResult`], the final output of [`super::StreamingParser`]
//! containing the parsed sections, detected script version, and any issues
//! encountered while streaming.

use crate::ScriptVersion;
use alloc::{string::String, vec::Vec};

/// Result of streaming parser containing owned sections
#[derive(Debug, Clone)]
pub struct StreamingResult {
    /// Parsed sections in document order (simplified)
    pub sections: Vec<String>,
    /// Script version detected from headers
    pub version: ScriptVersion,
    /// Parse warnings and recoverable errors
    pub issues: Vec<crate::parser::ParseIssue>,
}

impl StreamingResult {
    /// Get parsed sections (simplified)
    #[must_use]
    pub fn sections(&self) -> &[String] {
        &self.sections
    }

    /// Get detected script version
    #[must_use]
    pub const fn version(&self) -> ScriptVersion {
        self.version
    }

    /// Get parsing issues
    #[must_use]
    pub fn issues(&self) -> &[crate::parser::ParseIssue] {
        &self.issues
    }
}
