//! Tests for [`UndoStack`], [`UndoStackConfig`], and [`Operation`] accounting.

use super::*;
use crate::commands::CommandResult;
use crate::core::position::{Position, Range};
#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

#[test]
fn undo_stack_basic_operations() {
    let mut stack = UndoStack::new();
    assert!(!stack.can_undo());
    assert!(!stack.can_redo());

    // Create a dummy history entry
    let operation = Operation::Insert {
        position: Position::new(0),
        text: "test".to_string(),
    };
    let result = CommandResult::success();
    let entry = HistoryEntry::new(operation, "Test".to_string(), &result, None);

    stack.push(entry);
    assert!(stack.can_undo());
    assert!(!stack.can_redo());

    let popped = stack.pop_undo().unwrap();
    assert_eq!(popped.description, "Test");
    assert!(!stack.can_undo());
}

#[test]
fn undo_stack_memory_limits() {
    let config = UndoStackConfig {
        max_entries: 2,
        max_memory: 1000,
        enable_compression: false,
        arena_reset_interval: 0,
    };

    let mut stack = UndoStack::with_config(config);

    // Add entries beyond limit
    for i in 0..5 {
        let operation = Operation::Insert {
            position: Position::new(0),
            text: format!("test{i}"),
        };
        let result = CommandResult::success();
        let entry = HistoryEntry::new(operation, format!("Test {i}"), &result, None);
        stack.push(entry);
    }

    // Should be limited to 2 entries
    assert_eq!(stack.undo_count(), 2);
}

#[test]
fn operation_memory_usage() {
    let insert_op = Operation::Insert {
        position: Position::new(0),
        text: "Hello".to_string(),
    };
    assert!(insert_op.memory_usage() >= 5);

    let delete_op = Operation::Delete {
        range: Range::new(Position::new(0), Position::new(5)),
        deleted_text: "Hello".to_string(),
    };
    assert!(delete_op.memory_usage() >= 5);

    let replace_op = Operation::Replace {
        range: Range::new(Position::new(0), Position::new(5)),
        old_text: "Hello".to_string(),
        new_text: "World".to_string(),
    };
    assert!(replace_op.memory_usage() >= 10);
}

#[test]
fn undo_stack_config_default() {
    // Test that default configuration is sensible
    let default_config = UndoStackConfig::default();
    assert_eq!(default_config.max_entries, 50);
    assert_eq!(default_config.max_memory, 10 * 1024 * 1024); // 10MB
    assert!(default_config.enable_compression);
    assert_eq!(default_config.arena_reset_interval, 100);
}
