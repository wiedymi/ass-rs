//! Script delta types and computation for streaming editor updates.
//!
//! Defines the borrowed [`ScriptDelta`] and owned [`ScriptDeltaOwned`] change
//! descriptions and the [`calculate_delta`] routine that compares two scripts
//! while ignoring span-only differences.

use alloc::{string::String, vec::Vec};

use crate::parser::ast::Section;
use crate::parser::errors::ParseIssue;

use super::delta_eq::sections_equal_ignoring_spans;
use super::Script;

/// Incremental parsing delta for efficient editor updates
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

/// Calculate differences between two Scripts
///
/// Analyzes the differences between old and new scripts and returns
/// a delta containing the minimal set of changes needed to transform
/// the old script into the new one.
///
/// # Arguments
///
/// * `old_script` - The original script
/// * `new_script` - The updated script
///
/// # Returns
///
/// A `ScriptDelta` describing the changes
#[must_use]
pub fn calculate_delta<'a>(old_script: &Script<'a>, new_script: &Script<'a>) -> ScriptDelta<'a> {
    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut removed = Vec::new();

    // Create maps for efficient lookup
    let old_sections: Vec<_> = old_script.sections().iter().collect();
    let new_sections: Vec<_> = new_script.sections().iter().collect();

    // Find modifications and removals
    for (idx, old_section) in old_sections.iter().enumerate() {
        let old_type = old_section.section_type();

        // Look for matching section in new script
        if let Some((_new_idx, new_section)) = new_sections
            .iter()
            .enumerate()
            .find(|(_, s)| s.section_type() == old_type)
        {
            // Check if content changed (ignoring spans)
            if !sections_equal_ignoring_spans(old_section, new_section) {
                modified.push((idx, (*new_section).clone()));
            }
        } else {
            // Section was removed
            removed.push(idx);
        }
    }

    // Find additions
    for new_section in &new_sections {
        let new_type = new_section.section_type();

        // Check if this type exists in old script
        if !old_sections.iter().any(|s| s.section_type() == new_type) {
            added.push((*new_section).clone());
        }
    }

    // Calculate new issues
    // For simplicity, just take all issues from the new script
    // In a more sophisticated implementation, we could diff the issues
    let new_issues: Vec<_> = new_script.issues().to_vec();

    ScriptDelta {
        added,
        modified,
        removed,
        new_issues,
    }
}

/// Owned variant of `ScriptDelta` for incremental parsing with lifetime independence
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
