//! `Script` container definition and fundamental accessors.
//!
//! Holds the zero-copy [`Script`] struct together with its version, section,
//! issue, format, and source accessors plus the internal `from_parts`
//! constructor. Mutation, parsing, and serialization live in sibling modules.

use alloc::vec::Vec;

use crate::parser::ast::{Section, SectionType};
use crate::parser::errors::ParseIssue;
use crate::ScriptVersion;

use super::types::ChangeTracker;

/// Main ASS script container with zero-copy lifetime-generic design
///
/// Uses `&'a str` spans throughout the AST to avoid allocations during parsing.
/// Thread-safe via immutable design after construction.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Script<'a> {
    /// Input source text for span validation
    pub(super) source: &'a str,

    /// Script version detected from headers
    pub(super) version: ScriptVersion,

    /// Parsed sections in document order
    pub(super) sections: Vec<Section<'a>>,

    /// Parse warnings and recoverable errors
    ///
    /// Transient parse diagnostics are not part of the canonical script
    /// content and are skipped during (de)serialization; re-parse the
    /// source to recompute them.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(super) issues: Vec<ParseIssue>,

    /// Format fields for [V4+ Styles] section
    pub(super) styles_format: Option<Vec<&'a str>>,

    /// Format fields for `[Events\]` section
    pub(super) events_format: Option<Vec<&'a str>>,

    /// Change tracker for incremental updates
    ///
    /// Internal incremental-edit state; reset to default on deserialization.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub(super) change_tracker: ChangeTracker<'a>,
}

impl<'a> Script<'a> {
    /// Get script version detected during parsing
    #[must_use]
    pub const fn version(&self) -> ScriptVersion {
        self.version
    }

    /// Get all parsed sections in document order
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn sections(&self) -> &[Section<'a>] {
        &self.sections
    }

    /// Get parse issues (warnings, recoverable errors)
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn issues(&self) -> &[ParseIssue] {
        &self.issues
    }

    /// Get source text that spans reference
    #[must_use]
    pub const fn source(&self) -> &'a str {
        self.source
    }

    /// Get format fields for [V4+ Styles] section
    #[must_use]
    pub fn styles_format(&self) -> Option<&[&'a str]> {
        self.styles_format.as_deref()
    }

    /// Get format fields for `[Events\]` section
    #[must_use]
    pub fn events_format(&self) -> Option<&[&'a str]> {
        self.events_format.as_deref()
    }

    /// Find section by type
    #[must_use]
    pub fn find_section(&self, section_type: SectionType) -> Option<&Section<'a>> {
        self.sections
            .iter()
            .find(|s| s.section_type() == section_type)
    }

    /// Create script from parsed components (internal constructor)
    pub(in crate::parser) fn from_parts(
        source: &'a str,
        version: ScriptVersion,
        sections: Vec<Section<'a>>,
        issues: Vec<ParseIssue>,
        styles_format: Option<Vec<&'a str>>,
        events_format: Option<Vec<&'a str>>,
    ) -> Self {
        Self {
            source,
            version,
            sections,
            issues,
            styles_format,
            events_format,
            change_tracker: ChangeTracker::default(),
        }
    }
}
