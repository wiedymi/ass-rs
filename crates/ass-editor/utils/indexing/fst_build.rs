//! Word extraction and FST construction for [`FstSearchIndex`].

use super::common::IndexEntry;
use super::fst::FstSearchIndex;
use crate::core::{Position, Result};

use fst::{Set, SetBuilder};

use std::collections::HashMap;

#[cfg(feature = "search-index")]
impl FstSearchIndex {
    /// Extract words and positions from document content
    pub(super) fn extract_words(&self, content: &str) -> Vec<(String, IndexEntry)> {
        let mut words = Vec::new();
        let mut line_start = 0;
        let mut current_section = None;

        for (current_line, line) in content.lines().enumerate() {
            // Track current section
            if line.starts_with('[') && line.ends_with(']') {
                current_section = Some(line[1..line.len() - 1].to_string());
            }

            // Extract words from line
            let mut word_start = 0;
            let mut in_word = false;

            for (char_offset, ch) in line.char_indices() {
                if ch.is_alphanumeric() || ch == '_' {
                    if !in_word {
                        word_start = char_offset;
                        in_word = true;
                    }
                } else if in_word {
                    // End of word
                    let word = &line[word_start..char_offset];
                    if word.len() >= 2 {
                        // Index words with 2+ characters
                        let entry = IndexEntry {
                            position: Position::new(line_start + word_start),
                            context: line.to_string(),
                            line: current_line,
                            column: word_start,
                            section_type: current_section.clone(),
                        };
                        words.push((word.to_lowercase(), entry));
                    }
                    in_word = false;
                }
            }

            // Handle word at end of line
            if in_word {
                let word = &line[word_start..];
                if word.len() >= 2 {
                    let entry = IndexEntry {
                        position: Position::new(line_start + word_start),
                        context: line.to_string(),
                        line: current_line,
                        column: word_start,
                        section_type: current_section.clone(),
                    };
                    words.push((word.to_lowercase(), entry));
                }
            }

            line_start += line.len() + 1; // +1 for newline
        }

        words
    }

    /// Build FST from extracted words
    pub(super) fn build_fst(&mut self, words: Vec<(String, IndexEntry)>) -> Result<()> {
        // Group entries by word
        let mut word_map: HashMap<String, Vec<IndexEntry>> = HashMap::new();
        for (word, entry) in words {
            word_map.entry(word).or_default().push(entry);
        }

        // Build FST set
        let mut set_builder = SetBuilder::memory();
        let mut keys: Vec<_> = word_map.keys().cloned().collect();
        keys.sort();

        for key in &keys {
            set_builder
                .insert(key)
                .map_err(|e| crate::EditorError::SearchIndexError {
                    message: format!("FST insert error: {e}"),
                })?;
        }

        let fst_bytes =
            set_builder
                .into_inner()
                .map_err(|e| crate::EditorError::SearchIndexError {
                    message: format!("FST build error: {e}"),
                })?;
        self.index_size = fst_bytes.len();
        self.fst_set =
            Some(
                Set::new(fst_bytes).map_err(|e| crate::EditorError::SearchIndexError {
                    message: format!("FST creation error: {e}"),
                })?,
            );
        self.position_map = word_map;

        Ok(())
    }
}
