//! Fluent API for ASS karaoke operations and the karaoke generator builder.

use super::karaoke_builders::{KaraokeAdjuster, KaraokeApplicator, KaraokeSplitter};
use crate::commands::{EditorCommand, GenerateKaraokeCommand, KaraokeType};
use crate::core::{EditorDocument, Position, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Fluent API for ASS karaoke operations
pub struct KaraokeOps<'a> {
    document: &'a mut EditorDocument,
    range: Option<Range>,
}

impl<'a> KaraokeOps<'a> {
    /// Create new karaoke operations
    pub(super) fn new(document: &'a mut EditorDocument) -> Self {
        Self {
            document,
            range: None,
        }
    }

    /// Set range for karaoke operations
    #[must_use]
    pub fn in_range(mut self, range: Range) -> Self {
        self.range = Some(range);
        self
    }

    /// Generate karaoke timing for text
    pub fn generate(self, default_duration: u32) -> KaraokeGenerator<'a> {
        let default_range = if self.range.is_none() {
            let doc_len = self.document.text().len();
            Range::new(Position::new(0), Position::new(doc_len))
        } else {
            Range::new(Position::new(0), Position::new(0)) // Placeholder, won't be used
        };

        KaraokeGenerator {
            document: self.document,
            range: self.range.unwrap_or(default_range),
            default_duration,
            karaoke_type: KaraokeType::Standard,
            auto_detect_syllables: true,
        }
    }

    /// Split existing karaoke timing
    pub fn split(self, split_positions: Vec<usize>) -> KaraokeSplitter<'a> {
        let default_range = if self.range.is_none() {
            let doc_len = self.document.text().len();
            Range::new(Position::new(0), Position::new(doc_len))
        } else {
            Range::new(Position::new(0), Position::new(0)) // Placeholder
        };

        KaraokeSplitter {
            document: self.document,
            range: self.range.unwrap_or(default_range),
            split_positions,
            new_duration: None,
        }
    }

    /// Adjust existing karaoke timing
    pub fn adjust(self) -> KaraokeAdjuster<'a> {
        let default_range = if self.range.is_none() {
            let doc_len = self.document.text().len();
            Range::new(Position::new(0), Position::new(doc_len))
        } else {
            Range::new(Position::new(0), Position::new(0)) // Placeholder
        };

        KaraokeAdjuster {
            document: self.document,
            range: self.range.unwrap_or(default_range),
        }
    }

    /// Apply karaoke template
    pub fn apply(self) -> KaraokeApplicator<'a> {
        let default_range = if self.range.is_none() {
            let doc_len = self.document.text().len();
            Range::new(Position::new(0), Position::new(doc_len))
        } else {
            Range::new(Position::new(0), Position::new(0)) // Placeholder
        };

        KaraokeApplicator {
            document: self.document,
            range: self.range.unwrap_or(default_range),
        }
    }
}

/// Karaoke generator builder
pub struct KaraokeGenerator<'a> {
    document: &'a mut EditorDocument,
    range: Range,
    default_duration: u32,
    karaoke_type: KaraokeType,
    auto_detect_syllables: bool,
}

impl<'a> KaraokeGenerator<'a> {
    /// Set karaoke type
    #[must_use]
    pub fn karaoke_type(mut self, karaoke_type: KaraokeType) -> Self {
        self.karaoke_type = karaoke_type;
        self
    }

    /// Use manual syllable splitting
    #[must_use]
    pub fn manual_syllables(mut self) -> Self {
        self.auto_detect_syllables = false;
        self
    }

    /// Execute the generation
    pub fn execute(self) -> Result<&'a mut EditorDocument> {
        let mut command = GenerateKaraokeCommand::new(self.range, self.default_duration)
            .karaoke_type(self.karaoke_type);

        if !self.auto_detect_syllables {
            command = command.manual_syllables();
        }

        command.execute(self.document)?;
        Ok(self.document)
    }
}
