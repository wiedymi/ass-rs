//! Event management commands for ASS documents
//!
//! Provides commands for splitting, merging, timing adjustments, toggling event types,
//! and effect modifications with proper validation and delta tracking.

mod batch_delete;
mod delete;
mod effect;
mod effect_exec;
mod helpers;
mod merge;
mod split;
mod timing;
mod toggle;

#[cfg(test)]
mod effect_tests;
#[cfg(test)]
mod tests;

pub use batch_delete::BatchDeleteEventsCommand;
pub use delete::DeleteEventCommand;
pub use effect::{EffectOperation, EventEffectCommand};
pub use merge::MergeEventsCommand;
pub use split::SplitEventCommand;
pub use timing::TimingAdjustCommand;
pub use toggle::ToggleEventTypeCommand;
