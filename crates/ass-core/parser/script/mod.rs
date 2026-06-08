//! ASS script container with zero-copy lifetime-generic design
//!
//! The `Script` struct provides the main API for accessing parsed ASS content
//! while maintaining zero-copy semantics through lifetime-generic spans.

mod auto;
mod batch;
mod builder;
mod container;
mod incremental;
mod lookup;
mod mutate;
mod parse;
mod partial;
mod serialize;
mod tracking;
mod types;
mod update;

#[cfg(feature = "stream")]
mod delta;
#[cfg(feature = "stream")]
mod delta_eq;

#[cfg(test)]
mod atomic_robustness_tests;
#[cfg(test)]
mod batch_tests;
#[cfg(test)]
mod change_equality_tests;
#[cfg(test)]
mod construction_tests;
#[cfg(test)]
mod context_tests;
#[cfg(test)]
mod mutation_tests;
#[cfg(test)]
mod parse_basic_tests;
#[cfg(test)]
mod script_misc_tests;
#[cfg(test)]
mod section_query_tests;
#[cfg(test)]
mod serialize_tests;
#[cfg(test)]
mod tracking_diff_tests;

#[cfg(all(test, feature = "stream"))]
mod stream_tests;

pub use builder::ScriptBuilder;
pub use container::Script;
pub use types::{
    BatchUpdateResult, Change, ChangeTracker, EventBatch, LineContent, StyleBatch, UpdateOperation,
};

#[cfg(feature = "stream")]
pub use delta::{calculate_delta, ScriptDelta, ScriptDeltaOwned};
