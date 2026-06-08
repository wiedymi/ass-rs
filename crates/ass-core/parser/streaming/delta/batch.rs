//! Batch collection for parse deltas
//!
//! Defines [`DeltaBatch`], a container that aggregates multiple [`ParseDelta`]
//! values and exposes filtering, merging, and validation helpers.

use super::ParseDelta;
use alloc::vec::Vec;

/// Collection of parse deltas with batch operations
///
/// Provides utilities for working with multiple deltas efficiently,
/// including filtering, merging, and validation.
#[derive(Debug, Clone)]
pub struct DeltaBatch<'a> {
    /// Collection of deltas representing changes to the script
    deltas: Vec<ParseDelta<'a>>,
}

impl<'a> DeltaBatch<'a> {
    /// Create new empty delta batch
    #[must_use]
    pub const fn new() -> Self {
        Self { deltas: Vec::new() }
    }

    /// Create batch from existing deltas
    #[must_use]
    pub const fn from_deltas(deltas: Vec<ParseDelta<'a>>) -> Self {
        Self { deltas }
    }

    /// Add delta to batch
    pub fn push(&mut self, delta: ParseDelta<'a>) {
        self.deltas.push(delta);
    }

    /// Extend batch with multiple deltas
    pub fn extend(&mut self, other_deltas: impl IntoIterator<Item = ParseDelta<'a>>) {
        self.deltas.extend(other_deltas);
    }

    /// Get all deltas
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn deltas(&self) -> &[ParseDelta<'a>] {
        &self.deltas
    }

    /// Convert to vector of deltas
    #[must_use]
    pub fn into_deltas(self) -> Vec<ParseDelta<'a>> {
        self.deltas
    }

    /// Check if batch is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.deltas.is_empty()
    }

    /// Get number of deltas in batch
    #[must_use]
    pub fn len(&self) -> usize {
        self.deltas.len()
    }

    /// Filter deltas by predicate
    #[must_use]
    pub fn filter<F>(&self, predicate: F) -> Self
    where
        F: Fn(&ParseDelta<'a>) -> bool,
    {
        let filtered = self
            .deltas
            .iter()
            .filter(|d| predicate(d))
            .cloned()
            .collect();
        DeltaBatch::from_deltas(filtered)
    }

    /// Get only structural deltas (add/update/remove)
    #[must_use]
    pub fn structural_only(&self) -> Self {
        self.filter(ParseDelta::is_structural)
    }

    /// Get only error deltas
    #[must_use]
    pub fn errors_only(&self) -> Self {
        self.filter(ParseDelta::is_error)
    }

    /// Check if batch contains any errors
    pub fn has_errors(&self) -> bool {
        self.deltas.iter().any(ParseDelta::is_error)
    }
}

impl Default for DeltaBatch<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> FromIterator<ParseDelta<'a>> for DeltaBatch<'a> {
    fn from_iter<T: IntoIterator<Item = ParseDelta<'a>>>(iter: T) -> Self {
        Self::from_deltas(iter.into_iter().collect())
    }
}
