//! Change-tracking controls and structural diffing.
//!
//! Exposes the enable/disable/query surface over the script's internal
//! [`ChangeTracker`](super::types::ChangeTracker) and the [`Script::diff`]
//! routine that reports section-level differences between two scripts.

use alloc::vec::Vec;

use super::types::Change;
use super::Script;

impl<'a> Script<'a> {
    /// Enable change tracking
    ///
    /// When enabled, all modifications to the script will be recorded
    /// in the change tracker for later analysis.
    pub fn enable_change_tracking(&mut self) {
        self.change_tracker.enable();
    }

    /// Disable change tracking
    ///
    /// When disabled, modifications will not be recorded.
    pub fn disable_change_tracking(&mut self) {
        self.change_tracker.disable();
    }

    /// Check if change tracking is enabled
    #[must_use]
    pub const fn is_change_tracking_enabled(&self) -> bool {
        self.change_tracker.is_enabled()
    }

    /// Get all recorded changes
    ///
    /// Returns a slice of all changes recorded since tracking was enabled
    /// or since the last clear operation.
    #[must_use]
    pub fn changes(&self) -> &[Change<'a>] {
        self.change_tracker.changes()
    }

    /// Clear all recorded changes
    ///
    /// Removes all changes from the tracker while keeping tracking enabled/disabled.
    pub fn clear_changes(&mut self) {
        self.change_tracker.clear();
    }

    /// Get the number of recorded changes
    #[must_use]
    pub fn change_count(&self) -> usize {
        self.change_tracker.len()
    }

    /// Compute the difference between this script and another
    ///
    /// Analyzes the differences between two scripts and returns a list of changes
    /// that would transform the other script into this one.
    ///
    /// # Arguments
    ///
    /// * `other` - The script to compare against
    ///
    /// # Returns
    ///
    /// A vector of changes representing the differences
    #[must_use]
    pub fn diff(&self, other: &Self) -> Vec<Change<'a>> {
        let mut changes = Vec::new();

        // Compare sections
        let max_sections = self.sections.len().max(other.sections.len());

        for i in 0..max_sections {
            match (self.sections.get(i), other.sections.get(i)) {
                (Some(self_section), Some(other_section)) => {
                    // Both scripts have this section - check if they're different
                    if self_section != other_section {
                        // For now, record as section removed and added
                        // In a more sophisticated implementation, we could diff the contents
                        changes.push(Change::SectionRemoved {
                            section_type: other_section.section_type(),
                            index: i,
                        });
                        changes.push(Change::SectionAdded {
                            section: self_section.clone(),
                            index: i,
                        });
                    }
                }
                (Some(self_section), None) => {
                    // Section exists in self but not in other - it was added
                    changes.push(Change::SectionAdded {
                        section: self_section.clone(),
                        index: i,
                    });
                }
                (None, Some(other_section)) => {
                    // Section exists in other but not in self - it was removed
                    changes.push(Change::SectionRemoved {
                        section_type: other_section.section_type(),
                        index: i,
                    });
                }
                (None, None) => {
                    // Should not happen
                    unreachable!("max_sections calculation error");
                }
            }
        }

        changes
    }
}
