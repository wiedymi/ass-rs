//! Aegisub-specific section processors for ASS compatibility
//!
//! Implements section processors for Aegisub-specific sections that extend
//! the standard ASS format. These processors handle project metadata and
//! additional data storage used by the Aegisub subtitle editor.
//!
//! # Supported Sections
//!
//! - `[Aegisub Project]`: Project-specific metadata and settings
//! - `[Aegisub Extradata]`: Additional data storage for extended functionality
//!
//! # Performance
//!
//! - Zero allocations for validation
//! - O(n) processing where n = number of lines
//! - Minimal memory footprint per processor

mod processors;

#[cfg(test)]
mod tests;

pub use processors::{AegisubExtradataProcessor, AegisubProjectProcessor};

use crate::plugin::SectionProcessor;

/// Create all Aegisub section processors
///
/// Returns a vector of boxed section processors for all Aegisub-specific sections.
/// Useful for bulk registration with the extension registry.
///
/// # Example
///
/// ```rust
/// use ass_core::plugin::{ExtensionRegistry, sections::aegisub::create_aegisub_processors};
///
/// let mut registry = ExtensionRegistry::new();
/// for processor in create_aegisub_processors() {
///     registry.register_section_processor(processor).unwrap();
/// }
/// ```
#[must_use]
pub fn create_aegisub_processors() -> alloc::vec::Vec<alloc::boxed::Box<dyn SectionProcessor>> {
    alloc::vec![
        alloc::boxed::Box::new(AegisubProjectProcessor),
        alloc::boxed::Box::new(AegisubExtradataProcessor),
    ]
}
