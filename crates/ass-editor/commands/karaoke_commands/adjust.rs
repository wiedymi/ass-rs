//! Adjustment command type and timing-operation definitions for karaoke tags.

use crate::core::Range;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Adjust timing of existing karaoke tags
#[derive(Debug, Clone, PartialEq)]
pub struct AdjustKaraokeCommand {
    /// Range containing karaoke to adjust
    pub range: Range,
    /// Timing adjustment operation
    pub adjustment: TimingAdjustment,
}

/// Karaoke timing adjustment operations
#[derive(Debug, Clone, PartialEq)]
pub enum TimingAdjustment {
    /// Scale all timings by a factor (e.g., 1.2 to make 20% longer)
    Scale(f32),
    /// Add/subtract centiseconds to/from all timings
    Offset(i32),
    /// Set all timings to a specific duration
    SetAll(u32),
    /// Apply custom timing to each syllable
    Custom(Vec<u32>),
}

impl AdjustKaraokeCommand {
    /// Create a scaling adjustment command
    pub fn scale(range: Range, factor: f32) -> Self {
        Self {
            range,
            adjustment: TimingAdjustment::Scale(factor),
        }
    }

    /// Create an offset adjustment command
    pub fn offset(range: Range, offset: i32) -> Self {
        Self {
            range,
            adjustment: TimingAdjustment::Offset(offset),
        }
    }

    /// Create a set-all adjustment command
    pub fn set_all(range: Range, duration: u32) -> Self {
        Self {
            range,
            adjustment: TimingAdjustment::SetAll(duration),
        }
    }

    /// Create a custom timing adjustment command
    pub fn custom(range: Range, timings: Vec<u32>) -> Self {
        Self {
            range,
            adjustment: TimingAdjustment::Custom(timings),
        }
    }
}
