//! Query and inspection methods for [`DocumentEvent`].
//!
//! Provides human-readable descriptions, modification/text-effect predicates,
//! affected-range computation, and the event type name used by filtering.

use super::DocumentEvent;
use crate::core::Range;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};

impl DocumentEvent {
    /// Get a human-readable description of the event
    pub fn description(&self) -> String {
        match self {
            Self::TextInserted { length, .. } => format!("Inserted {length} bytes of text"),
            Self::TextDeleted { range, .. } => format!("Deleted text from {range}"),
            Self::TextReplaced { range, .. } => format!("Replaced text in {range}"),
            Self::SelectionChanged { .. } => "Selection changed".to_string(),
            Self::CursorMoved { .. } => "Cursor moved".to_string(),
            Self::DocumentSaved { file_path, save_as } => {
                if *save_as {
                    format!("Saved document as '{file_path}'")
                } else {
                    format!("Saved document to '{file_path}'")
                }
            }
            Self::DocumentLoaded { file_path, size } => {
                format!("Loaded document '{file_path}' ({size} bytes)")
            }
            Self::UndoPerformed {
                action_description,
                changes_count,
            } => {
                format!("Undid '{action_description}' ({changes_count} changes)")
            }
            Self::RedoPerformed {
                action_description,
                changes_count,
            } => {
                format!("Redid '{action_description}' ({changes_count} changes)")
            }
            Self::ValidationCompleted {
                issues_count,
                validation_time_ms,
                ..
            } => {
                format!(
                    "Validation completed: {issues_count} issues found in {validation_time_ms}ms"
                )
            }
            Self::SearchCompleted {
                pattern,
                matches_count,
                search_time_us,
                ..
            } => {
                format!(
                    "Search for '{pattern}' found {matches_count} matches in {search_time_us}μs"
                )
            }
            Self::ParsingCompleted {
                success,
                sections_count,
                parse_time_ms,
                ..
            } => {
                if *success {
                    format!("Parsed {sections_count} sections in {parse_time_ms}ms")
                } else {
                    format!("Parsing failed after {parse_time_ms}ms")
                }
            }
            Self::ExtensionChanged {
                extension_name,
                loaded,
            } => {
                if *loaded {
                    format!("Loaded extension '{extension_name}'")
                } else {
                    format!("Unloaded extension '{extension_name}'")
                }
            }
            Self::ConfigChanged { key, .. } => format!("Configuration '{key}' changed"),
            Self::CustomEvent { event_type, .. } => format!("Custom event: {event_type}"),
        }
    }

    /// Check if this event represents a document modification
    pub fn is_modification(&self) -> bool {
        matches!(
            self,
            Self::TextInserted { .. }
                | Self::TextDeleted { .. }
                | Self::TextReplaced { .. }
                | Self::UndoPerformed { .. }
                | Self::RedoPerformed { .. }
        )
    }

    /// Check if this event affects the document's text content
    pub fn affects_text(&self) -> bool {
        matches!(
            self,
            Self::TextInserted { .. }
                | Self::TextDeleted { .. }
                | Self::TextReplaced { .. }
                | Self::UndoPerformed { .. }
                | Self::RedoPerformed { .. }
                | Self::DocumentLoaded { .. }
        )
    }

    /// Get the affected range for events that modify text
    pub fn affected_range(&self) -> Option<Range> {
        match self {
            Self::TextInserted {
                position, length, ..
            } => Some(Range::new(*position, position.advance(*length))),
            Self::TextDeleted { range, .. } | Self::TextReplaced { range, .. } => Some(*range),
            _ => None,
        }
    }
}

impl DocumentEvent {
    /// Get the event type as a string for filtering
    pub(super) fn event_type_name(&self) -> String {
        match self {
            Self::TextInserted { .. } => "TextInserted".to_string(),
            Self::TextDeleted { .. } => "TextDeleted".to_string(),
            Self::TextReplaced { .. } => "TextReplaced".to_string(),
            Self::SelectionChanged { .. } => "SelectionChanged".to_string(),
            Self::CursorMoved { .. } => "CursorMoved".to_string(),
            Self::DocumentSaved { .. } => "DocumentSaved".to_string(),
            Self::DocumentLoaded { .. } => "DocumentLoaded".to_string(),
            Self::UndoPerformed { .. } => "UndoPerformed".to_string(),
            Self::RedoPerformed { .. } => "RedoPerformed".to_string(),
            Self::ValidationCompleted { .. } => "ValidationCompleted".to_string(),
            Self::SearchCompleted { .. } => "SearchCompleted".to_string(),
            Self::ParsingCompleted { .. } => "ParsingCompleted".to_string(),
            Self::ExtensionChanged { .. } => "ExtensionChanged".to_string(),
            Self::ConfigChanged { .. } => "ConfigChanged".to_string(),
            Self::CustomEvent { event_type, .. } => event_type.clone(),
        }
    }
}
