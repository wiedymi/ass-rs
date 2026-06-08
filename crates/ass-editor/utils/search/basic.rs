//! Basic (non-indexed) substring search
//!
//! Implements the fallback text search used by
//! [`DocumentSearchImpl`](super::DocumentSearchImpl) when no FST index is
//! available or the `search-index` feature is disabled.

use super::engine::DocumentSearchImpl;
use super::options::{SearchOptions, SearchResult, SearchScope};
use crate::core::{Position, Result};

#[cfg(feature = "std")]
use std::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::borrow::Cow;

#[cfg(not(feature = "std"))]
use alloc::{string::ToString, vec::Vec};

impl DocumentSearchImpl {
    /// Perform basic text search without FST indexing
    pub(super) fn basic_search(
        &self,
        pattern: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult<'static>>> {
        let mut results = Vec::new();
        let search_pattern = if options.case_sensitive {
            pattern.to_string()
        } else {
            pattern.to_lowercase()
        };

        let search_text = if options.case_sensitive {
            Cow::Borrowed(&self.cached_text)
        } else {
            Cow::Owned(self.cached_text.to_lowercase())
        };

        // Apply search scope
        let (search_start, search_end) = match &options.scope {
            SearchScope::All => (0, search_text.len()),
            SearchScope::Range(range) => {
                (range.start.offset, range.end.offset.min(search_text.len()))
            }
            SearchScope::Lines { start, end } => {
                // Calculate byte offsets for line range
                let mut line_num = 0;
                let mut start_offset = 0;
                let mut end_offset = search_text.len();

                for (i, ch) in search_text.char_indices() {
                    if line_num == *start && start_offset == 0 {
                        start_offset = i;
                    }
                    if ch == '\n' {
                        line_num += 1;
                        if line_num > *end {
                            end_offset = i;
                            break;
                        }
                    }
                }
                (start_offset, end_offset)
            }
            SearchScope::Sections(_) => {
                // For sections, would need to parse and identify section boundaries
                // For now, search entire document
                (0, search_text.len())
            }
        };

        let search_region = &search_text[search_start..search_end];
        let mut region_offset = 0;

        while let Some(match_idx) = search_region[region_offset..].find(&search_pattern) {
            let absolute_idx = search_start + region_offset + match_idx;
            let match_end = absolute_idx + search_pattern.len();

            // Calculate line and column
            let mut line = 0;
            let mut line_start = 0;

            for (i, ch) in self.cached_text[..absolute_idx].char_indices() {
                if ch == '\n' {
                    line += 1;
                    line_start = i + 1;
                }
            }

            let column = absolute_idx - line_start;

            // Extract context (line containing the match)
            let context_start = self.cached_text[..absolute_idx]
                .rfind('\n')
                .map(|i| i + 1)
                .unwrap_or(0);
            let context_end = self.cached_text[absolute_idx..]
                .find('\n')
                .map(|i| absolute_idx + i)
                .unwrap_or(self.cached_text.len());
            let context = self.cached_text[context_start..context_end].to_string();

            results.push(SearchResult {
                start: Position::new(absolute_idx),
                end: Position::new(match_end),
                text: Cow::Owned(self.cached_text[absolute_idx..match_end].to_string()),
                context: Cow::Owned(context),
                line,
                column,
            });

            if options.max_results > 0 && results.len() >= options.max_results {
                break;
            }

            region_offset += match_idx + 1;
        }

        Ok(results)
    }
}
