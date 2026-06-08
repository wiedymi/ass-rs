//! Parser state definition and constructors.
//!
//! Defines the internal [`Parser`] struct that accumulates sections, issues,
//! and format metadata while coordinating ASS section parsing, along with its
//! construction entry points.

use crate::{
    parser::{ast::Section, errors::ParseIssue},
    ScriptVersion,
};
use alloc::vec::Vec;

#[cfg(feature = "plugins")]
use crate::plugin::ExtensionRegistry;

/// Internal parser state for coordinating section parsing
pub(in crate::parser) struct Parser<'a> {
    /// Source text being parsed
    pub(super) source: &'a str,
    /// Current byte position in source
    pub(super) position: usize,
    /// Current line number for error reporting
    pub(super) line: usize,
    /// Detected script version
    pub(super) version: ScriptVersion,
    /// Parsed sections accumulated so far
    pub(super) sections: Vec<Section<'a>>,
    /// Parse issues and warnings
    pub(super) issues: Vec<ParseIssue>,
    /// Format fields for [V4+ Styles] section
    pub(super) styles_format: Option<Vec<&'a str>>,
    /// Format fields for `[Events\]` section
    pub(super) events_format: Option<Vec<&'a str>>,
    /// Extension registry for custom tag handlers and section processors
    #[cfg(feature = "plugins")]
    pub(super) registry: Option<&'a ExtensionRegistry>,
}

impl<'a> Parser<'a> {
    /// Create new parser for source text
    pub const fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
            line: 1,
            version: ScriptVersion::AssV4, // Default, updated when ScriptType found
            sections: Vec::new(),
            issues: Vec::new(),
            styles_format: None,
            events_format: None,
            #[cfg(feature = "plugins")]
            registry: None,
        }
    }

    /// Create new parser with extension registry
    #[cfg(feature = "plugins")]
    pub const fn new_with_registry(
        source: &'a str,
        registry: Option<&'a ExtensionRegistry>,
    ) -> Self {
        Self {
            source,
            position: 0,
            line: 1,
            version: ScriptVersion::AssV4, // Default, updated when ScriptType found
            sections: Vec::new(),
            issues: Vec::new(),
            styles_format: None,
            events_format: None,
            registry,
        }
    }
}
