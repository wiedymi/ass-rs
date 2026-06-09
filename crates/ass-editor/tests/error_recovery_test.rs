//! Error handling and recovery tests for ass-editor
//!
//! Tests focusing on error conditions, invalid inputs, and recovery scenarios

#[path = "error_recovery_test/creation.rs"]
mod creation;

#[path = "error_recovery_test/position_range.rs"]
mod position_range;

#[path = "error_recovery_test/atomicity.rs"]
mod atomicity;

#[path = "error_recovery_test/extension.rs"]
mod extension;

#[path = "error_recovery_test/session.rs"]
mod session;

#[path = "error_recovery_test/memory.rs"]
mod memory;

#[path = "error_recovery_test/parser_state.rs"]
mod parser_state;
