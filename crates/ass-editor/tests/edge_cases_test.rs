//! Edge case and stress tests for ass-editor
//!
//! Tests various edge cases, boundary conditions, error scenarios, and ASS format-specific parsing

#[path = "edge_cases_test/document.rs"]
mod document;

#[path = "edge_cases_test/position_range.rs"]
mod position_range;

#[path = "edge_cases_test/undo_redo.rs"]
mod undo_redo;

#[path = "edge_cases_test/error_recovery.rs"]
mod error_recovery;

#[path = "edge_cases_test/extension.rs"]
mod extension;

#[path = "edge_cases_test/concurrent.rs"]
mod concurrent;

#[path = "edge_cases_test/memory.rs"]
mod memory;

#[path = "edge_cases_test/ass_format.rs"]
mod ass_format;

#[path = "edge_cases_test/ass_dialogue.rs"]
mod ass_dialogue;
