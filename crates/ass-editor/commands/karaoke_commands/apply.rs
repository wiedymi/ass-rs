//! Apply-karaoke command type and timing-template definitions.

use super::KaraokeType;
use crate::core::Range;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Apply karaoke timing to event text
#[derive(Debug, Clone, PartialEq)]
pub struct ApplyKaraokeCommand {
    /// Event range to apply karaoke to
    pub event_range: Range,
    /// Karaoke template or pattern to apply
    pub karaoke_template: KaraokeTemplate,
}

/// Karaoke template for applying timing patterns
#[derive(Debug, Clone, PartialEq)]
pub enum KaraokeTemplate {
    /// Simple equal timing for all syllables
    Equal {
        syllable_duration: u32,
        karaoke_type: KaraokeType,
    },
    /// Beat-based timing (e.g., 4/4 time signature)
    Beat {
        beats_per_minute: u32,
        beats_per_syllable: f32,
        karaoke_type: KaraokeType,
    },
    /// Custom timing pattern that repeats
    Pattern {
        durations: Vec<u32>,
        karaoke_type: KaraokeType,
    },
    /// Import timing from another event
    ImportFrom { source_event_index: usize },
}

impl ApplyKaraokeCommand {
    /// Create an equal timing command
    pub fn equal(event_range: Range, duration: u32, karaoke_type: KaraokeType) -> Self {
        Self {
            event_range,
            karaoke_template: KaraokeTemplate::Equal {
                syllable_duration: duration,
                karaoke_type,
            },
        }
    }

    /// Create a beat-based timing command
    pub fn beat(
        event_range: Range,
        bpm: u32,
        beats_per_syllable: f32,
        karaoke_type: KaraokeType,
    ) -> Self {
        Self {
            event_range,
            karaoke_template: KaraokeTemplate::Beat {
                beats_per_minute: bpm,
                beats_per_syllable,
                karaoke_type,
            },
        }
    }

    /// Create a pattern-based timing command
    pub fn pattern(event_range: Range, durations: Vec<u32>, karaoke_type: KaraokeType) -> Self {
        Self {
            event_range,
            karaoke_template: KaraokeTemplate::Pattern {
                durations,
                karaoke_type,
            },
        }
    }

    /// Create an import timing command
    pub fn import_from(event_range: Range, source_event_index: usize) -> Self {
        Self {
            event_range,
            karaoke_template: KaraokeTemplate::ImportFrom { source_event_index },
        }
    }
}
