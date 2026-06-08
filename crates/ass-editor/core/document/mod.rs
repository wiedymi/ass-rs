//! Main document type for the editor
//!
//! Provides the `EditorDocument` struct which manages ASS script content
//! with direct access to parsed ASS structures and efficient text editing.

mod ass_api;
mod builder_edit;
mod constructors;
mod editing;
mod editing_raw;
mod event_edit;
mod event_line;
mod incremental_parse;
mod metadata;
mod position_api;
mod section_lines;
mod text_access;
mod types;
mod undo_redo;
mod validation;

#[cfg(feature = "stream")]
mod delta_apply;
#[cfg(feature = "stream")]
mod delta_sections;
#[cfg(feature = "stream")]
mod delta_undo;
#[cfg(feature = "stream")]
mod incremental_edit;
#[cfg(feature = "stream")]
mod incremental_fast;

#[cfg(feature = "plugins")]
mod plugins;

#[cfg(test)]
mod basic_tests;
#[cfg(test)]
mod event_builder_tests;
#[cfg(test)]
mod event_index_tests;
#[cfg(test)]
mod undo_tests;
#[cfg(test)]
mod validator_tests;

#[cfg(all(test, feature = "plugins"))]
mod plugin_tests;

pub use position_api::DocumentPosition;
pub use types::EditorDocument;

#[cfg(feature = "std")]
use types::EventSender;
