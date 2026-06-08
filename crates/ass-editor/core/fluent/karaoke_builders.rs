//! Karaoke splitter, adjuster, and applicator builders.
//!
//! These builders are constructed by [`super::KaraokeOps`], so their fields are
//! `pub(super)` to remain reachable from the karaoke operations entry point in the
//! sibling [`super::karaoke`] module.

use crate::commands::{
    AdjustKaraokeCommand, ApplyKaraokeCommand, EditorCommand, KaraokeType, SplitKaraokeCommand,
};
use crate::core::{EditorDocument, Range, Result};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Karaoke splitter builder
pub struct KaraokeSplitter<'a> {
    pub(super) document: &'a mut EditorDocument,
    pub(super) range: Range,
    pub(super) split_positions: Vec<usize>,
    pub(super) new_duration: Option<u32>,
}

impl<'a> KaraokeSplitter<'a> {
    /// Set new duration for split segments
    #[must_use]
    pub fn duration(mut self, duration: u32) -> Self {
        self.new_duration = Some(duration);
        self
    }

    /// Execute the split
    pub fn execute(self) -> Result<&'a mut EditorDocument> {
        let mut command = SplitKaraokeCommand::new(self.range, self.split_positions);

        if let Some(duration) = self.new_duration {
            command = command.duration(duration);
        }

        command.execute(self.document)?;
        Ok(self.document)
    }
}

/// Karaoke adjuster builder
pub struct KaraokeAdjuster<'a> {
    pub(super) document: &'a mut EditorDocument,
    pub(super) range: Range,
}

impl<'a> KaraokeAdjuster<'a> {
    /// Scale timing by factor
    pub fn scale(self, factor: f32) -> Result<&'a mut EditorDocument> {
        let command = AdjustKaraokeCommand::scale(self.range, factor);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Offset timing by centiseconds
    pub fn offset(self, offset: i32) -> Result<&'a mut EditorDocument> {
        let command = AdjustKaraokeCommand::offset(self.range, offset);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Set all timing to specific duration
    pub fn set_all(self, duration: u32) -> Result<&'a mut EditorDocument> {
        let command = AdjustKaraokeCommand::set_all(self.range, duration);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Apply custom timing to each syllable
    pub fn custom(self, timings: Vec<u32>) -> Result<&'a mut EditorDocument> {
        let command = AdjustKaraokeCommand::custom(self.range, timings);
        command.execute(self.document)?;
        Ok(self.document)
    }
}

/// Karaoke applicator builder
pub struct KaraokeApplicator<'a> {
    pub(super) document: &'a mut EditorDocument,
    pub(super) range: Range,
}

impl<'a> KaraokeApplicator<'a> {
    /// Apply equal timing
    pub fn equal(self, duration: u32, karaoke_type: KaraokeType) -> Result<&'a mut EditorDocument> {
        let command = ApplyKaraokeCommand::equal(self.range, duration, karaoke_type);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Apply beat-based timing
    pub fn beat(
        self,
        bpm: u32,
        beats_per_syllable: f32,
        karaoke_type: KaraokeType,
    ) -> Result<&'a mut EditorDocument> {
        let command = ApplyKaraokeCommand::beat(self.range, bpm, beats_per_syllable, karaoke_type);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Apply pattern-based timing
    pub fn pattern(
        self,
        durations: Vec<u32>,
        karaoke_type: KaraokeType,
    ) -> Result<&'a mut EditorDocument> {
        let command = ApplyKaraokeCommand::pattern(self.range, durations, karaoke_type);
        command.execute(self.document)?;
        Ok(self.document)
    }

    /// Import timing from another event
    pub fn import_from(self, source_event_index: usize) -> Result<&'a mut EditorDocument> {
        let command = ApplyKaraokeCommand::import_from(self.range, source_event_index);
        command.execute(self.document)?;
        Ok(self.document)
    }
}
