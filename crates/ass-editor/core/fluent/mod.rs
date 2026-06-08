//! Fluent API for document editing
//!
//! Provides an ergonomic builder pattern for document edits:
//! ```
//! # use ass_editor::{EditorDocument, Position, Range};
//! # let mut doc = EditorDocument::new();
//! # let pos = Position::new(0);
//! # let range = Range::new(Position::new(0), Position::new(5));
//! doc.at(pos).insert_text("Hello").unwrap();
//! // doc.at_line(5).replace_text("New content"); // Not yet implemented
//! doc.select(range).wrap_with_tag("\\b1", "\\b0").unwrap();
//! ```

mod document_ext;
mod event_accessor;
mod event_deleter;
mod event_merge_timing;
mod event_ops;
mod event_query;
mod event_query_exec;
mod event_query_filter;
mod event_toggle_effect;
mod karaoke;
mod karaoke_builders;
mod media;
mod position;
mod query_types;
mod script_info;
mod selection;
mod style;
mod tag;

#[cfg(test)]
mod basic_tests;
#[cfg(test)]
mod event_api_tests;
#[cfg(test)]
mod event_effects_tests;
#[cfg(test)]
mod event_ops_tests;
#[cfg(test)]
mod karaoke_apply_tests;
#[cfg(test)]
mod karaoke_tests;
#[cfg(test)]
mod karaoke_workflow_tests;
#[cfg(test)]
mod metadata_tests;
#[cfg(test)]
mod style_tests;

pub use event_accessor::EventAccessor;
pub use event_deleter::EventDeleter;
pub use event_merge_timing::{EventMerger, EventTimer};
pub use event_ops::EventOps;
pub use event_query::EventQuery;
pub use event_toggle_effect::{EventEffector, EventToggler};
pub use karaoke::{KaraokeGenerator, KaraokeOps};
pub use karaoke_builders::{KaraokeAdjuster, KaraokeApplicator, KaraokeSplitter};
pub use media::{FontsOps, GraphicsOps};
pub use position::AtPosition;
pub use query_types::{EventFilter, EventInfo, EventSortCriteria, EventSortOptions, OwnedEvent};
pub use script_info::ScriptInfoOps;
pub use selection::SelectRange;
pub use style::{StyleApplicator, StyleEditor, StyleOps};
pub use tag::TagOps;
