//! Incremental search-index updates
//!
//! Implements the incremental re-indexing path for
//! [`DocumentSearchImpl`](super::DocumentSearchImpl) used when a document
//! changes after an initial index build.

use super::engine::DocumentSearchImpl;
use crate::core::{EditorDocument, Range, Result};

#[cfg(feature = "search-index")]
use super::traits::DocumentSearch;

#[cfg(feature = "search-index")]
use crate::core::Position;

#[cfg(all(feature = "search-index", feature = "std"))]
use std::borrow::Cow;

#[cfg(all(feature = "search-index", not(feature = "std")))]
use alloc::borrow::Cow;

#[cfg(all(feature = "search-index", feature = "std"))]
use std::collections::HashMap;

#[cfg(all(feature = "search-index", not(feature = "std")))]
use hashbrown::HashMap;

#[cfg(all(feature = "search-index", not(feature = "std")))]
use alloc::vec::Vec;

impl DocumentSearchImpl {
    pub(super) fn update_index_impl(
        &mut self,
        document: &EditorDocument,
        changes: &[Range],
    ) -> Result<()> {
        if changes.is_empty() {
            return Ok(());
        }

        // Update cached text first
        self.cached_text = document.text();

        // For non-FST search, we're done after updating the text
        #[cfg(not(feature = "search-index"))]
        {
            self.document_version += 1;
            self.result_cache.clear();
            Ok(())
        }

        // For FST search, perform incremental updates
        #[cfg(feature = "search-index")]
        {
            // Calculate total change size
            let total_change_size: usize = changes
                .iter()
                .map(|r| r.end.offset.saturating_sub(r.start.offset))
                .sum();

            // If too many changes or large changes, rebuild entirely
            if changes.len() > 10 || total_change_size > 1000 {
                return self.build_index(document);
            }

            // Sort changes by position to process them in order
            let mut sorted_changes = changes.to_vec();
            sorted_changes.sort_by_key(|r| r.start.offset);

            // Calculate cumulative offset adjustments
            let mut offset_adjustments: Vec<(usize, isize)> = Vec::new();
            let mut cumulative_adjustment = 0isize;

            for change in &sorted_changes {
                let old_len = change.end.offset - change.start.offset;
                // Find the actual new text in this region
                let new_text_region = &self.cached_text[change
                    .start
                    .offset
                    .saturating_add_signed(cumulative_adjustment)..];

                // Estimate new length by finding next unchanged content
                let new_len = if let Some(next_change) = sorted_changes
                    .iter()
                    .find(|c| c.start.offset > change.end.offset)
                {
                    let expected_gap = next_change.start.offset - change.end.offset;
                    new_text_region.len().min(expected_gap)
                } else {
                    // No more changes, estimate based on word boundaries
                    new_text_region
                        .find(['\n', '\r'])
                        .unwrap_or(new_text_region.len().min(old_len * 2))
                };

                let adjustment = new_len as isize - old_len as isize;
                offset_adjustments.push((change.start.offset, adjustment));
                cumulative_adjustment += adjustment;
            }

            // Update word positions
            let mut updated_positions = HashMap::new();

            for (word, positions) in &self.word_positions {
                let mut new_positions = Vec::new();

                for &pos in positions {
                    let mut adjusted_offset = pos.offset;
                    let mut should_remove = false;

                    // Check each change to see how it affects this position
                    for (i, change) in sorted_changes.iter().enumerate() {
                        if pos.offset >= change.start.offset && pos.offset < change.end.offset {
                            // Word was in changed region - remove it
                            should_remove = true;
                            break;
                        } else if pos.offset >= change.end.offset {
                            // Word is after this change - apply adjustment
                            let (_, adjustment) = offset_adjustments[i];
                            adjusted_offset = adjusted_offset.saturating_add_signed(adjustment);
                        }
                    }

                    if !should_remove {
                        new_positions.push(Position::new(adjusted_offset));
                    }
                }

                if !new_positions.is_empty() {
                    updated_positions.insert(word.clone(), new_positions);
                }
            }

            // Extract new words from changed regions
            for (i, change) in sorted_changes.iter().enumerate() {
                let start_adjustment: isize =
                    offset_adjustments[..i].iter().map(|(_, adj)| adj).sum();
                let adjusted_start = change.start.offset.saturating_add_signed(start_adjustment);

                // Get text around the change
                let extract_start = adjusted_start.saturating_sub(50);
                let extract_end = (adjusted_start + 100).min(self.cached_text.len());

                if extract_start < extract_end {
                    let region_text = &self.cached_text[extract_start..extract_end];
                    let new_words = self.extract_words(region_text);

                    // Add new words with adjusted positions
                    for (word, positions) in new_words {
                        let entry = updated_positions.entry(word.into_owned()).or_default();
                        for pos in positions {
                            let global_pos = Position::new(pos.offset + extract_start);
                            if !entry.contains(&global_pos) {
                                entry.push(global_pos);
                            }
                        }
                        entry.sort_by_key(|p| p.offset);
                    }
                }
            }

            // Update the word positions
            self.word_positions = updated_positions;

            // Rebuild FST with updated word list
            if !self.word_positions.is_empty() {
                let words: Vec<(Cow<str>, Vec<Position>)> = self
                    .word_positions
                    .iter()
                    .map(|(k, v)| (Cow::Borrowed(k.as_str()), v.clone()))
                    .collect();
                self.word_index = Some(self.build_fst(&words)?);
            } else {
                self.word_index = None;
            }

            // Update statistics
            self.stats.index_size = self.word_positions.len() * 50;
            self.document_version += 1;
            self.result_cache.clear();

            Ok(())
        }
    }
}
