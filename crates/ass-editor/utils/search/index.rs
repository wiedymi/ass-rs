//! FST word-index construction helpers
//!
//! Provides the word-extraction and FST-building routines used by the
//! [`DocumentSearchImpl`](super::DocumentSearchImpl) indexing paths when the
//! `search-index` feature is enabled.

use super::engine::DocumentSearchImpl;

#[cfg(feature = "search-index")]
use crate::core::{Position, Result};

#[cfg(all(feature = "search-index", feature = "std"))]
use std::borrow::Cow;

#[cfg(all(feature = "search-index", not(feature = "std")))]
use alloc::borrow::Cow;

#[cfg(all(feature = "search-index", feature = "std"))]
use std::collections::HashMap;

#[cfg(all(feature = "search-index", not(feature = "std")))]
use hashbrown::HashMap;

#[cfg(all(feature = "search-index", not(feature = "std")))]
use alloc::{format, string::String, vec::Vec};

#[cfg(feature = "search-index")]
use fst::{Set, SetBuilder};

impl DocumentSearchImpl {
    /// Extract words from document text for indexing
    #[cfg(feature = "search-index")]
    pub(super) fn extract_words<'a>(&self, text: &'a str) -> Vec<(Cow<'a, str>, Vec<Position>)> {
        let mut words: HashMap<Cow<'a, str>, Vec<Position>> = HashMap::new();
        let mut word_start_byte = None;
        let mut current_word = String::new();
        let mut word_start_idx = 0;

        for (byte_idx, ch) in text.char_indices() {
            if ch.is_alphanumeric() || ch == '_' {
                if word_start_byte.is_none() {
                    word_start_byte = Some(byte_idx);
                    word_start_idx = byte_idx;
                }
                current_word.push(ch);
            } else {
                if let Some(start_byte) = word_start_byte {
                    if !current_word.is_empty() {
                        // Use Cow to avoid allocation when possible
                        let word_slice = &text[word_start_idx..byte_idx];
                        let word_key = if word_slice.chars().all(|c| c.is_lowercase()) {
                            Cow::Borrowed(word_slice)
                        } else {
                            Cow::Owned(current_word.to_lowercase())
                        };

                        words
                            .entry(word_key)
                            .or_default()
                            .push(Position::new(start_byte));
                    }
                }
                current_word.clear();
                word_start_byte = None;
            }
        }

        // Handle final word
        if let Some(start_byte) = word_start_byte {
            if !current_word.is_empty() {
                let word_slice = &text[word_start_idx..];
                let word_key = if word_slice.chars().all(|c| c.is_lowercase()) {
                    Cow::Borrowed(word_slice)
                } else {
                    Cow::Owned(current_word.to_lowercase())
                };

                words
                    .entry(word_key)
                    .or_default()
                    .push(Position::new(start_byte));
            }
        }

        words.into_iter().collect()
    }

    /// Build FST from word list
    #[cfg(feature = "search-index")]
    pub(super) fn build_fst(&self, words: &[(Cow<str>, Vec<Position>)]) -> Result<Set<Vec<u8>>> {
        let mut builder = SetBuilder::memory();
        let mut word_list: Vec<&str> = words.iter().map(|(word, _)| word.as_ref()).collect();
        word_list.sort();
        word_list.dedup();

        for word in word_list {
            builder.insert(word.as_bytes()).map_err(|e| {
                crate::core::EditorError::CommandFailed {
                    message: format!("FST build error: {e}"),
                }
            })?;
        }

        Ok(builder.into_set())
    }
}
