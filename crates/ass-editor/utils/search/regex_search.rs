//! Regular-expression search
//!
//! Implements regex-based matching for
//! [`DocumentSearchImpl`](super::DocumentSearchImpl) when both the `formats`
//! and `std` features are enabled.

#[cfg(all(feature = "formats", feature = "std"))]
use super::engine::DocumentSearchImpl;

#[cfg(all(feature = "formats", feature = "std"))]
use super::options::{SearchOptions, SearchResult, SearchScope};

#[cfg(all(feature = "formats", feature = "std"))]
use crate::core::{Position, Result};

#[cfg(all(feature = "formats", feature = "std"))]
use std::borrow::Cow;

#[cfg(all(feature = "formats", feature = "std"))]
use regex::Regex;

#[cfg(all(feature = "formats", feature = "std"))]
impl DocumentSearchImpl {
    /// Perform regex-based search
    pub(super) fn regex_search(
        &self,
        pattern: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult<'static>>> {
        use crate::core::EditorError;

        // Compile regex
        let regex_pattern = if options.case_sensitive {
            Regex::new(pattern)
        } else {
            Regex::new(&format!("(?i){pattern}"))
        };

        let regex = regex_pattern.map_err(|e| EditorError::CommandFailed {
            message: format!("Invalid regex pattern: {e}"),
        })?;

        let mut results = Vec::new();
        let text = &self.cached_text;

        // Apply search scope
        let (search_start, search_end) = match &options.scope {
            SearchScope::All => (0, text.len()),
            SearchScope::Range(range) => (range.start.offset, range.end.offset.min(text.len())),
            SearchScope::Lines { start, end } => {
                // Calculate byte offsets for line range
                let mut line_num = 0;
                let mut start_offset = 0;
                let mut end_offset = text.len();

                for (i, ch) in text.char_indices() {
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
            SearchScope::Sections(_sections) => {
                // For sections, would need to parse and identify section boundaries
                // For now, search entire document
                (0, text.len())
            }
        };

        let search_text = &text[search_start..search_end];

        // Find all matches
        for mat in regex.find_iter(search_text) {
            let match_start = search_start + mat.start();
            let match_end = search_start + mat.end();

            // Calculate line and column
            let mut line = 0;
            let mut line_start = 0;
            for (i, ch) in text[..match_start].char_indices() {
                if ch == '\n' {
                    line += 1;
                    line_start = i + 1;
                }
            }
            let column = match_start - line_start;

            // Extract context (line containing the match)
            let context_start = text[..match_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let context_end = text[match_start..]
                .find('\n')
                .map(|i| match_start + i)
                .unwrap_or(text.len());
            let context = text[context_start..context_end].to_string();

            results.push(SearchResult {
                start: Position::new(match_start),
                end: Position::new(match_end),
                text: Cow::Owned(text[match_start..match_end].to_string()),
                context: Cow::Owned(context),
                line,
                column,
            });

            if options.max_results > 0 && results.len() >= options.max_results {
                break;
            }
        }

        Ok(results)
    }
}
