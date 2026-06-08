//! Core extension traits and their result types.
//!
//! Defines [`TagHandler`] and [`SectionProcessor`] for processing custom ASS
//! override tags and sections, along with the [`TagResult`] and
//! [`SectionResult`] values they return.

use alloc::string::String;

/// Result of tag processing operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagResult {
    /// Tag was successfully processed
    Processed,
    /// Tag was ignored (not handled by this processor)
    Ignored,
    /// Tag processing failed with error message
    Failed(String),
}

/// Result of section processing operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionResult {
    /// Section was successfully processed
    Processed,
    /// Section was ignored (not handled by this processor)
    Ignored,
    /// Section processing failed with error message
    Failed(String),
}

/// Trait for handling custom ASS override tags
///
/// Implementors can process custom tags that extend standard ASS functionality.
/// Tag handlers are called during parsing when unknown tags are encountered.
pub trait TagHandler: Send + Sync {
    /// Unique name identifier for this tag handler
    fn name(&self) -> &'static str;

    /// Process a tag with its arguments
    ///
    /// # Arguments
    /// * `args` - Raw tag arguments as string slice
    ///
    /// # Returns
    /// * `TagResult::Processed` - Tag was handled successfully
    /// * `TagResult::Ignored` - Tag not recognized by this handler
    /// * `TagResult::Failed` - Error occurred during processing
    fn process(&self, args: &str) -> TagResult;

    /// Optional validation of tag arguments during parsing
    fn validate(&self, args: &str) -> bool {
        !args.is_empty()
    }
}

/// Trait for handling custom ASS sections
///
/// Implementors can process non-standard sections that extend ASS functionality.
/// Section processors are called when unknown section headers are encountered.
pub trait SectionProcessor: Send + Sync {
    /// Unique name identifier for this section processor
    fn name(&self) -> &'static str;

    /// Process section header and content lines
    ///
    /// # Arguments
    /// * `header` - Section header (e.g., "Aegisub Project")
    /// * `lines` - All lines belonging to this section
    ///
    /// # Returns
    /// * `SectionResult::Processed` - Section was handled successfully
    /// * `SectionResult::Ignored` - Section not recognized by this processor
    /// * `SectionResult::Failed` - Error occurred during processing
    fn process(&self, header: &str, lines: &[&str]) -> SectionResult;

    /// Optional validation of section format
    fn validate(&self, header: &str, lines: &[&str]) -> bool {
        !header.is_empty() && !lines.is_empty()
    }
}
