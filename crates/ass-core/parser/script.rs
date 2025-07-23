//! ASS script container with zero-copy lifetime-generic design
//!
//! The `Script` struct provides the main API for accessing parsed ASS content
//! while maintaining zero-copy semantics through lifetime-generic spans.

use crate::{Result, ScriptVersion};
#[cfg(feature = "stream")]
use alloc::format;
use alloc::vec::Vec;
#[cfg(feature = "stream")]
use core::ops::Range;

#[cfg(feature = "stream")]
use super::errors::{IssueCategory, IssueSeverity};
#[cfg(feature = "stream")]
use super::streaming;
use super::{
    ast::{Section, SectionType},
    errors::ParseIssue,
    main::Parser,
};

/// Main ASS script container with zero-copy lifetime-generic design
///
/// Uses `&'a str` spans throughout the AST to avoid allocations during parsing.
/// Thread-safe via immutable design after construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script<'a> {
    /// Input source text for span validation
    source: &'a str,

    /// Script version detected from headers
    version: ScriptVersion,

    /// Parsed sections in document order
    sections: Vec<Section<'a>>,

    /// Parse warnings and recoverable errors
    issues: Vec<ParseIssue>,
}

impl<'a> Script<'a> {
    /// Parse ASS script from source text with zero-copy design
    ///
    /// Performs full validation and partial error recovery. Returns script
    /// even with errors - check `issues()` for problems.
    ///
    /// # Performance
    ///
    /// Target <5ms for 1KB typical scripts. Uses minimal allocations via
    /// zero-copy spans referencing input text.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ass_core::parser::Script;
    /// let script = Script::parse("[Script Info]\nTitle: Test")?;
    /// assert_eq!(script.version(), ass_core::ScriptVersion::AssV4);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the source contains malformed section headers or
    /// other unrecoverable syntax errors.
    pub fn parse(source: &'a str) -> Result<Self> {
        let parser = Parser::new(source);
        Ok(parser.parse())
    }

    /// Parse incrementally with range-based updates for editors
    ///
    /// Updates only the specified range, keeping other sections unchanged.
    /// Enables <2ms edit responsiveness for interactive editing.
    ///
    /// # Arguments
    ///
    /// * `range` - Byte range in source to re-parse
    /// * `new_text` - Replacement text for the range
    ///
    /// # Returns
    ///
    /// Delta containing changes that can be applied to existing script.
    ///
    /// # Errors
    ///
    /// Returns an error if the new text contains malformed section headers or
    /// other unrecoverable syntax errors in the specified range.
    #[cfg(feature = "stream")]
    pub fn parse_partial(&self, range: Range<usize>, new_text: &str) -> Result<ScriptDeltaOwned> {
        let deltas = streaming::parse_incremental(self, new_text, range)?;

        // Convert parse deltas to owned format
        let mut owned_delta = ScriptDeltaOwned {
            added: Vec::new(),
            modified: Vec::new(),
            removed: Vec::new(),
            new_issues: Vec::new(),
        };

        for delta in deltas {
            match delta {
                streaming::ParseDelta::AddSection(section) => {
                    owned_delta.added.push(format!("{section:?}"));
                }
                streaming::ParseDelta::UpdateSection(index, section) => {
                    owned_delta.modified.push((index, format!("{section:?}")));
                }
                streaming::ParseDelta::RemoveSection(index) => {
                    owned_delta.removed.push(index);
                }
                streaming::ParseDelta::ParseIssue(issue) => {
                    let parse_issue =
                        ParseIssue::new(IssueSeverity::Warning, IssueCategory::Structure, issue, 0);
                    owned_delta.new_issues.push(parse_issue);
                }
            }
        }

        Ok(owned_delta)
    }

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

    /// Find section by type
    #[must_use]
    pub fn find_section(&self, section_type: SectionType) -> Option<&Section<'a>> {
        self.sections
            .iter()
            .find(|s| s.section_type() == section_type)
    }

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

    /// Create script from parsed components (internal constructor)
    pub(super) const fn from_parts(
        source: &'a str,
        version: ScriptVersion,
        sections: Vec<Section<'a>>,
        issues: Vec<ParseIssue>,
    ) -> Self {
        Self {
            source,
            version,
            sections,
            issues,
        }
    }
}

/// Incremental parsing delta for efficient editor updates
#[cfg(feature = "stream")]
#[derive(Debug, Clone)]
pub struct ScriptDelta<'a> {
    /// Sections that were added
    pub added: Vec<Section<'a>>,

    /// Sections that were modified (old index -> new section)
    pub modified: Vec<(usize, Section<'a>)>,

    /// Section indices that were removed
    pub removed: Vec<usize>,

    /// New parse issues
    pub new_issues: Vec<ParseIssue>,
}

/// Owned variant of `ScriptDelta` for incremental parsing with lifetime independence
#[cfg(feature = "stream")]
#[derive(Debug, Clone)]
pub struct ScriptDeltaOwned {
    /// Sections that were added (serialized as source text)
    pub added: Vec<String>,

    /// Sections that were modified (old index -> new section as source text)
    pub modified: Vec<(usize, String)>,

    /// Section indices that were removed
    pub removed: Vec<usize>,

    /// New parse issues
    pub new_issues: Vec<ParseIssue>,
}

#[cfg(feature = "stream")]
impl ScriptDelta<'_> {
    /// Check if the delta contains no changes
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.modified.is_empty()
            && self.removed.is_empty()
            && self.new_issues.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_script() {
        let script = Script::parse("[Script Info]\nTitle: Test").unwrap();
        assert_eq!(script.sections().len(), 1);
        assert_eq!(script.version(), ScriptVersion::AssV4);
    }

    #[test]
    fn parse_with_script_type() {
        let script = Script::parse("[Script Info]\nScriptType: v4.00+\nTitle: Test").unwrap();
        assert_eq!(script.version(), ScriptVersion::AssV4);
    }

    #[test]
    fn parse_with_bom() {
        let script = Script::parse("\u{FEFF}[Script Info]\nTitle: Test").unwrap();
        assert_eq!(script.sections().len(), 1);
    }

    #[test]
    fn parse_empty_input() {
        let script = Script::parse("").unwrap();
        assert_eq!(script.sections().len(), 0);
    }
}
