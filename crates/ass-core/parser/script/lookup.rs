//! Section lookup helpers for span validation and byte-range queries.
//!
//! Provides read-only navigation over a [`Script`]'s parsed sections: span
//! validation, range lookup by type, offset-to-section resolution, and bulk
//! boundary enumeration.

use alloc::vec::Vec;

use crate::parser::ast::{Section, SectionType};

use super::Script;

impl<'a> Script<'a> {
    /// Validate all spans reference source text correctly
    ///
    /// Debug helper to ensure zero-copy invariants are maintained.
    #[cfg(debug_assertions)]
    #[must_use]
    pub fn validate_spans(&self) -> bool {
        let source_ptr = self.source.as_ptr();
        let source_range = source_ptr as usize..source_ptr as usize + self.source.len();

        self.sections
            .iter()
            .all(|section| section.validate_spans(&source_range))
    }

    /// Get byte range for a section
    ///
    /// Returns the byte range (start..end) for the specified section type,
    /// or None if the section doesn't exist or has no span.
    #[must_use]
    pub fn section_range(&self, section_type: SectionType) -> Option<core::ops::Range<usize>> {
        self.find_section(section_type)?
            .span()
            .map(|s| s.start..s.end)
    }

    /// Find section containing the given byte offset
    ///
    /// Returns the section that contains the specified byte offset,
    /// or None if no section contains that offset.
    #[must_use]
    pub fn section_at_offset(&self, offset: usize) -> Option<&Section<'a>> {
        self.sections.iter().find(|s| {
            s.span()
                .is_some_and(|span| span.start <= offset && offset < span.end)
        })
    }

    /// Get all section boundaries for quick lookup
    ///
    /// Returns a vector of (`SectionType`, `Range`) pairs for all sections
    /// that have valid spans. Useful for building lookup tables or
    /// determining which sections need reparsing after edits.
    #[must_use]
    pub fn section_boundaries(&self) -> Vec<(SectionType, core::ops::Range<usize>)> {
        self.sections
            .iter()
            .filter_map(|s| {
                s.span()
                    .map(|span| (s.section_type(), span.start..span.end))
            })
            .collect()
    }
}
