//! Karaoke management commands for ASS karaoke timing
//!
//! Provides commands for generating, splitting, adjusting, and applying
//! ASS karaoke timing tags like \k, \kf, \ko, \kt with proper syllable
//! detection and timing validation.

mod adjust;
mod adjust_impl;
mod apply;
mod apply_impl;
mod generate;
mod split;
mod types;

#[cfg(test)]
mod tests;

pub use adjust::{AdjustKaraokeCommand, TimingAdjustment};
pub use apply::{ApplyKaraokeCommand, KaraokeTemplate};
pub use generate::GenerateKaraokeCommand;
pub use split::SplitKaraokeCommand;
pub use types::KaraokeType;
